use std::collections::HashMap;

use chrono::Utc;

use crate::adapters::analytics::pricing::anthropic::AnthropicModelPrice;
use crate::adapters::analytics::pricing::fetcher::ModelsDevEntry;
use crate::domain::analytics::ModelPricing;

pub struct PricingMerger;

impl PricingMerger {
    /// Merge models.dev entries + Anthropic scraped prices → Vec<ModelPricing>.
    /// Only processes Anthropic Claude models (model id contains "claude").
    pub fn merge(
        models_dev: &[ModelsDevEntry],
        anthropic: &[AnthropicModelPrice],
    ) -> Vec<ModelPricing> {
        // Build lookup: normalized model name → AnthropicModelPrice
        let anthropic_lookup: HashMap<String, &AnthropicModelPrice> = anthropic
            .iter()
            .map(|p| (p.model_name.to_lowercase(), p))
            .collect();

        let synced_at = Utc::now().to_rfc3339();

        models_dev
            .iter()
            .filter(|e| {
                let id = e.id.to_lowercase();
                id.contains("claude")
                    || e.cost
                        .as_ref()
                        .is_some_and(|c| c.input.is_some_and(|v| v > 0.0))
            })
            .filter_map(|entry| {
                let anthropic_name = models_dev_id_to_anthropic_name(&entry.id);
                let lookup_key = anthropic_name.to_lowercase();

                if let Some(ap) = anthropic_lookup.get(&lookup_key) {
                    // Found in Anthropic table — use Anthropic prices
                    Some(ModelPricing {
                        model_id: entry.id.clone(),
                        input: ap.input,
                        output: ap.output,
                        cache_write: ap.cache_write,
                        cache_read: ap.cache_read,
                        source: "anthropic+models_dev".to_string(),
                        synced_at: synced_at.clone(),
                    })
                } else if let Some(cost) = &entry.cost {
                    // Fall back to models.dev if it has a finite, positive input cost
                    let input = match cost.input {
                        Some(v) if v.is_finite() && v > 0.0 => v,
                        _ => return None, // skip models with missing or non-finite input price
                    };
                    let output = match cost.output {
                        Some(v) if v.is_finite() && v > 0.0 => v,
                        _ => input * 5.0,
                    };
                    let is_claude = entry.id.to_lowercase().contains("claude");
                    let (cache_write, cache_read) = if is_claude {
                        // Claude models: derive cache rates from input ratio
                        (input * 1.25, input * 0.1)
                    } else {
                        // Non-Claude models: use actual cache prices from models.dev
                        let cw = cost
                            .cache_write
                            .filter(|v| v.is_finite() && *v >= 0.0)
                            .unwrap_or(0.0);
                        let cr = cost
                            .cache_read
                            .filter(|v| v.is_finite() && *v >= 0.0)
                            .unwrap_or(0.0);
                        (cw, cr)
                    };
                    Some(ModelPricing {
                        model_id: entry.id.clone(),
                        input,
                        output,
                        cache_write,
                        cache_read,
                        source: if is_claude {
                            "models_dev+ratio".to_string()
                        } else {
                            "models_dev".to_string()
                        },
                        synced_at: synced_at.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Convert a models.dev model id to the display name used in Anthropic's pricing table.
///
/// Examples:
/// - "claude-opus-4-7"        → "Claude Opus 4.7"
/// - "claude-sonnet-4-6"      → "Claude Sonnet 4.6"
/// - "anthropic/claude-haiku-4-5" → "Claude Haiku 4.5"
fn models_dev_id_to_anthropic_name(id: &str) -> String {
    // Strip a leading "provider/" prefix if present
    let bare = if let Some(pos) = id.rfind('/') {
        &id[pos + 1..]
    } else {
        id
    };

    // Split on '-' and build name
    let parts: Vec<&str> = bare.split('-').collect();

    // We expect at least: "claude", <family>, <major>, <minor>
    // e.g. ["claude", "sonnet", "4", "6"]
    // Reconstruct: capitalize each word; join last two numeric segments with '.'
    if parts.len() < 2 {
        // Fallback: just capitalize words
        return parts
            .iter()
            .map(|p| capitalize(p))
            .collect::<Vec<_>>()
            .join(" ");
    }

    // Detect trailing version numbers: collect trailing all-digit segments
    let mut version_parts: Vec<&str> = Vec::new();
    let mut word_end = parts.len();
    for part in parts.iter().rev() {
        if part.chars().all(|c| c.is_ascii_digit()) {
            version_parts.insert(0, part);
            word_end -= 1;
        } else {
            break;
        }
    }

    let words: Vec<String> = parts[..word_end].iter().map(|p| capitalize(p)).collect();

    if version_parts.is_empty() {
        words.join(" ")
    } else {
        format!("{} {}", words.join(" "), version_parts.join("."))
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::analytics::pricing::fetcher::ModelsDevCost;

    fn make_entry(id: &str, input: Option<f64>, output: Option<f64>) -> ModelsDevEntry {
        ModelsDevEntry {
            id: id.to_string(),
            name: None,
            cost: Some(ModelsDevCost {
                input,
                output,
                cache_read: None,
                cache_write: None,
            }),
        }
    }

    fn make_entry_no_cost(id: &str) -> ModelsDevEntry {
        ModelsDevEntry {
            id: id.to_string(),
            name: None,
            cost: None,
        }
    }

    fn make_anthropic_price(
        model_name: &str,
        input: f64,
        output: f64,
        cache_write: f64,
        cache_read: f64,
    ) -> AnthropicModelPrice {
        AnthropicModelPrice {
            model_name: model_name.to_string(),
            input,
            output,
            cache_write,
            cache_read,
        }
    }

    #[test]
    fn test_models_dev_id_to_anthropic_name() {
        assert_eq!(
            models_dev_id_to_anthropic_name("claude-opus-4-7"),
            "Claude Opus 4.7"
        );
        assert_eq!(
            models_dev_id_to_anthropic_name("claude-sonnet-4-6"),
            "Claude Sonnet 4.6"
        );
        assert_eq!(
            models_dev_id_to_anthropic_name("claude-haiku-4-5"),
            "Claude Haiku 4.5"
        );
        assert_eq!(
            models_dev_id_to_anthropic_name("anthropic/claude-sonnet-4-6"),
            "Claude Sonnet 4.6"
        );
    }

    #[test]
    fn test_merge_uses_anthropic_prices_when_matched() {
        let models_dev = vec![make_entry("claude-sonnet-4-6", Some(3.0), Some(15.0))];
        let anthropic = vec![make_anthropic_price(
            "Claude Sonnet 4.6",
            3.5,
            17.5,
            4.375,
            0.35,
        )];

        let result = PricingMerger::merge(&models_dev, &anthropic);

        assert_eq!(result.len(), 1);
        let p = &result[0];
        assert_eq!(p.model_id, "claude-sonnet-4-6");
        assert!((p.input - 3.5).abs() < 1e-9, "input should use Anthropic value");
        assert!((p.output - 17.5).abs() < 1e-9);
        assert!((p.cache_write - 4.375).abs() < 1e-9);
        assert!((p.cache_read - 0.35).abs() < 1e-9);
        assert_eq!(p.source, "anthropic+models_dev");
    }

    #[test]
    fn test_merge_falls_back_to_ratio() {
        let models_dev = vec![make_entry("claude-haiku-4-5", Some(0.80), Some(4.00))];
        let anthropic: Vec<AnthropicModelPrice> = vec![]; // no match

        let result = PricingMerger::merge(&models_dev, &anthropic);

        assert_eq!(result.len(), 1);
        let p = &result[0];
        assert_eq!(p.model_id, "claude-haiku-4-5");
        assert!((p.input - 0.80).abs() < 1e-9);
        assert!((p.output - 4.00).abs() < 1e-9);
        // cache_write = input * 1.25 = 1.0
        assert!((p.cache_write - 1.0).abs() < 1e-9);
        // cache_read = input * 0.1 = 0.08
        assert!((p.cache_read - 0.08).abs() < 1e-9);
        assert_eq!(p.source, "models_dev+ratio");
    }

    #[test]
    fn test_merge_skips_models_without_cost() {
        let models_dev = vec![make_entry_no_cost("claude-unknown-model")];
        let anthropic: Vec<AnthropicModelPrice> = vec![];

        let result = PricingMerger::merge(&models_dev, &anthropic);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_includes_non_claude_models_with_pricing() {
        let models_dev = vec![make_entry("gpt-4o", Some(5.0), Some(15.0))];
        let anthropic: Vec<AnthropicModelPrice> = vec![];

        let result = PricingMerger::merge(&models_dev, &anthropic);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].model_id, "gpt-4o");
        assert!((result[0].input - 5.0).abs() < 1e-9);
        assert_eq!(result[0].source, "models_dev");
    }

    #[test]
    fn test_merge_with_provider_prefix_in_id() {
        let models_dev = vec![make_entry("anthropic/claude-opus-4-7", Some(15.0), Some(75.0))];
        let anthropic = vec![make_anthropic_price(
            "Claude Opus 4.7",
            15.0,
            75.0,
            18.75,
            1.5,
        )];

        let result = PricingMerger::merge(&models_dev, &anthropic);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source, "anthropic+models_dev");
        assert!((result[0].input - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_merge_skips_models_with_non_finite_or_zero_input() {
        // input = 0.0 should be skipped (not positive)
        let models_dev_zero = vec![make_entry("claude-test-zero", Some(0.0), Some(15.0))];
        let anthropic: Vec<AnthropicModelPrice> = vec![];
        let result = PricingMerger::merge(&models_dev_zero, &anthropic);
        assert!(result.is_empty(), "zero input price must be skipped");

        // input = NaN should be skipped
        let models_dev_nan = vec![make_entry("claude-test-nan", Some(f64::NAN), Some(15.0))];
        let result = PricingMerger::merge(&models_dev_nan, &anthropic);
        assert!(result.is_empty(), "NaN input price must be skipped");

        // input = Infinity should be skipped
        let models_dev_inf = vec![make_entry("claude-test-inf", Some(f64::INFINITY), Some(15.0))];
        let result = PricingMerger::merge(&models_dev_inf, &anthropic);
        assert!(result.is_empty(), "Infinity input price must be skipped");
    }

    #[test]
    fn test_merge_ratio_fallback_when_output_missing() {
        let models_dev = vec![ModelsDevEntry {
            id: "claude-test-3".to_string(),
            name: None,
            cost: Some(ModelsDevCost {
                input: Some(2.0),
                output: None,
                cache_read: None,
                cache_write: None,
            }),
        }];
        let anthropic: Vec<AnthropicModelPrice> = vec![];

        let result = PricingMerger::merge(&models_dev, &anthropic);
        assert_eq!(result.len(), 1);
        // output defaults to input * 5.0 = 10.0
        assert!((result[0].output - 10.0).abs() < 1e-9);
        assert_eq!(result[0].source, "models_dev+ratio");
    }

    #[test]
    fn test_merge_non_claude_uses_actual_cache_prices() {
        let models_dev = vec![ModelsDevEntry {
            id: "glm-5.1".to_string(),
            name: None,
            cost: Some(ModelsDevCost {
                input: Some(1.4),
                output: Some(4.4),
                cache_read: Some(0.26),
                cache_write: Some(0.0),
            }),
        }];
        let anthropic: Vec<AnthropicModelPrice> = vec![];

        let result = PricingMerger::merge(&models_dev, &anthropic);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].model_id, "glm-5.1");
        assert!((result[0].input - 1.4).abs() < 1e-9);
        assert!((result[0].output - 4.4).abs() < 1e-9);
        // Non-Claude: use actual cache prices from models.dev, not ratio-derived
        assert!((result[0].cache_read - 0.26).abs() < 1e-9);
        assert!((result[0].cache_write - 0.0).abs() < 1e-9);
        assert_eq!(result[0].source, "models_dev");
    }

    #[test]
    fn test_merge_claude_still_uses_ratio_cache() {
        let models_dev = vec![ModelsDevEntry {
            id: "claude-test-3".to_string(),
            name: None,
            cost: Some(ModelsDevCost {
                input: Some(2.0),
                output: Some(10.0),
                cache_read: None,
                cache_write: None,
            }),
        }];
        let anthropic: Vec<AnthropicModelPrice> = vec![];

        let result = PricingMerger::merge(&models_dev, &anthropic);
        assert_eq!(result.len(), 1);
        // Claude models: ratio-derived cache rates
        assert!((result[0].cache_write - 2.5).abs() < 1e-9); // input * 1.25
        assert!((result[0].cache_read - 0.2).abs() < 1e-9); // input * 0.1
        assert_eq!(result[0].source, "models_dev+ratio");
    }
}
