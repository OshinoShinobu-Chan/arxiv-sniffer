//! Centralized constants used across the application.

pub mod app {
    pub const DEFAULT_MODEL_NAME: &str = "deepseek-chat";
    pub const DEEPSEEK_API_KEY_ENV: &str = "DEEPSEEK_API_KEY";
}

pub mod crawler {
    pub const ARXIV_CATCHUP_URL_TEMPLATE: &str =
        "https://arxiv.org/catchup/{subject_code}/{date}?abs=True";
    pub const DEFAULT_USER_AGENT: &str = "arxiv-sniffer/0.1";
}

pub mod mkdocs {
    pub const PAGE_TEMPLATE_PATH: &str = "./mkdocs/templates/page_template.md";
    pub const PAPER_TEMPLATE_PATH: &str = "./mkdocs/templates/paper_template.md";
    pub const METRICS_TEMPLATE_PATH: &str = "./mkdocs/templates/metrics_template.md";
    pub const TOPIC_RELEVANCE_TEMPLATE_PATH: &str =
        "./mkdocs/templates/topic_relevance_template.md";
    pub const DIMENSION_TEMPLATE_PATH: &str = "./mkdocs/templates/dimension_template.md";
}

pub mod filter {
    pub const WEIGHT_SUM_EPSILON: f64 = 1e-9;
}
