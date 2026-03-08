//! This module is used to filter the arXiv papers based on the given topics.
use crate::ai_api::AiClient;
use crate::arxiv::ArxivPaperEntry;
use crate::{debug, warn};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct TopicFilter {
    topics: String,
    /// The AI client used to filter the papaers, shared by all filter instances.
    ai_client: Arc<dyn AiClient>,
    relevance_dimensions: HashMap<String, RelevanceDimension>,
    relevance_prompt_template: String,
    /// The threshold of overall relevance score
    threshold: f64,
    /// The maximum concurrency for calling AI client.
    eval_concurrency: usize,
}

#[derive(Debug, Clone)]
pub struct RelevanceDimension {
    pub weight: f64,
    pub name: String,
    pub description: String,
}

pub struct RelevanceEvaluation {
    /// key to their score (0-10)
    pub dimensional_scores: HashMap<String, u8>,
    /// key to their reason for the assigned score
    pub dimensional_reasons: HashMap<String, String>,
    /// key to their display name, for better interpretability
    pub key_to_name: HashMap<String, String>,
    /// The overall score
    pub overall_score: f64,
}

impl TopicFilter {
    pub fn new(
        topics: String,
        ai_client: Arc<dyn AiClient>,
        relevance_dimensions: &HashMap<String, RelevanceDimension>,
        relevance_template: &str,
        threshold: f64,
        eval_concurrency: usize,
    ) -> Self {
        let rendered_template =
            render_relevance_template(relevance_template, &topics, relevance_dimensions);

        Self {
            topics,
            ai_client,
            relevance_dimensions: relevance_dimensions.clone(),
            relevance_prompt_template: rendered_template,
            threshold,
            eval_concurrency,
        }
    }

    pub fn relevance_dimensions(&self) -> &HashMap<String, RelevanceDimension> {
        &self.relevance_dimensions
    }

    pub fn relevance_prompt_template(&self) -> &str {
        &self.relevance_prompt_template
    }

    pub fn check_relevance(
        &self,
        title: String,
        abstract_text: String,
    ) -> Result<RelevanceEvaluation, String> {
        let prompt = self
            .relevance_prompt_template
            .replace("{title}", &title)
            .replace("{abstract}", &abstract_text);

        let ai_response = self.ai_client.chat_completions_json(prompt)?;
        let dimensional_scores = ai_response
            .get("dimensional_scores")
            .and_then(serde_json::Value::as_object)
            .ok_or("AI response missing object field 'dimensional_scores'".to_string())?;

        let mut scores = HashMap::with_capacity(self.relevance_dimensions.len());
        let mut reasons = HashMap::with_capacity(self.relevance_dimensions.len());
        let mut key_to_name = HashMap::with_capacity(self.relevance_dimensions.len());
        let mut weighted_score_sum = 0.0_f64;

        for (key, dimension) in &self.relevance_dimensions {
            let score_item = dimensional_scores
                .get(key)
                .ok_or_else(|| format!("AI response missing dimension key '{key}'"))?;

            let score_u64 = score_item
                .get("score")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| format!("dimension '{key}' missing integer field 'score'"))?;

            if score_u64 > 10 {
                return Err(format!(
                    "dimension '{key}' has invalid score {}, expected 0..=10",
                    score_u64
                ));
            }

            let score = u8::try_from(score_u64)
                .map_err(|_| format!("dimension '{key}' score overflow: {score_u64}"))?;

            let reason = score_item
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("dimension '{key}' missing string field 'reason'"))?
                .to_string();

