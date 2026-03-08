mod ai_api;
mod arxiv;
mod config;
mod crawler;
mod filter;
mod logger;

pub use logger::{debug, error, info, warn};

use crate::ai_api::AiClient;
use crate::ai_api::deepseek::DeepSeekClient;
use crate::config::AppConfig;
use crate::crawler::ArxivCrawler;
use crate::filter::{TopicFilter, load_relevance_dimensions, load_relevance_template};
use chrono::{DateTime, NaiveDate, Utc};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

const CATCHUP_DATE: &str = "2026-03-06";
const TOPIC: &str = "多个Agent相互协作的Agentic AI系统有关的研究";
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

fn demo_topic_relevance_from_catchup(config: &AppConfig) {
    let _timer = ScopeTimer::new("demo_topic_relevance_from_catchup");
    info("start topic relevance demo from arXiv catchup");
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

    let filter = TopicFilter::new(
        TOPIC.to_string(),
        ai_client,
        &relevance_dimensions,
        &relevance_template,
        config.filter.relevance_threshold,
        config.filter.eval_concurrency,
    );

    let mut crawler = ArxivCrawler::new(
        Duration::from_secs(config.crawler.interval_secs),
        config.crawler.timeout_secs,
        config.crawler.user_agent.as_deref(),
    );

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
        "evaluating {} entries from catchup {}",
        entries.len(),
        CATCHUP_DATE
    ));

    let mut filtered = filter.entries_filter(entries);
    if filtered.is_empty() {
        info("no papers matched the configured relevance threshold");
    } else {
        filtered.sort_by(|a, b| {
            b.1.overall_score
                .partial_cmp(&a.1.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.id.cmp(&b.0.id))
        });

        info(format!(
            "found {} papers matching topic '{}' (sorted by overall score desc)",
            filtered.len(),
            TOPIC
        ));

        for (idx, (entry, evaluation)) in filtered.into_iter().enumerate() {
            info(format!(
                "rank #{} | id={} | title={}\n{}",
                idx + 1,
                entry.id,
                entry.title,
                evaluation
            ));
        }
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

fn parse_catchup_date(date_text: &str) -> Result<SystemTime, String> {
    let date = NaiveDate::parse_from_str(date_text, "%Y-%m-%d")
        .map_err(|err| format!("invalid format, expected YYYY-MM-DD: {err}"))?;
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

    demo_topic_relevance_from_catchup(&config);
}
