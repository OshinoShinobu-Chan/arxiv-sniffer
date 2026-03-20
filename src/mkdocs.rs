//! This is the mkdocs module for rendering markdown pages.

use crate::ai_api::TokenMetrics;
use crate::arxiv::ArxivPaperEntry;
use crate::filter::RelevanceEvaluation;
use crate::r#const::mkdocs::{METRICS_TEMPLATE_PATH, PAGE_TEMPLATE_PATH, PAPER_TEMPLATE_PATH};
use chrono::{Duration as ChronoDuration, NaiveDate};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Render one topic page by filling `page_template.md` and `paper_template.md`.
pub fn render_mkdocs_page(
    filter_results: Vec<(ArxivPaperEntry, RelevanceEvaluation)>,
    topic_description: &str,
    date: NaiveDate,
    ai_name: &str,
    token_usage: TokenMetrics,
    time: Duration,
) -> Result<String, String> {
    let page_template = fs::read_to_string(PAGE_TEMPLATE_PATH)
        .map_err(|err| format!("read page template failed: {err}"))?;
    let paper_template = fs::read_to_string(PAPER_TEMPLATE_PATH)
        .map_err(|err| format!("read paper template failed: {err}"))?;

    let mut filter_results = filter_results;
    filter_results.sort_by(|a, b| {
        b.1.overall_score
            .partial_cmp(&a.1.overall_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let result_len = filter_results.len();
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

    let yesterday = date - ChronoDuration::days(1);
    let tomorrow = date + ChronoDuration::days(1);
    let metrics = render_metrics(ai_name, token_usage, time)?;

    let rendered = page_template
        .replace("{date}", &date.format("%Y-%m-%d").to_string())
        .replace("{topic}", topic_description)
        .replace("{result_len}", &result_len.to_string())
        .replace("{papers}", &papers)
        .replace("{metrics}", &metrics)
        .replace(
            "{yesterday_link}",
            &format!("./{}.md", &yesterday.format("%Y-%m-%d").to_string()),
        )
        .replace(
            "{tomorrow_link}",
            &format!("./{}.md", &tomorrow.format("%Y-%m-%d").to_string()),
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
fn mkdocs_topic_summary_page_path(topic_name: &str, date: NaiveDate) -> String {
    let safe_topic_name = sanitize_topic_name_for_path(topic_name);
    format!(
        "./mkdocs/docs/{}/{}.md",
        safe_topic_name,
        date.format("%Y-%m-%d")
    )
}

/// Build a markdown section using `metrics_template.md``
fn render_metrics(
    ai_name: &str,
    token_usage: TokenMetrics,
    time: Duration,
) -> Result<String, String> {
    let metrics_template = fs::read_to_string(METRICS_TEMPLATE_PATH)
        .map_err(|err| format!("read metrics template failed: {err}"))?;

    let formatted_time = format_duration_hms(time);
    let rendered = metrics_template
        .replace("{AI_name}", ai_name)
        .replace(
            "{prompt_cache_miss_tokens}",
            &token_usage.prompt_cache_miss_tokens.to_string(),
        )
        .replace(
            "{prompt_cache_hit_tokens}",
            &token_usage.prompt_cache_hit_tokens.to_string(),
        )
        .replace(
            "{completion_tokens}",
            &token_usage.completion_tokens.to_string(),
        )
        .replace(
            "{reasoning_tokens}",
            &token_usage.reasoning_tokens.to_string(),
        )
        .replace("{total_tokens}", &token_usage.total_tokens.to_string())
        .replace("{time}", &formatted_time);

    Ok(rendered)
}

fn format_duration_hms(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {:.3}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {:.3}s", minutes, seconds)
    } else {
        format!("{:.3}s", seconds)
    }
}

/// Create the folder for one topic if it doesn't exist.
fn create_topic_folder(topic_name: &str) -> Result<(), String> {
    let safe_topic_name = sanitize_topic_name_for_path(topic_name);
    let topic_dir = format!("./mkdocs/docs/{}", safe_topic_name);
    let topic_path = Path::new(&topic_dir);

    if topic_path.exists() {
        if topic_path.is_dir() {
            return Ok(());
        }

        return Err(format!(
            "topic path exists but is not a directory: {}",
            topic_dir
        ));
    }

    fs::create_dir_all(&topic_dir)
        .map_err(|err| format!("create topic folder failed (path='{}'): {err}", topic_dir))
}

pub fn create_mkdocs_page(
    filter_results: Vec<(ArxivPaperEntry, RelevanceEvaluation)>,
    topic_name: &str,
    topic_description: &str,
    date: NaiveDate,
    ai_name: &str,
    token_usage: TokenMetrics,
    time: Duration,
) -> Result<(), String> {
    create_topic_folder(topic_name)?;
    let rendered = render_mkdocs_page(
        filter_results,
        topic_description,
        date,
        ai_name,
        token_usage,
        time,
    )?;
    let page_path = mkdocs_topic_summary_page_path(topic_name, date);
    fs::write(&page_path, rendered)
        .map_err(|err| format!("write mkdocs page failed (path='{}'): {err}", page_path))?;

    modify_mkdocs_nav(topic_name, date)
}

/// Modify the `nav` section in `mkdocs.yml` to add the new page link under
/// the topic section, and create the topic section if it doesn't exist.
///
/// Expected nav item format:
/// - Topic Name: topic_name/YYYY-MM-DD.md
fn modify_mkdocs_nav(topic_name: &str, date: NaiveDate) -> Result<(), String> {
    let mkdocs_yml_path = "./mkdocs/mkdocs.yml";
    let raw = fs::read_to_string(mkdocs_yml_path)
        .map_err(|err| format!("read mkdocs.yml failed (path='{}'): {err}", mkdocs_yml_path))?;

    let mut root: serde_yaml::Value = serde_yaml::from_str(&raw).map_err(|err| {
        format!(
            "parse mkdocs.yml failed (path='{}'): {err}",
            mkdocs_yml_path
        )
    })?;

    let root_map = root
        .as_mapping_mut()
        .ok_or("mkdocs.yml root must be a mapping".to_string())?;

    let nav_key = serde_yaml::Value::String("nav".to_string());
    if !root_map.contains_key(&nav_key) {
        root_map.insert(nav_key.clone(), serde_yaml::Value::Sequence(Vec::new()));
    }

    let nav = root_map
        .get_mut(&nav_key)
        .and_then(serde_yaml::Value::as_sequence_mut)
        .ok_or("mkdocs.yml field 'nav' must be a sequence".to_string())?;

    let safe_topic_name = sanitize_topic_name_for_path(topic_name);
    let entry_path = format!("{}/{}.md", safe_topic_name, date.format("%Y-%m-%d"));
    let topic_key = serde_yaml::Value::String(topic_name.to_string());
    let mut topic_found = false;

    for nav_item in nav.iter_mut() {
        let Some(item_map) = nav_item.as_mapping_mut() else {
            continue;
        };

        let Some(current_path_value) = item_map.get_mut(&topic_key) else {
            continue;
        };

        if let Some(current_path) = current_path_value.as_str() {
            if let Some(current_date) = extract_date_from_nav_path(current_path) {
                if date > current_date {
                    *current_path_value = serde_yaml::Value::String(entry_path.clone());
                }
            } else {
                *current_path_value = serde_yaml::Value::String(entry_path.clone());
            }
        } else {
            *current_path_value = serde_yaml::Value::String(entry_path.clone());
        }

        topic_found = true;
        break;
    }

    if !topic_found {
        let mut topic_entry = serde_yaml::Mapping::new();
        topic_entry.insert(topic_key, serde_yaml::Value::String(entry_path));

        nav.push(serde_yaml::Value::Mapping(topic_entry));
    }

    let serialized = serde_yaml::to_string(&root).map_err(|err| {
        format!(
            "serialize mkdocs.yml failed (path='{}'): {err}",
            mkdocs_yml_path
        )
    })?;

    fs::write(mkdocs_yml_path, serialized).map_err(|err| {
        format!(
            "write mkdocs.yml failed (path='{}'): {err}",
            mkdocs_yml_path
        )
    })
}

fn extract_date_from_nav_path(path: &str) -> Option<NaiveDate> {
    let stem = Path::new(path).file_stem()?.to_str()?;
    NaiveDate::parse_from_str(stem, "%Y-%m-%d").ok()
}
