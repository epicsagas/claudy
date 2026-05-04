use std::path::Path;

use crate::adapters::analytics::pricing::anthropic::AnthropicPricingScraper;
use crate::adapters::analytics::pricing::fetcher::ModelsDev;
use crate::adapters::analytics::pricing::merger::PricingMerger;
use crate::ports::analytics_ports::PricingStore;

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

    // Step 1: fetch models.dev data (network first, then cache fallback)
    let (models_dev_entries, used_cache) =
        match ModelsDev::new(cache_path.to_path_buf()).fetch_and_cache() {
            Ok(entries) => (entries, false),
            Err(_) => {
                let entries = ModelsDev::new(cache_path.to_path_buf()).load_cache()
                    .map_err(|e| anyhow::anyhow!("network unavailable and no local cache: {e}"))?;
                (entries, true)
            }
        };

    // Step 2: fetch Anthropic pricing (failure is non-fatal)
    let (anthropic_entries, anthropic_failed) = match AnthropicPricingScraper::fetch() {
        Ok(entries) => (entries, false),
        Err(e) => {
            warnings.push(format!(
                "Anthropic pricing page parse failed — using ratio fallback: {e}"
            ));
            (Vec::new(), true)
        }
    };

    // Step 3: merge
    let merged = PricingMerger::merge(&models_dev_entries, &anthropic_entries);

    // Step 4: upsert each pricing record
    let count = merged.len();
    for pricing in merged {
        store.upsert_model_pricing(&pricing)?;
    }

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
