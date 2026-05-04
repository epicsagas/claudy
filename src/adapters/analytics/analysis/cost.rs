/// Model pricing per 1M tokens (USD).
/// Source: https://platform.claude.com/docs/en/about-claude/pricing
struct ModelPricing {
    input: f64,
    output: f64,
    cache_write: f64,
    cache_read: f64,
}

fn get_pricing(model: &str) -> ModelPricing {
    let m = model.to_lowercase();

    // Extract version number from model ID (e.g. "claude-opus-4-7" → major=4, minor=7)
    // Pattern: <family>-<major>-<minor> or <family>-<major>
    let version: Option<(u32, u32)> = (|| {
        // Find family keyword position, then parse trailing digits
        let digits_part = m
            .trim_start_matches("claude-")
            .trim_start_matches("anthropic.")
            .trim_start_matches("anthropic/");
        // Split by '-' and collect numeric tail segments
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
    })();

    let (major, minor) = version.unwrap_or((0, 0));

    if m.contains("opus") {
        // Opus 4.5+ (4.5, 4.6, 4.7, ...): $5 input
        // Opus 4.0, 4.1: $15 input
        // Opus 3: $15 input
        if major >= 4 && minor >= 5 {
            ModelPricing { input: 5.0, output: 25.0, cache_write: 6.25, cache_read: 0.50 }
        } else {
            ModelPricing { input: 15.0, output: 75.0, cache_write: 18.75, cache_read: 1.50 }
        }
    } else if m.contains("sonnet") {
        // All Sonnet 3.x and 4.x: $3 input
        ModelPricing { input: 3.0, output: 15.0, cache_write: 3.75, cache_read: 0.30 }
    } else if m.contains("haiku") {
        if major >= 4 {
            // Haiku 4.5+: $1 input
            ModelPricing { input: 1.0, output: 5.0, cache_write: 1.25, cache_read: 0.10 }
        } else {
            // Haiku 3.5: $0.80, Haiku 3: $0.25
            let is_3_5 = minor >= 5;
            if is_3_5 {
                ModelPricing { input: 0.80, output: 4.0, cache_write: 1.0, cache_read: 0.08 }
            } else {
                ModelPricing { input: 0.25, output: 1.25, cache_write: 0.30, cache_read: 0.03 }
            }
        }
    } else if m.contains("gpt-4o") {
        ModelPricing { input: 2.5, output: 10.0, cache_write: 2.5, cache_read: 1.25 }
    } else if m.contains("gpt-4") {
        ModelPricing { input: 30.0, output: 60.0, cache_write: 30.0, cache_read: 15.0 }
    } else if m.contains("deepseek") {
        ModelPricing { input: 0.27, output: 1.10, cache_write: 0.27, cache_read: 0.07 }
    } else {
        // Conservative default: Sonnet pricing
        ModelPricing { input: 3.0, output: 15.0, cache_write: 3.75, cache_read: 0.30 }
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
