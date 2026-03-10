//! This is the crawler module to crawl arXiv and get the metadata of papers.

use crate::arxiv::ArxivPaperEntry;
use crate::{debug, error, info, warn};
use scraper::node::Node;
use scraper::{Html, Selector};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const ARXIV_CATCHUP_URL_TEMPLATE: &str = "https://arxiv.org/catchup/{subject_code}/{date}?abs=True";
const DEFAULT_USER_AGENT: &str = "arxiv-sniffer/0.1";

/// This struct is the crawler to crawl arXiv and get the metadata of papers.
pub struct ArxivCrawler {
    /// Synchronous HTTP client used to fetch arXiv pages.
    client: reqwest::blocking::Client,
    /// arXiv catchup subject code (e.g. "cs", "math").
    subject_code: String,
    /// Minimal waiting time between two crawl operations.
    interval: Duration,
    /// Start time of the last crawl operation.
    last_at: Option<Instant>,
}

impl ArxivCrawler {
    /// Create a new Crawler.
    ///
    /// If `user_agent` is `None`, a default user-agent string is used.
    pub fn new(
        interval: Duration,
        timeout_secs: u64,
        subject_code: &str,
        user_agent: Option<&str>,
    ) -> Self {
        let user_agent = user_agent.unwrap_or(DEFAULT_USER_AGENT);
        let normalized_subject_code = subject_code.to_lowercase();
        let client = reqwest::blocking::Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("failed to build reqwest blocking client");

        Self {
            client,
            subject_code: normalized_subject_code,
            interval,
            last_at: None,
        }
    }

    /// Runs a crawl operation under a global minimal-interval throttle.
    ///
    /// Future crawl operations should also go through this method,
    /// so the same rate-limit policy is consistently applied.
    fn run_crawl_operation<T, F>(&mut self, operation: F) -> T
    where
        F: FnOnce(&reqwest::blocking::Client) -> T,
    {
        if let Some(last_started_at) = self.last_at {
            let elapsed = last_started_at.elapsed();
            if elapsed < self.interval {
                let wait_for = self.interval - elapsed;
                warn(format!(
                    "throttle active, wait {:?} before next crawl",
                    wait_for
                ));
                thread::sleep(wait_for);
            }
        }

        self.last_at = Some(Instant::now());
        debug("starting crawl operation");

        operation(&self.client)
    }

    /// Crawl arXiv and get raw page content of catchup with given date.
    pub fn crawl_catchup_raw(&mut self, date: SystemTime) -> Result<String, reqwest::Error> {
        let subject_code = self.subject_code.clone();
        self.run_crawl_operation(|client| {
            let date_text = Self::format_catchup_date(date);
            let encoded_date = urlencoding::encode(&date_text);
            let url = ARXIV_CATCHUP_URL_TEMPLATE
                .replace("{subject_code}", subject_code.as_str())
                .replace("{date}", encoded_date.as_ref());
            debug(format!("crawl_catchup_raw request url={}", url));

            let result = client
                .get(url)
                .send()
                .and_then(|resp| resp.error_for_status())
                .and_then(|resp| resp.text());

            match &result {
                Ok(body) => info(format!("crawl_catchup_raw success, bytes={}", body.len())),
                Err(err) => error(format!("crawl_catchup_raw failed: {:?}", err)),
            }

            result
        })
    }

    fn format_catchup_date(date: SystemTime) -> String {
        // arXiv catchup path expects YYYY-MM-DD.
        let date_utc: chrono::DateTime<chrono::Utc> = date.into();
        date_utc.format("%Y-%m-%d").to_string()
    }

    /// Parse the raw page content into a list of paper entries.
    pub fn parse_paper_entries(&mut self, raw: &str) -> Vec<ArxivPaperEntry> {
        let all_entries = Self::extract_entries_from_html(raw);
        if all_entries.is_empty() {
            warn("parse_paper_entries found no entries");
            return all_entries;
        }

        info(format!(
            "parse_paper_entries extracted {} entries",
            all_entries.len()
        ));

        all_entries
    }

