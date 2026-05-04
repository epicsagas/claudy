use std::path::Path;

use crate::adapters::analytics::pricing::anthropic::AnthropicPricingScraper;
use crate::adapters::analytics::pricing::fetcher::ModelsDev;
use crate::adapters::analytics::pricing::merger::PricingMerger;
use crate::ports::analytics_ports::PricingStore;

fn cache_is_fresh(cache_path: &Path, max_age_secs: u64) -> bool {
    std::fs::metadata(cache_path)
        .and_then(|m| m.modified())
        .and_then(|t| t.elapsed().map_err(std::io::Error::other))
        .map(|age| age.as_secs() < max_age_secs)
        .unwrap_or(false)
}

pub enum PricingSyncSource {
    AnthropicPlusModelsDev,
    ModelsDevRatio,
    Cache,
}

impl PricingSyncSource {
    pub fn label(&self) -> &'static str {
        match self {
            PricingSyncSource::AnthropicPlusModelsDev => "anthropic+models_dev",
            PricingSyncSource::ModelsDevRatio => "models_dev+ratio",
            PricingSyncSource::Cache => "cache",
        }
    }
}

pub struct PricingSyncResult {
    pub models_synced: usize,
    pub source: PricingSyncSource,
    pub warnings: Vec<String>,
}

