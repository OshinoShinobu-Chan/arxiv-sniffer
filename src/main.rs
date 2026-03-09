mod ai_api;
mod arxiv;
mod config;
mod crawler;
mod filter;
mod logger;

pub use logger::{debug, error, info, warn};

use crate::ai_api::AiClient;
use crate::ai_api::deepseek::DeepSeekClient;
use crate::arxiv::{ArxivPaperEntry, render_mkdocs_page};
use crate::config::AppConfig;
use crate::crawler::ArxivCrawler;
use crate::filter::{TopicFilter, load_relevance_dimensions, load_relevance_template};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

const CATCHUP_DATE: &str = "2026-03-06";
const DEFAULT_MODEL_NAME: &str = "deepseek-chat";
const DEEPSEEK_API_KEY_ENV: &str = "DEEPSEEK_API_KEY";

struct ScopeTimer {
    label: &'static str,
    started_at: Instant,
}

impl ScopeTimer {
    fn new(label: &'static str) -> Self {
        Self {
            label,
            started_at: Instant::now(),
        }
    }
}

impl Drop for ScopeTimer {
    fn drop(&mut self) {
        let elapsed = self.started_at.elapsed();
        info(format!("{} elapsed: {:.3?}", self.label, elapsed));
    }
}

#[derive(Serialize)]
struct FilterResultsDump {
    generated_at_utc: String,
    catchup_date: String,
    model_name: String,
    relevance_threshold: f64,
    topics: Vec<TopicDump>,
}

#[derive(Serialize)]
struct TopicDump {
    topic_name: String,
    topic_description: String,
    total_candidates: usize,
    matched_count: usize,
    results: Vec<PaperDump>,
}

#[derive(Serialize)]
struct PaperDump {
    id: String,
    title: String,
    authors: Vec<String>,
    abstract_text: String,
    overall_score: f64,
    dimensional_scores: HashMap<String, u8>,
    dimensional_reasons: HashMap<String, String>,
}

