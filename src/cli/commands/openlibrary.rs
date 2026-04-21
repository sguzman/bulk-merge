use crate::cli::args::OpenlibraryCommand;
use crate::cli::Args;
use crate::config::AppConfig;
use crate::db::{Db, ImportRunStatus};
use crate::openlibrary::ingest::{ingest_openlibrary_dump, OlIngestPlan};
use tracing::info;

pub async fn run(_args: &Args, config: &AppConfig, command: OpenlibraryCommand) -> anyhow::Result<()> {
    match command {
        OpenlibraryCommand::Ingest {
            authors,
            editions,
            works,
            dataset_id,
        } => {
            let db = Db::connect(config).await?;
            db.migrate().await?;

            let dataset_id = dataset_id
                .or_else(|| config.openlibrary.dump.dataset_id.clone())
                .unwrap_or_else(|| "openlibrary-default".to_string());

            let import_run_id = db
                .create_import_run(
                    "openlibrary",
                    &dataset_id,
                    None,
                    ImportRunStatus::InProgress,
                    serde_json::to_value(&config.openlibrary)?,
                )
                .await?;

            info!(import_run_id, dataset_id, "started openlibrary ingestion");

            let authors_path = authors.or_else(|| config.openlibrary.dump.authors.clone());
            let editions_path = editions.or_else(|| config.openlibrary.dump.editions.clone());
            let works_path = works.or_else(|| config.openlibrary.dump.works.clone());

            if let Some(path) = authors_path {
                info!(path, "ingesting authors");
                let plan = OlIngestPlan {
                    dump_path: path,
                    table_name: "authors".to_string(),
                    schema: config.postgres.schema_openlibrary.clone(),
                };
                ingest_openlibrary_dump(&db, config, &plan, import_run_id).await?;
            }

            if let Some(path) = editions_path {
                info!(path, "ingesting editions");
                let plan = OlIngestPlan {
                    dump_path: path,
                    table_name: "editions".to_string(),
                    schema: config.postgres.schema_openlibrary.clone(),
                };
                ingest_openlibrary_dump(&db, config, &plan, import_run_id).await?;
            }

            if let Some(path) = works_path {
                info!(path, "ingesting works");
                let plan = OlIngestPlan {
                    dump_path: path,
                    table_name: "works".to_string(),
                    schema: config.postgres.schema_openlibrary.clone(),
                };
                ingest_openlibrary_dump(&db, config, &plan, import_run_id).await?;
            }

            db.finish_import_run(import_run_id, ImportRunStatus::Succeeded)
                .await?;

            // Also update dataset state
            db.upsert_dataset_state("openlibrary", &dataset_id, "all", import_run_id, None)
                .await?;

            info!(import_run_id, "openlibrary ingestion completed successfully");
            Ok(())
        }
    }
}
