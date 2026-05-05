/// File-private fallback pricing per 1M tokens (USD).
/// Source: https://platform.claude.com/docs/en/about-claude/pricing
/// Named `FallbackPricing` to avoid shadowing `crate::domain::analytics::ModelPricing`.
struct FallbackPricing {
    input: f64,
    output: f64,
    cache_write: f64,
    cache_read: f64,
}

fn parse_version(model: &str) -> Option<(u32, u32)> {
    let digits_part = model
        .trim_start_matches("claude-")
        .trim_start_matches("anthropic.")
        .trim_start_matches("anthropic/");
    let parts: Vec<&str> = digits_part.split('-').collect();
    let nums: Vec<u32> = parts
        .iter()
        .rev()
        .take_while(|p| p.chars().all(|c| c.is_ascii_digit()))
        .filter_map(|p| p.parse().ok())
        .collect();
    match nums.len() {
        0 => None,
        1 => Some((nums[0], 0)),
        _ => Some((nums[1], nums[0])), // reversed: [minor, major]
    }
}

fn get_pricing(model: &str) -> FallbackPricing {
    let m = model.to_lowercase();

    // Extract version number from model ID (e.g. "claude-opus-4-7" → major=4, minor=7)
    // Pattern: <family>-<major>-<minor> or <family>-<major>
    let (major, minor) = parse_version(&m).unwrap_or((0, 0));

    if m.contains("opus") {
        // Opus 4.5+ (4.5, 4.6, 4.7, ...): $5 input
        // Opus 4.0, 4.1: $15 input
        // Opus 3: $15 input
        if major >= 4 && minor >= 5 {
            FallbackPricing { input: 5.0, output: 25.0, cache_write: 6.25, cache_read: 0.50 }
        } else {
            FallbackPricing { input: 15.0, output: 75.0, cache_write: 18.75, cache_read: 1.50 }
        }
    } else if m.contains("sonnet") {
        // All Sonnet 3.x and 4.x: $3 input
        FallbackPricing { input: 3.0, output: 15.0, cache_write: 3.75, cache_read: 0.30 }
    } else if m.contains("haiku") {
        if major >= 4 {
            // Haiku 4.5+: $1 input
            FallbackPricing { input: 1.0, output: 5.0, cache_write: 1.25, cache_read: 0.10 }
        } else {
            // Haiku 3.5: $0.80, Haiku 3: $0.25
            let is_3_5 = minor >= 5;
            if is_3_5 {
                FallbackPricing { input: 0.80, output: 4.0, cache_write: 1.0, cache_read: 0.08 }
            } else {
                FallbackPricing { input: 0.25, output: 1.25, cache_write: 0.30, cache_read: 0.03 }
            }
        }
    } else if m.contains("gpt-4o") {
        FallbackPricing { input: 2.5, output: 10.0, cache_write: 2.5, cache_read: 1.25 }
    } else if m.contains("gpt-4") {
        FallbackPricing { input: 30.0, output: 60.0, cache_write: 30.0, cache_read: 15.0 }
    } else if m.contains("deepseek") {
        FallbackPricing { input: 0.27, output: 1.10, cache_write: 0.27, cache_read: 0.07 }
    } else if m.contains("glm-5.1") {
        // z.ai pricing (models.dev)
        FallbackPricing { input: 1.4, output: 4.4, cache_write: 0.0, cache_read: 0.26 }
    } else if m.contains("glm-5-turbo") || m.contains("glm-5v") {
        FallbackPricing { input: 1.2, output: 4.0, cache_write: 0.0, cache_read: 0.24 }
    } else if m.contains("glm") {
        FallbackPricing { input: 1.0, output: 3.2, cache_write: 0.0, cache_read: 0.2 }
    } else if m.contains("qwen") {
        FallbackPricing { input: 0.4, output: 1.2, cache_write: 0.0, cache_read: 0.0 }
    } else {
        // Conservative default: Sonnet pricing
        FallbackPricing { input: 3.0, output: 15.0, cache_write: 3.75, cache_read: 0.30 }
    }
}

