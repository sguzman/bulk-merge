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
        let mut config: Self = toml::from_str(&contents)?;
        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(url) = std::env::var("BULK_MERGE_POSTGRES_URL") {
            if !url.trim().is_empty() {
                self.postgres.url = Some(url);
            }
        }

        if let Ok(host) = std::env::var("BULK_MERGE_POSTGRES_HOST") {
            if !host.trim().is_empty() {
                self.postgres.host = Some(host);
            }
        }

        if let Ok(port) = std::env::var("BULK_MERGE_POSTGRES_PORT") {
            if let Ok(port) = port.trim().parse::<u16>() {
                self.postgres.port = Some(port);
            }
        }

        if let Ok(user) = std::env::var("BULK_MERGE_POSTGRES_USER") {
            if !user.trim().is_empty() {
                self.postgres.user = Some(user);
            }
        }

        if let Ok(password) = std::env::var("BULK_MERGE_POSTGRES_PASSWORD") {
            self.postgres.password = Some(password);
        }

        if let Ok(db) = std::env::var("BULK_MERGE_POSTGRES_DATABASE") {
            if !db.trim().is_empty() {
                self.postgres.database = Some(db);
            }
        }

        if let Ok(level) = std::env::var("BULK_MERGE_LOG_LEVEL") {
            if !level.trim().is_empty() {
                self.logging.level = level;
            }
        }

        if let Ok(format) = std::env::var("BULK_MERGE_LOG_FORMAT") {
            match format.trim() {
                "human" => self.logging.format = LogFormat::Human,
                "json" => self.logging.format = LogFormat::Json,
                _ => {}
            }
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let mut errors: Vec<String> = Vec::new();

        if self.postgres.url.is_none() {
            if self.postgres.host.as_deref().unwrap_or("").trim().is_empty() {
                errors.push("postgres.host must not be empty (or set postgres.url)".to_string());
            }
            if self.postgres.port.is_none() {
                errors.push("postgres.port must be set (or set postgres.url)".to_string());
            }
            if self.postgres.user.as_deref().unwrap_or("").trim().is_empty() {
                errors.push("postgres.user must not be empty (or set postgres.url)".to_string());
            }
            if self.postgres.password.as_deref().unwrap_or("").trim().is_empty() {
                errors.push("postgres.password must not be empty (or set postgres.url)".to_string());
            }
            if self.postgres.database.as_deref().unwrap_or("").trim().is_empty() {
                errors.push("postgres.database must not be empty (or set postgres.url)".to_string());
            }
        }

        if self.postgres.schema_meta.trim().is_empty() {
            errors.push("postgres.schema_meta must not be empty".to_string());
        }

        if self.postgres.schema_libgen.trim().is_empty() {
            errors.push("postgres.schema_libgen must not be empty".to_string());
        }

        if self.postgres.pool.max_connections == 0 {
            errors.push("postgres.pool.max_connections must be > 0".to_string());
        }

        if self.execution.concurrency == 0 {
            errors.push("execution.concurrency must be > 0".to_string());
        }

        if self.execution.batch.max_rows == 0 {
            errors.push("execution.batch.max_rows must be > 0".to_string());
        }

        if self.execution.batch.max_bytes == 0 {
            errors.push("execution.batch.max_bytes must be > 0".to_string());
        }

        if self.libgen.dump.max_statement_bytes == 0 {
            errors.push("libgen.dump.max_statement_bytes must be > 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("invalid config:\n- {}", errors.join("\n- ")))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default = "default_schema_meta")]
    pub schema_meta: String,
    #[serde(default = "default_schema_libgen")]
    pub schema_libgen: String,
    #[serde(default)]
    pub table_prefix: Option<String>,
    #[serde(default)]
    pub pool: PostgresPoolConfig,
    #[serde(default)]
    pub indexing: PostgresIndexingConfig,
}

impl PostgresConfig {
    pub fn connection_url(&self) -> anyhow::Result<String> {
        if let Some(url) = &self.url {
            return Ok(url.clone());
        }
        let host = self
            .host
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("postgres.host must be set when postgres.url is not set"))?;
        let port = self
            .port
            .ok_or_else(|| anyhow::anyhow!("postgres.port must be set when postgres.url is not set"))?;
        let user = self
            .user
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("postgres.user must be set when postgres.url is not set"))?;
        let password = self.password.as_deref().ok_or_else(|| {
            anyhow::anyhow!("postgres.password must be set when postgres.url is not set")
        })?;
        let database = self.database.as_deref().ok_or_else(|| {
            anyhow::anyhow!("postgres.database must be set when postgres.url is not set")
        })?;

        Ok(format!(
            "postgresql://{}:{}@{}:{}/{}",
            urlencoding::encode(user),
            urlencoding::encode(password),
            host,
            port,
            database
        ))
    }
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
    #[serde(default)]
    pub main_fields: PostgresIndexMainFields,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PostgresIndexMainFields {
    #[serde(default)]
    pub fiction: Vec<String>,
    #[serde(default)]
    pub compact: Vec<String>,
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
    #[serde(default)]
    pub tables: LibgenTablesConfig,
    #[serde(default)]
    pub resume: LibgenResumeConfig,
    #[serde(default)]
    pub incremental: LibgenIncrementalConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibgenDumpConfig {
    #[serde(default)]
    pub kind: Option<LibgenDumpKind>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub dataset_id: Option<String>,
    #[serde(default = "default_libgen_max_statement_bytes")]
    pub max_statement_bytes: u64,
    #[serde(default)]
    pub allow_invalid_utf8: bool,
}

impl Default for LibgenDumpConfig {
    fn default() -> Self {
        Self {
            kind: None,
            path: None,
            dataset_id: None,
            max_statement_bytes: default_libgen_max_statement_bytes(),
            allow_invalid_utf8: false,
        }
    }
}

fn default_libgen_max_statement_bytes() -> u64 {
    16_000_000
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibgenTablesConfig {
    #[serde(default = "default_libgen_fiction_prefix")]
    pub fiction_prefix: String,
    #[serde(default = "default_libgen_compact_prefix")]
    pub compact_prefix: String,
}

impl Default for LibgenTablesConfig {
    fn default() -> Self {
        Self {
            fiction_prefix: default_libgen_fiction_prefix(),
            compact_prefix: default_libgen_compact_prefix(),
        }
    }
}

fn default_libgen_fiction_prefix() -> String {
    "fiction_".to_string()
}

fn default_libgen_compact_prefix() -> String {
    "compact_".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibgenResumeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_libgen_checkpoint_granularity")]
    pub checkpoint_granularity: String,
}

impl Default for LibgenResumeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            checkpoint_granularity: default_libgen_checkpoint_granularity(),
        }
    }
}

fn default_libgen_checkpoint_granularity() -> String {
    "statement".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibgenIncrementalConfig {
    #[serde(default = "default_libgen_incremental_strategy")]
    pub strategy: String,
    #[serde(default)]
    pub apply_deletes: bool,
}

impl Default for LibgenIncrementalConfig {
    fn default() -> Self {
        Self {
            strategy: default_libgen_incremental_strategy(),
            apply_deletes: false,
        }
    }
}

fn default_libgen_incremental_strategy() -> String {
    "primary_key".to_string()
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