pub fn run_pricing_sync(
    store: &dyn PricingStore,
    cache_path: &Path,
) -> anyhow::Result<PricingSyncResult> {
    let mut warnings: Vec<String> = Vec::new();

    // Step 1: fetch models.dev data — skip network when cache is fresh (<1 h old)
    let (models_dev_entries, used_cache) = if cache_is_fresh(cache_path, 3600) {
        match ModelsDev::new(cache_path.to_path_buf()).load_cache() {
            Ok(entries) => (entries, true),
            Err(_) => match ModelsDev::new(cache_path.to_path_buf()).fetch_and_cache() {
                Ok(entries) => (entries, false),
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "network unavailable and no local cache: {e}"
                    ))
                }
            },
        }
    } else {
        match ModelsDev::new(cache_path.to_path_buf()).fetch_and_cache() {
            Ok(entries) => (entries, false),
            Err(_) => {
                let entries = ModelsDev::new(cache_path.to_path_buf())
                    .load_cache()
                    .map_err(|e| {
                        anyhow::anyhow!("network unavailable and no local cache: {e}")
                    })?;
                (entries, true)
            }
        }
    };

    // Step 2: fetch Anthropic pricing — skip when cache was fresh (skip both network calls)
    let (anthropic_entries, anthropic_failed) = if used_cache {
        // cache was fresh — skip Anthropic fetch too
        (Vec::new(), true)
    } else {
        match AnthropicPricingScraper::fetch() {
            Ok(entries) => (entries, false),
            Err(e) => {
                warnings.push(format!(
                    "Anthropic pricing page parse failed — using ratio fallback: {e}"
                ));
                (Vec::new(), true)
            }
        }
    };

    // Step 3: merge
    let merged = PricingMerger::merge(&models_dev_entries, &anthropic_entries);

    // Step 4: batch-upsert all pricing records in one transaction
    let count = merged.len();
    store.batch_upsert_model_pricing(&merged)?;

    // Step 5: determine source
    let source = if used_cache {
        PricingSyncSource::Cache
    } else if anthropic_failed {
        PricingSyncSource::ModelsDevRatio
    } else {
        PricingSyncSource::AnthropicPlusModelsDev
    };

    Ok(PricingSyncResult {
        models_synced: count,
        source,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore;
    use crate::ports::analytics_ports::AnalyticsStore;
    use tempfile::{NamedTempFile, TempDir};

    /// Build a minimal models.dev JSON cache with one claude entry that has cost data.
    fn write_models_dev_cache(dir: &TempDir) -> std::path::PathBuf {
        let cache_path = dir.path().join("models_dev.json");
        let json = r#"[
            {
                "id": "claude-haiku-4-5",
                "name": "Claude Haiku 4.5",
                "cost": {
                    "input": 0.80,
                    "output": 4.00,
                    "cache_read": 0.08,
                    "cache_write": 1.00
                }
            }
        ]"#;
        std::fs::write(&cache_path, json).unwrap();
        cache_path
    }

    fn open_store(db_file: &NamedTempFile) -> SqliteAnalyticsStore {
        let store = SqliteAnalyticsStore::open(db_file.path().to_str().unwrap()).unwrap();
        store.initialize_schema().unwrap();
        store
    }

    /// When Anthropic scraper returns no entries (empty vec simulated via cache-only path),
    /// the source must be ModelsDevRatio and exactly 1 model should be synced.
    #[test]
    fn test_sync_with_empty_anthropic_falls_back_to_ratio() {
        let tmp_dir = TempDir::new().unwrap();
        let cache_path = write_models_dev_cache(&tmp_dir);
        let db_file = NamedTempFile::new().unwrap();
        let store = open_store(&db_file);

        // We can't intercept network calls in unit tests, so we exercise the
        // fallback path by calling PricingMerger directly via run_pricing_sync
        // with a pre-seeded cache and relying on the fact that fetch_and_cache
        // will fail in the test environment (no network or wrong URL won't matter
        // because we only care about the cache fallback logic).
        //
        // Instead, test the logic directly: merge with empty anthropic list → ratio source.
        let models_dev_entries =
            ModelsDev::new(cache_path.clone()).load_cache().unwrap();
        let anthropic_entries: Vec<
            crate::adapters::analytics::pricing::anthropic::AnthropicModelPrice,
        > = Vec::new();

        let merged = PricingMerger::merge(&models_dev_entries, &anthropic_entries);
        assert_eq!(merged.len(), 1, "should produce one merged entry");
        assert_eq!(merged[0].source, "models_dev+ratio");
        assert_eq!(merged[0].model_id, "claude-haiku-4-5");

        // Upsert into the store
        for p in &merged {
            store.upsert_model_pricing(p).unwrap();
        }

        let rows = store.list_model_pricing().unwrap();
        assert_eq!(rows.len(), 1);
    }

    /// cache_is_fresh returns true when file was just written.
    #[test]
    fn test_cache_is_fresh_returns_true_for_new_file() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("fresh.json");
        std::fs::write(&path, b"{}").unwrap();
        assert!(cache_is_fresh(&path, 3600), "newly written file must be fresh");
    }

    /// cache_is_fresh returns false when file does not exist.
    #[test]
    fn test_cache_is_fresh_returns_false_for_missing_file() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("nonexistent.json");
        assert!(!cache_is_fresh(&path, 3600), "missing file must not be fresh");
    }

    /// batch_upsert_model_pricing stores all rows inside one transaction.
    #[test]
    fn test_batch_upsert_model_pricing_inserts_all_rows() {
        use crate::domain::analytics::ModelPricing;
        let db_file = NamedTempFile::new().unwrap();
        let store = open_store(&db_file);

        let pricings = vec![
            ModelPricing {
                model_id: "claude-opus-4".to_string(),
                input: 15.0,
                output: 75.0,
                cache_write: 18.75,
                cache_read: 1.5,
                source: "anthropic+models_dev".to_string(),
                synced_at: "2026-05-04T00:00:00Z".to_string(),
            },
            ModelPricing {
                model_id: "claude-haiku-4".to_string(),
                input: 0.8,
                output: 4.0,
                cache_write: 1.0,
                cache_read: 0.08,
                source: "models_dev+ratio".to_string(),
                synced_at: "2026-05-04T00:00:00Z".to_string(),
            },
        ];

        store.batch_upsert_model_pricing(&pricings).unwrap();
        let rows = store.list_model_pricing().unwrap();
        assert_eq!(rows.len(), 2, "both rows must be stored");
    }

    /// Running sync twice must not duplicate rows — upsert semantics.
    #[test]
    fn test_upsert_idempotency_no_duplicates() {
        let tmp_dir = TempDir::new().unwrap();
        let cache_path = write_models_dev_cache(&tmp_dir);
        let db_file = NamedTempFile::new().unwrap();
        let store = open_store(&db_file);

        let models_dev_entries = ModelsDev::new(cache_path).load_cache().unwrap();
        let anthropic_entries: Vec<
            crate::adapters::analytics::pricing::anthropic::AnthropicModelPrice,
        > = Vec::new();

        // First sync
        let merged1 = PricingMerger::merge(&models_dev_entries.clone(), &anthropic_entries);
        for p in &merged1 {
            store.upsert_model_pricing(p).unwrap();
        }
        let rows_after_first = store.list_model_pricing().unwrap();

        // Second sync (same data)
        let merged2 = PricingMerger::merge(&models_dev_entries, &anthropic_entries);
        for p in &merged2 {
            store.upsert_model_pricing(p).unwrap();
        }
        let rows_after_second = store.list_model_pricing().unwrap();

        assert_eq!(
            rows_after_first.len(),
            rows_after_second.len(),
            "row count must be stable after second upsert"
        );
        assert_eq!(rows_after_second.len(), 1);
    }
}
