mod ai_api;
mod arxiv;
mod config;
mod r#const;
mod crawler;
mod filter;
mod logger;
mod mkdocs;

pub use logger::{debug, error, info, warn};

use crate::ai_api::AiClient;
use crate::ai_api::deepseek::DeepSeekClient;
use crate::arxiv::ArxivPaperEntry;
use crate::config::AppConfig;
use crate::r#const::app::{DEEPSEEK_API_KEY_ENV, DEFAULT_MODEL_NAME};
use crate::crawler::ArxivCrawler;
use crate::filter::{TopicFilter, load_relevance_dimensions, load_relevance_template};
use crate::mkdocs::{create_mkdocs_page, sanitize_topic_name_for_path};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, Utc};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

fn run_app(config: &AppConfig) {
    info("start production workflow");

    if config.topics().is_empty() {
        warn("config has no topics, nothing to process");
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
    let deepseek_client = Arc::new(DeepSeekClient::new(api_key, deepseek_model_cfg));
    let ai_client: Arc<dyn AiClient> = deepseek_client.clone();

    let mut crawler = ArxivCrawler::new(
        Duration::from_secs(config.crawler.interval_secs),
        config.crawler.timeout_secs,
        &config.crawler.subject_code,
        config.crawler.user_agent.as_deref(),
    );

    let target_naive_date = Utc::now().date_naive() - ChronoDuration::days(1);
    let target_date = match naive_date_to_system_time(target_naive_date) {
        Ok(v) => v,
        Err(err) => {
            error(format!("build crawl date failed: {err}"));
            return;
        }
    };

    info(format!("crawl catchup for date {}", target_naive_date));
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

    info(format!("loaded {} entries from catchup", entries.len()));

    for topic in config.topics() {
        let page_path = mkdocs_topic_page_path(&topic.name, target_naive_date);
        if Path::new(&page_path).exists() {
            info(format!(
                "skip topic '{}' because page already exists: {}",
                topic.name, page_path
            ));
            continue;
        }

        info(format!(
            "topic '{}' start sequential filtering over {} papers",
            topic.name,
            entries.len()
        ));

        let topic_started_at = Instant::now();
        let filter = TopicFilter::new(
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

        let token_usage = deepseek_client.get_token_metrics();
        if let Err(err) = create_mkdocs_page(
            filter_results,
            &topic.name,
            &topic.description,
            target_naive_date,
            DEFAULT_MODEL_NAME,
            token_usage,
            topic_started_at.elapsed(),
        ) {
            error(format!(
                "create mkdocs page failed for topic '{}': {}",
                topic.name, err
            ));
        } else {
            info(format!("topic '{}' page written to mkdocs", topic.name));
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

fn mkdocs_topic_page_path(topic_name: &str, date: NaiveDate) -> String {
    let safe_topic_name = sanitize_topic_name_for_path(topic_name);
    format!(
        "./mkdocs/docs/{}/{}.md",
        safe_topic_name,
        date.format("%Y-%m-%d")
    )
}

fn naive_date_to_system_time(date: NaiveDate) -> Result<SystemTime, String> {
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

    run_app(&config);
}
