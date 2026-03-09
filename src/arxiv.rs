//! This is the arxiv module, which defines the data structures for arXiv papers.

use crate::filter::RelevanceEvaluation;
use chrono::{Duration, NaiveDate};
use std::fs;

const MKDOCS_PAGE_TEMPLATE_PATH: &str = "./mkdocs/templates/page_template.md";
const MKDOCS_PAPER_TEMPLATE_PATH: &str = "./mkdocs/templates/paper_template.md";

#[derive(Debug)]
/// This struct represents the arXiv paper entry in search results.
pub struct ArxivPaperEntry {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
}

impl ArxivPaperEntry {
    /// Create a new ArxivPaperEntry.
    pub fn new(id: String, title: String, authors: Vec<String>, abstract_text: String) -> Self {
        Self {
            id,
            title,
            authors,
            abstract_text,
        }
    }

    pub fn get_arxiv_url(&self) -> String {
        format!("https://arxiv.org/abs/{}", self.id)
    }

    pub fn get_pdf_url(&self) -> String {
        format!("https://arxiv.org/pdf/{}.pdf", self.id)
    }

    pub fn get_src_url(&self) -> String {
        format!("https://arxiv.org/src/{}", self.id)
    }
}

/// Render one topic page by filling `page_template.md` and `paper_template.md`.
pub fn render_mkdocs_page(
    filter_results: Vec<(ArxivPaperEntry, RelevanceEvaluation)>,
    topic_name: &str,
    topic_description: &str,
    date: NaiveDate,
    ai_name: &str,
) -> Result<String, String> {
    let page_template = fs::read_to_string(MKDOCS_PAGE_TEMPLATE_PATH)
        .map_err(|err| format!("read page template failed: {err}"))?;
    let paper_template = fs::read_to_string(MKDOCS_PAPER_TEMPLATE_PATH)
        .map_err(|err| format!("read paper template failed: {err}"))?;

    let mut filter_results = filter_results;
    filter_results.sort_by(|a, b| {
        b.1.overall_score
            .partial_cmp(&a.1.overall_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let papers = filter_results
        .into_iter()
        .map(|(entry, evaluation)| {
            paper_template
                .replace("{paper_title}", &entry.title)
                .replace("{topic_relevance}", &format!("{evaluation}"))
                .replace("{AI_name}", ai_name)
                .replace("{abstract}", &entry.abstract_text)
                .replace("{arXiv_link}", &entry.get_arxiv_url())
                .replace("{pdf_link}", &entry.get_pdf_url())
                .replace("{src_link}", &entry.get_src_url())
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let yesterday = date - Duration::days(1);
    let tomorrow = date + Duration::days(1);

    let rendered = page_template
        .replace("{date}", &date.format("%Y-%m-%d").to_string())
        .replace("{topic}", topic_description)
        .replace("{papers}", &papers)
        .replace(
            "{yesterday_link}",
            &mkdocs_topic_summary_page_path(topic_name, yesterday),
        )
        .replace(
            "{tomorrow_link}",
            &mkdocs_topic_summary_page_path(topic_name, tomorrow),
        );

    Ok(rendered)
}

/// Convert topic name to a safe folder name.
///
/// Rules:
/// - Replace whitespace with `_`
/// - Remove potentially dangerous path characters: `.`, `?`, `/`, `\`
/// - Collapse repeated `_`
/// - Trim leading/trailing `_`
pub fn sanitize_topic_name_for_path(topic_name: &str) -> String {
    let mut out = String::with_capacity(topic_name.len());

    for ch in topic_name.chars() {
        if ch.is_whitespace() {
            out.push('_');
        } else if ch != '.' && ch != '?' && ch != '/' && ch != '\\' {
            out.push(ch);
        }
    }

    let mut collapsed = String::with_capacity(out.len());
    let mut prev_underscore = false;
    for ch in out.chars() {
        if ch == '_' {
            if !prev_underscore {
                collapsed.push(ch);
            }
            prev_underscore = true;
        } else {
            collapsed.push(ch);
            prev_underscore = false;
        }
    }

    let trimmed = collapsed.trim_matches('_');
    if trimmed.is_empty() {
        "topic".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Build the markdown path for one topic summary page.
///
/// Path format: `./mkdocs/docs/{topic_name}/{date}.md`
pub fn mkdocs_topic_summary_page_path(topic_name: &str, date: NaiveDate) -> String {
    let safe_topic_name = sanitize_topic_name_for_path(topic_name);
    format!(
        "./mkdocs/docs/{}/{}.md",
        safe_topic_name,
        date.format("%Y-%m-%d")
    )
}