#[must_use]
pub fn estimate_cost(
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_creation: i64,
    cache_read: i64,
) -> f64 {
    let pricing = get_pricing(model);
    #[allow(clippy::cast_precision_loss)]
    let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input;
    #[allow(clippy::cast_precision_loss)]
    let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output;
    #[allow(clippy::cast_precision_loss)]
    let cache_write_cost = (cache_creation as f64 / 1_000_000.0) * pricing.cache_write;
    #[allow(clippy::cast_precision_loss)]
    let cache_read_cost = (cache_read as f64 / 1_000_000.0) * pricing.cache_read;
    input_cost + output_cost + cache_write_cost + cache_read_cost
}

/// Savings from prompt caching: tokens that were cache hits vs. billed at full input rate.
#[must_use]
pub fn estimate_cache_savings(model: &str, cache_read_tokens: i64) -> f64 {
    let pricing = get_pricing(model);
    #[allow(clippy::cast_precision_loss)]
    let tokens_m = cache_read_tokens as f64 / 1_000_000.0;
    // Savings = what we would have paid at full input rate minus what we actually paid
    tokens_m * (pricing.input - pricing.cache_read)
}

/// Like `estimate_cost` but uses DB pricing when available, falling back to hardcoded values.
#[must_use]
pub fn estimate_cost_with_store(
    store: &dyn crate::ports::analytics_ports::PricingStore,
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_creation: i64,
    cache_read: i64,
) -> f64 {
    let (input_rate, output_rate, cache_write_rate, cache_read_rate) =
        match store.get_model_pricing(model) {
            Ok(Some(p)) => (p.input, p.output, p.cache_write, p.cache_read),
            other => {
                if let Err(ref e) = other {
                    eprintln!("[pricing] warn: DB lookup failed for {model}: {e}");
                }
                let p = get_pricing(model);
                (p.input, p.output, p.cache_write, p.cache_read)
            }
        };
    #[allow(clippy::cast_precision_loss)]
    let input_cost = (input_tokens as f64 / 1_000_000.0) * input_rate;
    #[allow(clippy::cast_precision_loss)]
    let output_cost = (output_tokens as f64 / 1_000_000.0) * output_rate;
    #[allow(clippy::cast_precision_loss)]
    let cache_write_cost = (cache_creation as f64 / 1_000_000.0) * cache_write_rate;
    #[allow(clippy::cast_precision_loss)]
    let cache_read_cost = (cache_read as f64 / 1_000_000.0) * cache_read_rate;
    input_cost + output_cost + cache_write_cost + cache_read_cost
}