            scores.insert(key.clone(), score);
            reasons.insert(key.clone(), reason);
            key_to_name.insert(key.clone(), dimension.name.clone());
            weighted_score_sum += f64::from(score) * dimension.weight;
        }

        // Each dimension score is 0..=10, and weights sum to 1.0,
        // so convert weighted average to a 0..=100 overall score.
        let overall_score = weighted_score_sum * 10.0;

        Ok(RelevanceEvaluation {
            dimensional_scores: scores,
            dimensional_reasons: reasons,
            key_to_name,
            overall_score,
        })
    }

    /// This function is used to filter the papers based on the given topics.
    pub fn entries_filter(
        &self,
        entries: Vec<ArxivPaperEntry>,
    ) -> Vec<(ArxivPaperEntry, RelevanceEvaluation)> {
        let total = entries.len();
        if total == 0 {
            return Vec::new();
        }

        let worker_count = self.eval_concurrency.max(1).min(total);
        let base_chunk_size = total / worker_count;
        let remainder = total % worker_count;

        let mut iter = entries.into_iter();
        let mut chunks: Vec<Vec<ArxivPaperEntry>> = Vec::with_capacity(worker_count);
        for worker_idx in 0..worker_count {
            let this_chunk_size = base_chunk_size + usize::from(worker_idx < remainder);
            let chunk: Vec<ArxivPaperEntry> = iter.by_ref().take(this_chunk_size).collect();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
        }

        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(chunks.len());
            for chunk in chunks {
                handles.push(scope.spawn(move || {
                    let mut local_results = Vec::new();
                    for entry in chunk {
                        match self.check_relevance(entry.title.clone(), entry.abstract_text.clone()) {
                            Ok(evaluation) if evaluation.overall_score >= self.threshold => {
                                debug(format!(
                                    "arXiv:{} passed the filter with overall score {:.2}",
                                    entry.id, evaluation.overall_score
                                ));
                                local_results.push((entry, evaluation));
                            }
                            Ok(evaluation) => {
                                debug(format!(
                                    "arXiv:{} did not pass the filter with overall score below threshold: {:.2} < {:.2}",
                                    entry.id, evaluation.overall_score, self.threshold
                                ));
                            }
                            Err(err) => {
                                warn(format!(
                                    "skip arXiv:{} due to relevance evaluation error: {}",
                                    entry.id, err
                                ));
                            }
                        }
                    }
                    local_results
                }));
            }

            let mut merged = Vec::new();
            for handle in handles {
                match handle.join() {
                    Ok(mut local_results) => merged.append(&mut local_results),
                    Err(_) => warn("worker thread panicked during entries_filter"),
                }
            }
            merged
        })
    }
}

pub fn load_relevance_dimensions(
    prompts_dir: &Path,
) -> Result<HashMap<String, RelevanceDimension>, Box<dyn std::error::Error>> {
    let file_path = find_relevance_dimensions_file(prompts_dir)?;
    let raw = fs::read_to_string(&file_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;

    let dimensional_scores = parsed
        .get("dimensional_scores")
        .and_then(serde_json::Value::as_object)
        .ok_or("missing or invalid 'dimensional_scores' object")?;

    if dimensional_scores.is_empty() {
        return Err("relevance dimensions cannot be empty".into());
    }

    let mut dimensions = HashMap::with_capacity(dimensional_scores.len());
    let mut sum = 0.0_f64;

    for (name, item) in dimensional_scores {
        let weight = item
            .get("weight")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| format!("dimension '{name}' missing numeric 'weight'"))?;

        let display_name = item
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("dimension '{name}' missing string 'name'"))?
            .to_string();

        // Accept both `description` and the existing typo `descrtiption` for compatibility.
        let description = item
            .get("description")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                format!("dimension '{name}' missing string 'description' or 'descrtiption'")
            })?
            .to_string();

        if weight <= 0.0 {
            return Err(format!("dimension '{name}' weight must be positive").into());
        }
        sum += weight;
        dimensions.insert(
            name.clone(),
            RelevanceDimension {
                weight,
                name: display_name,
                description,
            },
        );
    }

    const EPSILON: f64 = 1e-9;
    if (sum - 1.0).abs() > EPSILON {
        return Err(format!("dimension weights must sum to 1.0, got {sum}").into());
    }

    Ok(dimensions)
}

