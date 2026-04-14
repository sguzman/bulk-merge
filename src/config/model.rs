use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub postgres: PostgresConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub libgen: LibgenConfig,
}

impl AppConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub url: String,
    #[serde(default = "default_schema_meta")]
    pub schema_meta: String,
    #[serde(default = "default_schema_libgen")]
    pub schema_libgen: String,
    #[serde(default)]
    pub pool: PostgresPoolConfig,
    #[serde(default)]
    pub indexing: PostgresIndexingConfig,
}

fn default_schema_meta() -> String {
    "bm_meta".to_string()
}

fn default_schema_libgen() -> String {
    "src_libgen".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PostgresPoolConfig {
    #[serde(default = "default_pool_max_connections")]
    pub max_connections: u32,
    #[serde(default)]
    pub min_connections: u32,
    #[serde(default = "default_pool_acquire_timeout_ms")]
    pub acquire_timeout_ms: u64,
}

fn default_pool_max_connections() -> u32 {
    10
}

fn default_pool_acquire_timeout_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PostgresIndexingConfig {
    #[serde(default = "default_true")]
    pub create_after_load: bool,
    #[serde(default)]
    pub concurrent: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: LogFormat,
    #[serde(default)]
    pub include_target: bool,
    #[serde(default)]
    pub include_location: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            include_target: false,
            include_location: false,
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> LogFormat {
    LogFormat::Human
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionConfig {
    #[serde(default)]
    pub dry_run_default: bool,
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,
    #[serde(default)]
    pub batch: BatchConfig,
    #[serde(default)]
    pub retry: RetryConfig,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            dry_run_default: false,
            concurrency: default_concurrency(),
            batch: BatchConfig::default(),
            retry: RetryConfig::default(),
        }
    }
}

fn default_concurrency() -> u32 {
    2
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchConfig {
    #[serde(default = "default_batch_max_rows")]
    pub max_rows: u64,
    #[serde(default = "default_batch_max_bytes")]
    pub max_bytes: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_rows: default_batch_max_rows(),
            max_bytes: default_batch_max_bytes(),
        }
    }
}

fn default_batch_max_rows() -> u64 {
    25_000
}

fn default_batch_max_bytes() -> u64 {
    64_000_000
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_retry_max_attempts")]
    pub max_attempts: u32,
    #[serde(default = "default_retry_backoff_ms_initial")]
    pub backoff_ms_initial: u64,
    #[serde(default = "default_retry_backoff_ms_max")]
    pub backoff_ms_max: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: default_retry_max_attempts(),
            backoff_ms_initial: default_retry_backoff_ms_initial(),
            backoff_ms_max: default_retry_backoff_ms_max(),
        }
    }
}

fn default_retry_max_attempts() -> u32 {
    3
}

fn default_retry_backoff_ms_initial() -> u64 {
    250
}

fn default_retry_backoff_ms_max() -> u64 {
    5_000
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_format")]
    pub format: OutputFormat,
    #[serde(default = "default_output_color")]
    pub color: OutputColor,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_output_format(),
            color: default_output_color(),
        }
    }
}

fn default_output_format() -> OutputFormat {
    OutputFormat::Human
}

fn default_output_color() -> OutputColor {
    OutputColor::Auto
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputColor {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LibgenConfig {
    #[serde(default)]
    pub dump: LibgenDumpConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibgenDumpConfig {
    #[serde(default)]
    pub kind: Option<LibgenDumpKind>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub dataset_id: Option<String>,
}

impl Default for LibgenDumpConfig {
    fn default() -> Self {
        Self {
            kind: None,
            path: None,
            dataset_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LibgenDumpKind {
    Fiction,
    Compact,
}

fn default_true() -> bool {
    true
}