/// Like `estimate_cache_savings` but uses DB pricing when available, falling back to hardcoded values.
#[must_use]
pub fn estimate_cache_savings_with_store(
    store: &dyn crate::ports::analytics_ports::PricingStore,
    model: &str,
    cache_read_tokens: i64,
) -> f64 {
    let (input_rate, cache_read_rate) = match store.get_model_pricing(model) {
        Ok(Some(p)) => (p.input, p.cache_read),
        other => {
            if let Err(ref e) = other {
                eprintln!("[pricing] warn: DB lookup failed for {model}: {e}");
            }
            let p = get_pricing(model);
            (p.input, p.cache_read)
        }
    };
    #[allow(clippy::cast_precision_loss)]
    let tokens_m = cache_read_tokens as f64 / 1_000_000.0;
    tokens_m * (input_rate - cache_read_rate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore;
    use crate::domain::analytics::ModelPricing;
    use crate::ports::analytics_ports::{AnalyticsStore as _, PricingStore};
    use tempfile::tempdir;

    // ── Warning-1 & Warning-2 regression tests ──────────────────────────────

    /// Verify the file-private fallback struct is named `FallbackPricing`,
    /// not `ModelPricing` (which would shadow the domain type).
    #[test]
    fn test_fallback_pricing_struct_exists() {
        // This test compiles only if `FallbackPricing` is the name of the
        // private struct returned by `get_pricing()`.
        let p: FallbackPricing = get_pricing("claude-sonnet-4-5");
        assert!(p.input > 0.0);
    }

    /// A mock `PricingStore` that always returns `Err` so we can exercise the
    /// DB-error warning path in `estimate_cost_with_store` and
    /// `estimate_cache_savings_with_store`.
    struct AlwaysErrStore;
    impl PricingStore for AlwaysErrStore {
        fn upsert_model_pricing(&self, _: &ModelPricing) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("injected DB error"))
        }
        fn batch_upsert_model_pricing(&self, _: &[ModelPricing]) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("injected DB error"))
        }
        fn get_model_pricing(&self, _: &str) -> anyhow::Result<Option<ModelPricing>> {
            Err(anyhow::anyhow!("injected DB error"))
        }
        fn list_model_pricing(&self) -> anyhow::Result<Vec<ModelPricing>> {
            Err(anyhow::anyhow!("injected DB error"))
        }
    }

    /// When the store returns `Err`, both `_with_store` functions must still
    /// return the same result as the pure fallback (not panic / return 0).
    #[test]
    fn test_estimate_cost_with_store_falls_back_on_db_error() {
        let store = AlwaysErrStore;
        let expected =
            estimate_cost("claude-sonnet-4-5", 1_000_000, 1_000_000, 1_000_000, 1_000_000);
        let got = estimate_cost_with_store(
            &store,
            "claude-sonnet-4-5",
            1_000_000,
            1_000_000,
            1_000_000,
            1_000_000,
        );
        assert!((got - expected).abs() < 1e-9, "expected {expected}, got {got}");
    }

    #[test]
    fn test_estimate_cache_savings_with_store_falls_back_on_db_error() {
        let store = AlwaysErrStore;
        let expected = estimate_cache_savings("claude-sonnet-4-5", 1_000_000);
        let got = estimate_cache_savings_with_store(&store, "claude-sonnet-4-5", 1_000_000);
        assert!((got - expected).abs() < 1e-9, "expected {expected}, got {got}");
    }

    fn test_store() -> SqliteAnalyticsStore {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("test.db");
        let store = SqliteAnalyticsStore::open(db_path.to_str().expect("path")).expect("open");
        store.initialize_schema().expect("schema");
        store
    }

    #[test]
    fn test_estimate_cost_with_store_uses_db_pricing() {
        let store = test_store();
        store
            .upsert_model_pricing(&ModelPricing {
                model_id: "test-model".to_string(),
                input: 10.0,
                output: 30.0,
                cache_write: 12.5,
                cache_read: 1.0,
                source: "test".to_string(),
                synced_at: "2026-05-04T00:00:00Z".to_string(),
            })
            .expect("upsert");

        // 1_000_000 of each token type → cost = 10 + 30 + 12.5 + 1.0 = 53.5
        let cost = estimate_cost_with_store(
            &store,
            "test-model",
            1_000_000,
            1_000_000,
            1_000_000,
            1_000_000,
        );
        assert!((cost - 53.5).abs() < 1e-9, "expected 53.5, got {cost}");
    }

    #[test]
    fn test_estimate_cost_with_store_falls_back_when_missing() {
        let store = test_store();
        // "claude-sonnet-4-5" is not in DB → falls back to hardcoded
        let expected =
            estimate_cost("claude-sonnet-4-5", 1_000_000, 1_000_000, 1_000_000, 1_000_000);
        let got = estimate_cost_with_store(
            &store,
            "claude-sonnet-4-5",
            1_000_000,
            1_000_000,
            1_000_000,
            1_000_000,
        );
        assert!((got - expected).abs() < 1e-9, "expected {expected}, got {got}");
    }

    #[test]
    fn test_glm_51_fallback_pricing() {
        let cost = estimate_cost("glm-5.1", 1_000_000, 1_000_000, 1_000_000, 1_000_000);
        let expected = 1.4 + 4.4 + 0.0 + 0.26;
        assert!((cost - expected).abs() < 1e-9, "GLM-5.1 expected {expected}, got {cost}");
    }

    #[test]
    fn test_glm_5_turbo_fallback_pricing() {
        let cost = estimate_cost("glm-5-turbo", 1_000_000, 0, 0, 1_000_000);
        let expected = 1.2 + 0.0 + 0.0 + 0.24;
        assert!((cost - expected).abs() < 1e-9, "GLM-5-turbo expected {expected}, got {cost}");
    }

    #[test]
    fn test_generic_glm_fallback_pricing() {
        let cost = estimate_cost("glm-4.7", 1_000_000, 1_000_000, 0, 0);
        let expected = 1.0 + 3.2;
        assert!((cost - expected).abs() < 1e-9, "generic GLM expected {expected}, got {cost}");
    }
}
