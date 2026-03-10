mod ai_api;
mod arxiv;
mod config;
mod crawler;
mod filter;
mod logger;
mod mkdocs;

pub use logger::{debug, error, info, warn};

use crate::ai_api::TokenMetrics;
use crate::arxiv::ArxivPaperEntry;
use crate::config::AppConfig;
use crate::filter::RelevanceEvaluation;
use crate::mkdocs::render_mkdocs_page;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

const CATCHUP_DATE: &str = "2026-03-06";

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

#[derive(Debug, Serialize, Deserialize)]
struct FilterResultsDump {
    generated_at_utc: String,
    catchup_date: String,
    model_name: String,
    relevance_threshold: f64,
    topics: Vec<TopicDump>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TopicDump {
    topic_name: String,
    topic_description: String,
    total_candidates: usize,
    matched_count: usize,
    results: Vec<PaperDump>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PaperDump {
    id: String,
    title: String,
    authors: Vec<String>,
    abstract_text: String,
    overall_score: f64,
    dimensional_scores: HashMap<String, u8>,
    dimensional_reasons: HashMap<String, String>,
}

fn run_catchup_filter_and_render(_config: &AppConfig) {
    let _timer = ScopeTimer::new("run_catchup_filter_and_render");
    let dump_path = format!("/tmp/filter_results_{}.json", CATCHUP_DATE);
    info(format!("start render demo from dump: {}", dump_path));

    let started_at = Instant::now();
    let dump = match load_filter_results_dump(&dump_path) {
        Ok(v) => v,
        Err(err) => {
            error(format!("load filter results dump failed: {}", err));
            return;
        }
    };

    let target_naive_date = match parse_catchup_naive_date(&dump.catchup_date) {
        Ok(v) => v,
        Err(err) => {
            error(format!(
                "invalid catchup date '{}': {err}",
                dump.catchup_date
            ));
            return;
        }
    };

    info(format!(
        "loaded dump with {} topics, sleep 5 minutes to simulate real runtime",
        dump.topics.len()
    ));
    std::thread::sleep(Duration::from_secs(5 * 60));

    for topic in dump.topics {
        let filter_results = topic
            .results
            .iter()
            .map(|paper| {
                (
                    ArxivPaperEntry::new(
                        paper.id.clone(),
                        paper.title.clone(),
                        paper.authors.clone(),
                        paper.abstract_text.clone(),
                    ),
                    build_relevance_evaluation_from_dump(paper),
                )
            })
            .collect::<Vec<_>>();

        match render_mkdocs_page(
            filter_results,
            &topic.topic_name,
            &topic.topic_description,
            target_naive_date,
            &dump.model_name,
            TokenMetrics::new(),
            started_at.elapsed(),
        ) {
            Ok(page) => {
                info(format!(
                    "mkdocs page for topic '{}' on {} rendered",
                    topic.topic_name, dump.catchup_date
                ));
                println!("{}", page);
            }
            Err(err) => {
                error(format!(
                    "render mkdocs page failed for topic '{}': {}",
                    topic.topic_name, err
                ));
            }
        }
    }
}

fn load_filter_results_dump(path: &str) -> Result<FilterResultsDump, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("read dump file failed: {err}"))?;
    serde_json::from_str::<FilterResultsDump>(&raw)
        .map_err(|err| format!("parse dump json failed: {err}"))
}

fn build_relevance_evaluation_from_dump(paper: &PaperDump) -> RelevanceEvaluation {
    RelevanceEvaluation {
        dimensional_scores: paper.dimensional_scores.clone(),
        dimensional_reasons: paper.dimensional_reasons.clone(),
        key_to_name: HashMap::new(),
        key_to_description: HashMap::new(),
        key_to_weight: HashMap::new(),
        overall_score: paper.overall_score,
    }
}

fn parse_catchup_naive_date(date_text: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_text, "%Y-%m-%d")
        .map_err(|err| format!("invalid format, expected YYYY-MM-DD: {err}"))
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
