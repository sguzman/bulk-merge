use crate::config::AppConfig;
use crate::db::Db;
use tracing::{info, instrument};

#[instrument(skip_all)]
pub async fn run(config: &AppConfig) -> anyhow::Result<()> {
    info!("connecting to postgres");
    let db = Db::connect(config).await?;
    db.migrate().await?;
    info!("database initialized");
    Ok(())
}