    fn extract_entries_from_html(raw: &str) -> Vec<ArxivPaperEntry> {
        let doc = Html::parse_document(raw);
        let articles_sel = Selector::parse("dl#articles").expect("invalid selector");
        let dt_sel = Selector::parse("dt").expect("invalid selector");
        let dd_sel = Selector::parse("dd").expect("invalid selector");
        let id_link_sel = Selector::parse("a[title='Abstract']").expect("invalid selector");
        let title_sel = Selector::parse("div.list-title").expect("invalid selector");
        let author_sel = Selector::parse("div.list-authors a").expect("invalid selector");
        let abstract_sel = Selector::parse("p.mathjax").expect("invalid selector");

        let mut entries = Vec::new();
        let Some(articles) = doc.select(&articles_sel).next() else {
            return entries;
        };

        for (dt, dd) in articles.select(&dt_sel).zip(articles.select(&dd_sel)) {
            let id = dt
                .select(&id_link_sel)
                .filter_map(|a| a.value().attr("href"))
                .find_map(Self::extract_arxiv_id_from_href)
                .unwrap_or_default();

            let title = dd
                .select(&title_sel)
                .next()
                .map(|node| node.text().collect::<Vec<_>>().join(" "))
                .map(Self::normalize_spaces)
                .map(|text| Self::strip_descriptor_prefix(text, "Title:"))
                .unwrap_or_default();

            let authors = dd
                .select(&author_sel)
                .map(|node| node.text().collect::<Vec<_>>().join(" "))
                .map(Self::normalize_spaces)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

            let abstract_text = dd
                .select(&abstract_sel)
                .next()
                .map(Self::extract_plain_text_abstract)
                .unwrap_or_default();

            if id.is_empty() || title.is_empty() {
                continue;
            }

            entries.push(ArxivPaperEntry::new(id, title, authors, abstract_text));
        }

        entries
    }

    fn extract_arxiv_id_from_href(href: &str) -> Option<String> {
        if let Some(pos) = href.find("/abs/") {
            let id = href[(pos + 5)..].trim();
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }

        None
    }

    fn normalize_spaces(input: String) -> String {
        input.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn extract_plain_text_abstract(node: scraper::ElementRef<'_>) -> String {
        let mut output = String::new();

        for current in node.descendants() {
            match current.value() {
                Node::Text(text) => {
                    let mut ignored = false;
                    for ancestor in current.ancestors() {
                        let Node::Element(element) = ancestor.value() else {
                            continue;
                        };

                        if element.name() == "script" {
                            ignored = true;
                            break;
                        }

                        if element.name() == "span"
                            && let Some(class_attr) = element.attr("class")
                        {
                            let classes = class_attr.split_whitespace();
                            if classes
                                .clone()
                                .any(|name| name == "MathJax" || name == "MathJax_Preview")
                            {
                                ignored = true;
                                break;
                            }
                        }
                    }

                    if ignored {
                        continue;
                    }

                    output.push_str(text);
                }
                Node::Element(element) => {
                    if element.name() == "script" && element.attr("type") == Some("math/tex") {
                        let mut tex = String::new();
                        for child in current.children() {
                            if let Node::Text(text) = child.value() {
                                tex.push_str(text);
                            }
                        }

                        let tex = tex.trim().to_string();
                        if !tex.is_empty() {
                            if !output.is_empty() && !output.ends_with(' ') {
                                output.push(' ');
                            }
                            output.push('$');
                            output.push_str(&tex);
                            output.push('$');
                            output.push(' ');
                        }
                    }
                }
                _ => {}
            }
        }

        Self::normalize_spaces(output)
    }

    fn strip_descriptor_prefix(text: String, prefix: &str) -> String {
        text.strip_prefix(prefix)
            .map(str::trim)
            .unwrap_or(text.as_str())
            .to_string()
    }
}