fn run_catchup_filter_and_render(config: &AppConfig) {
    let _timer = ScopeTimer::new("run_catchup_filter_and_render");
    info("start catchup filter + mkdocs render demo");

    if config.topics().is_empty() {
        warn("config has no topics, nothing to do");
        return;
    }

    let prompts_dir = Path::new(&config.prompts.dir);

    let relevance_dimensions = match load_relevance_dimensions(prompts_dir) {
        Ok(v) => v,
        Err(err) => {
            error(format!("load relevance dimensions failed: {err}"));
            return;
        }
    };

    let relevance_template = match load_relevance_template(prompts_dir) {
        Ok(v) => v,
        Err(err) => {
            error(format!("load relevance template failed: {err}"));
            return;
        }
    };

    let api_key = match std::env::var(DEEPSEEK_API_KEY_ENV) {
        Ok(v) if !v.trim().is_empty() => v,
        _ => {
            error(format!(
                "missing environment variable {DEEPSEEK_API_KEY_ENV}; cannot call AI model"
            ));
            return;
        }
    };

    let model_cfg = match config.ai_model_config(DEFAULT_MODEL_NAME) {
        Some(cfg) => cfg,
        None => {
            error(format!(
                "model config '{DEFAULT_MODEL_NAME}' not found in config.toml"
            ));
            return;
        }
    };

    let deepseek_model_cfg = DeepSeekClient::from_ai_model_config(DEFAULT_MODEL_NAME, model_cfg);
    let deepseek_client = Arc::new(DeepSeekClient::new_with_model_config(
        api_key,
        deepseek_model_cfg,
    ));
    let ai_client: Arc<dyn AiClient> = deepseek_client.clone();

    let mut crawler = ArxivCrawler::new(
        Duration::from_secs(config.crawler.interval_secs),
        config.crawler.timeout_secs,
        config.crawler.user_agent.as_deref(),
    );

    let target_naive_date = match parse_catchup_naive_date(CATCHUP_DATE) {
        Ok(v) => v,
        Err(err) => {
            error(format!("invalid catchup date '{}': {err}", CATCHUP_DATE));
            return;
        }
    };

    let target_date = match parse_catchup_date(CATCHUP_DATE) {
        Ok(v) => v,
        Err(err) => {
            error(format!("invalid catchup date '{}': {err}", CATCHUP_DATE));
            return;
        }
    };

    let raw = match crawler.crawl_catchup_raw(target_date) {
        Ok(v) => v,
        Err(err) => {
            error(format!("crawl catchup failed: {err}"));
            return;
        }
    };

    let entries = crawler.parse_paper_entries(&raw);
    if entries.is_empty() {
        warn("catchup contains no entries");
        return;
    }

    info(format!(
        "loaded {} entries from catchup {}",
        entries.len(),
        CATCHUP_DATE
    ));

    let mut dump = FilterResultsDump {
        generated_at_utc: Utc::now().to_rfc3339(),
        catchup_date: CATCHUP_DATE.to_string(),
        model_name: DEFAULT_MODEL_NAME.to_string(),
        relevance_threshold: config.filter.relevance_threshold,
        topics: Vec::new(),
    };
    let dump_path = format!("/tmp/filter_results_{}.json", CATCHUP_DATE);

    for topic in config.topics() {
        info(format!(
            "filtering topic '{}' with {} candidate papers",
            topic.name,
            entries.len()
        ));

        let filter = TopicFilter::new(
            topic.name.clone(),
            topic.description.clone(),
            ai_client.clone(),
            &relevance_dimensions,
            &relevance_template,
            config.filter.relevance_threshold,
            config.filter.eval_concurrency,
        );

        let topic_entries = clone_entries(&entries);
        let filter_results = filter.entries_filter(topic_entries);
        info(format!(
            "topic '{}' matched {} papers",
            topic.name,
            filter_results.len()
        ));

        let topic_dump = TopicDump {
            topic_name: topic.name.clone(),
            topic_description: topic.description.clone(),
            total_candidates: entries.len(),
            matched_count: filter_results.len(),
            results: filter_results
                .iter()
                .map(|(entry, evaluation)| PaperDump {
                    id: entry.id.clone(),
                    title: entry.title.clone(),
                    authors: entry.authors.clone(),
                    abstract_text: entry.abstract_text.clone(),
                    overall_score: evaluation.overall_score,
                    dimensional_scores: evaluation.dimensional_scores.clone(),
                    dimensional_reasons: evaluation.dimensional_reasons.clone(),
                })
                .collect(),
        };
        dump.topics.push(topic_dump);

        match render_mkdocs_page(
            filter_results,
            &topic.name,
            &topic.description,
            target_naive_date,
            DEFAULT_MODEL_NAME,
        ) {
            Ok(page) => {
                info(format!(
                    "mkdocs page for topic '{}' on {}:\n{}",
                    topic.name, CATCHUP_DATE, page
                ));
            }
            Err(err) => {
                error(format!(
                    "render mkdocs page failed for topic '{}': {}",
                    topic.name, err
                ));
            }
        }
    }

    if let Err(err) = write_filter_results_dump(&dump_path, &dump) {
        error(format!(
            "write filter results dump failed (path='{}'): {}",
            dump_path, err
        ));
    } else {
        info(format!("filter results dump written to {}", dump_path));
    }

    let metrics = deepseek_client.get_token_metrics();
    info(format!(
        "deepseek token usage | total={} | completion={} | reasoning={} | prompt_cache_hit={} | prompt_cache_miss={}",
        metrics.total_tokens,
        metrics.completion_tokens,
        metrics.reasoning_tokens,
        metrics.prompt_cache_hit_tokens,
        metrics.prompt_cache_miss_tokens
    ));
}

fn clone_entries(entries: &[ArxivPaperEntry]) -> Vec<ArxivPaperEntry> {
    entries
        .iter()
        .map(|entry| {
            ArxivPaperEntry::new(
                entry.id.clone(),
                entry.title.clone(),
                entry.authors.clone(),
                entry.abstract_text.clone(),
            )
        })
        .collect()
}

fn write_filter_results_dump(
    output_path: &str,
    dump: &FilterResultsDump,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Path::new(output_path);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(dump)?;
    fs::write(output, json)?;
    Ok(())
}

fn parse_catchup_naive_date(date_text: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_text, "%Y-%m-%d")
        .map_err(|err| format!("invalid format, expected YYYY-MM-DD: {err}"))
}

fn parse_catchup_date(date_text: &str) -> Result<SystemTime, String> {
    let date = parse_catchup_naive_date(date_text)?;
    let naive = date
        .and_hms_opt(0, 0, 0)
        .ok_or("failed to build datetime at 00:00:00")?;
    let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
    Ok(dt.into())
}

fn main() {
    let config = match AppConfig::load_from_file("config.toml") {
        Ok(config) => config,
        Err(err) => {
            error(format!("load config.toml failed: {err}"));
            return;
        }
    };

    run_catchup_filter_and_render(&config);
}