pub fn load_relevance_template(prompts_dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = find_relevance_template_file(prompts_dir)?;
    let raw = fs::read_to_string(&file_path)?;
    Ok(raw)
}

fn render_relevance_template(
    template: &str,
    topic: &str,
    relevance_dimensions: &HashMap<String, RelevanceDimension>,
) -> String {
    let dimensions_block = build_dimensions_block(relevance_dimensions);
    let json_output = build_json_output_block(relevance_dimensions);

    template
        .replace("{topic}", topic)
        .replace("{dimonsion_num}", &relevance_dimensions.len().to_string())
        .replace("{dimonsions}", &dimensions_block)
        .replace("{json_output}", &json_output)
}

fn build_dimensions_block(relevance_dimensions: &HashMap<String, RelevanceDimension>) -> String {
    let entries = sorted_dimensions_by_weight(relevance_dimensions);

    entries
        .into_iter()
        .enumerate()
        .map(|(idx, (key, dim))| {
            format!(
                "{}. **{}**({})：{}",
                idx + 1,
                dim.name,
                key,
                dim.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_json_output_block(relevance_dimensions: &HashMap<String, RelevanceDimension>) -> String {
    let entries = sorted_dimensions_by_weight(relevance_dimensions);

    let mut dimensions_json = serde_json::Map::new();
    for (key, _dim) in entries {
        let reason_placeholder = format!("REASON_FOR_{}", key.to_ascii_uppercase());
        let value = serde_json::json!({
            "score": 0,
            "reason": reason_placeholder,
        });
        dimensions_json.insert(key.clone(), value);
    }

    let output = serde_json::json!({
        "dimensional_scores": dimensions_json,
    });

    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
}

fn sorted_dimensions_by_weight(
    relevance_dimensions: &HashMap<String, RelevanceDimension>,
) -> Vec<(&String, &RelevanceDimension)> {
    let mut entries: Vec<_> = relevance_dimensions.iter().collect();
    entries.sort_by(|a, b| {
        b.1.weight
            .partial_cmp(&a.1.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });
    entries
}

fn find_relevance_dimensions_file(
    prompts_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let primary = prompts_dir.join("relevance_dimonsions.json");
    if primary.exists() {
        return Ok(primary);
    }

    Err(format!(
        "cannot find relevance dimensions file in '{}' (expected relevance_dimonsions.json or relevance_dimonsion.json)",
        prompts_dir.display()
    )
    .into())
}

fn find_relevance_template_file(prompts_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = prompts_dir.join("relevance_template.txt");
    if path.exists() {
        return Ok(path);
    }

    Err(format!(
        "cannot find relevance template file in '{}' (expected relevance_template.txt)",
        prompts_dir.display()
    )
    .into())
}

impl std::fmt::Display for RelevanceEvaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut entries: Vec<_> = self.dimensional_scores.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        writeln!(f, "总体相关性得分: {:.2} / 100", self.overall_score)?;
        writeln!(
            f,
            "评价: {}\n",
            relevance_comment_by_overall_score(self.overall_score)
        )?;

        for (key, score) in entries {
            let name = self.key_to_name.get(key).unwrap_or(key).to_string();
            let reason = self
                .dimensional_reasons
                .get(key)
                .map(String::as_str)
                .unwrap_or("");
            writeln!(
                f,
                "- {} ({}): {} / 10\n理由：{}\n",
                name, key, score, reason
            )?;
        }

        Ok(())
    }
}

fn relevance_comment_by_overall_score(score: f64) -> &'static str {
    if score >= 85.0 {
        "高度相关。这篇论文几乎是必读的，完全切中主题。"
    } else if score >= 65.0 {
        "中度相关。论文有很强的参考价值，可能侧重点不同，但值得仔细阅读。"
    } else if score >= 40.0 {
        "低度相关。论文提供了一些背景信息或间接的方法论启示，可以作为扩展阅读。"
    } else {
        "不相关。论文内容与你的主题基本无关，可以舍弃。"
    }
}
