/// Model pricing per 1M tokens (USD).
/// Source: Anthropic public pricing as of 2025.
struct ModelPricing {
    input: f64,
    output: f64,
    cache_write: f64,
    cache_read: f64,
}

fn get_pricing(model: &str) -> ModelPricing {
    let m = model.to_lowercase();
    if m.contains("opus") && (m.contains('4') || m.contains("opus-4")) {
        ModelPricing {
            input: 15.0,
            output: 75.0,
            cache_write: 18.75,
            cache_read: 1.5,
        }
    } else if m.contains("sonnet") && (m.contains('4') || m.contains("sonnet-4")) {
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_write: 3.75,
            cache_read: 0.3,
        }
    } else if m.contains("haiku") {
        ModelPricing {
            input: 0.80,
            output: 4.0,
            cache_write: 1.0,
            cache_read: 0.08,
        }
    } else if m.contains("sonnet") {
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_write: 3.75,
            cache_read: 0.3,
        }
    } else if m.contains("opus") {
        ModelPricing {
            input: 15.0,
            output: 75.0,
            cache_write: 18.75,
            cache_read: 1.5,
        }
    } else if m.contains("gpt-4o") {
        ModelPricing {
            input: 2.5,
            output: 10.0,
            cache_write: 2.5,
            cache_read: 1.25,
        }
    } else if m.contains("gpt-4") {
        ModelPricing {
            input: 30.0,
            output: 60.0,
            cache_write: 30.0,
            cache_read: 15.0,
        }
    } else if m.contains("deepseek") {
        ModelPricing {
            input: 0.27,
            output: 1.10,
            cache_write: 0.27,
            cache_read: 0.07,
        }
    } else {
        // Default conservative estimate
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_write: 3.75,
            cache_read: 0.3,
        }
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
