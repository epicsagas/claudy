pub mod analysis;
pub mod ingestion;
pub mod pricing;
pub mod recommendations;
pub mod sqlite_store;

#[cfg(feature = "analytics-ui")]
pub mod tauri;
