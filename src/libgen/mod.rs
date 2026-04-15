//! LibGen ingestion domain (Phase 1).
//!
//! The implementation is introduced incrementally; this module exists to keep
//! future parsing/ingestion logic out of `cli` and `db`.

pub mod mysql_dump;
pub mod ingest;
pub mod offline;
pub mod provision;
pub(crate) mod typing;
