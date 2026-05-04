use ureq::config::Config;

const ANTHROPIC_PRICING_URL: &str =
    "https://platform.claude.com/docs/en/about-claude/pricing";

#[derive(Debug, Clone)]
pub struct AnthropicModelPrice {
    pub model_name: String,
    pub input: f64,
    pub output: f64,
    pub cache_write: f64,
    pub cache_read: f64,
}

pub struct AnthropicPricingScraper;

impl AnthropicPricingScraper {
    pub fn fetch() -> anyhow::Result<Vec<AnthropicModelPrice>> {
        let config = Config::builder()
            .timeout_global(Some(std::time::Duration::from_secs(15)))
            .build();
        let agent = ureq::Agent::new_with_config(config);
        let mut resp = agent.get(ANTHROPIC_PRICING_URL).call()?;
        let html = resp
            .body_mut()
            .with_config()
            .limit(4 * 1024 * 1024)
            .read_to_string()?;
        Ok(Self::parse(&html))
    }

    /// Parse pricing from HTML. Public for testing.
    pub fn parse(html: &str) -> Vec<AnthropicModelPrice> {
        // Find a <table> that contains "Base Input Tokens" in a header row
        let mut results = Vec::new();

        // Split the HTML into table blocks
        let tables = split_tables(html);
        for table in &tables {
            if !table.contains("Base Input Tokens") {
                continue;
            }
            if let Some(prices) = parse_pricing_table(table) {
                results.extend(prices);
            }
        }

        results
    }
}

/// Split HTML into individual table contents (between <table> ... </table>)
fn split_tables(html: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let mut tables = Vec::new();
    let mut search_from = 0;

    while let Some(start) = lower[search_from..].find("<table") {
        let abs_start = search_from + start;
        if let Some(end) = lower[abs_start..].find("</table>") {
            let abs_end = abs_start + end + "</table>".len();
            tables.push(html[abs_start..abs_end].to_string());
            search_from = abs_end;
        } else {
            break;
        }
    }

    tables
}

/// Parse rows from a pricing table. Returns None if structure unrecognized.
fn parse_pricing_table(table: &str) -> Option<Vec<AnthropicModelPrice>> {
    let rows = extract_rows(table);
    if rows.is_empty() {
        return None;
    }

    // Find the header row — the one that contains "Base Input Tokens"
    let header_row_idx = rows
        .iter()
        .position(|r| r.to_lowercase().contains("base input tokens"))?;

    let header_cells = extract_cells(&rows[header_row_idx]);

    // Detect column indices (partial match, case-insensitive)
    let col_model = 0usize; // model name is always first column
    let col_input = find_col_idx(&header_cells, "base input")?;
    let col_cache_write = find_col_idx(&header_cells, "cache write")?;
    let col_cache_read = find_col_idx(&header_cells, "cache hit")?;
    let col_output = find_col_idx(&header_cells, "output")?;

    let mut prices = Vec::new();

    for row in rows.iter().skip(header_row_idx + 1) {
        let cells = extract_cells(row);
        if cells.len()
            <= col_input
                .max(col_cache_write)
                .max(col_cache_read)
                .max(col_output)
        {
            continue;
        }

        let model_name = strip_html_tags(&cells[col_model]).trim().to_string();
        if model_name.is_empty() {
            continue;
        }

        let input = parse_price(&cells[col_input]);
        let cache_write = parse_price(&cells[col_cache_write]);
        let cache_read = parse_price(&cells[col_cache_read]);
        let output = parse_price(&cells[col_output]);

        // Sanity check
        if !sanity_check(input, output, cache_read) {
            continue;
        }

        prices.push(AnthropicModelPrice {
            model_name,
            input,
            output,
            cache_write,
            cache_read,
        });
    }

    Some(prices)
}

fn sanity_check(input: f64, output: f64, cache_read: f64) -> bool {
    input > 0.0 && input <= 100.0 && cache_read < input && output > input
}

fn find_col_idx(headers: &[String], needle: &str) -> Option<usize> {
    let needle_lower = needle.to_lowercase();
    headers.iter().position(|h| {
        let text = strip_html_tags(h);
        text.to_lowercase().contains(&needle_lower)
    })
}

/// Extract <tr>...</tr> blocks from table HTML.
fn extract_rows(html: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let mut rows = Vec::new();
    let mut pos = 0;

    while let Some(start) = lower[pos..].find("<tr") {
        let abs_start = pos + start;
        if let Some(end) = lower[abs_start..].find("</tr>") {
            let abs_end = abs_start + end + "</tr>".len();
            rows.push(html[abs_start..abs_end].to_string());
            pos = abs_end;
        } else {
            break;
        }
    }

    rows
}

/// Extract <th> or <td> cell contents from a row.
fn extract_cells(row: &str) -> Vec<String> {
    let lower = row.to_ascii_lowercase();
    let mut cells = Vec::new();
    let mut pos = 0;

    while pos < lower.len() {
        // Find next <th or <td
        let th_pos = lower[pos..].find("<th").map(|i| pos + i);
        let td_pos = lower[pos..].find("<td").map(|i| pos + i);

        let (tag_start, closing) = match (th_pos, td_pos) {
            (Some(a), Some(b)) => {
                if a <= b {
                    (a, "</th>")
                } else {
                    (b, "</td>")
                }
            }
            (Some(a), None) => (a, "</th>"),
            (None, Some(b)) => (b, "</td>"),
            (None, None) => break,
        };

        // Skip past the opening tag >
        if let Some(gt) = lower[tag_start..].find('>') {
            let content_start = tag_start + gt + 1;
            if let Some(end) = lower[content_start..].find(closing) {
                let content_end = content_start + end;
                cells.push(row[content_start..content_end].to_string());
                pos = content_end + closing.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    cells
}

/// Strip all HTML tags from a string and decode common entities.
fn strip_html_tags(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&#36;", "$")
        .replace("&dollar;", "$")
}

/// Parse a price string like "$3.00/MTok" or "$3.00 - $3.50/MTok" → 3.0
///
/// Extracts the first contiguous run of ASCII digits and '.' characters,
/// skipping any leading non-digit chars (e.g. '$'). This correctly handles
/// range strings — only the first price token is returned.
fn parse_price(s: &str) -> f64 {
    let text = strip_html_tags(s);
    let first: String = text
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    first.parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_pricing_html(rows: &[(&str, &str, &str, &str, &str)]) -> String {
        let header = r#"<tr>
            <th>Model</th>
            <th>Base Input Tokens</th>
            <th>5m Cache Writes</th>
            <th>Cache Hits &amp; Refreshes</th>
            <th>Output Tokens</th>
        </tr>"#;

        let data_rows: String = rows
            .iter()
            .map(|(name, input, cache_write, cache_read, output)| {
                format!(
                    "<tr><td>{name}</td><td>{input}</td><td>{cache_write}</td><td>{cache_read}</td><td>{output}</td></tr>",
                )
            })
            .collect();

        format!("<table>{header}{data_rows}</table>")
    }

    #[test]
    fn test_parse_pricing_table() {
        let html = minimal_pricing_html(&[(
            "Claude Sonnet 4.6",
            "$3.00/MTok",
            "$3.75/MTok",
            "$0.30/MTok",
            "$15.00/MTok",
        )]);

        let prices = AnthropicPricingScraper::parse(&html);
        assert_eq!(prices.len(), 1);
        let p = &prices[0];
        assert_eq!(p.model_name, "Claude Sonnet 4.6");
        assert!((p.input - 3.0).abs() < 1e-9);
        assert!((p.output - 15.0).abs() < 1e-9);
        assert!((p.cache_write - 3.75).abs() < 1e-9);
        assert!((p.cache_read - 0.30).abs() < 1e-9);
    }

    #[test]
    fn test_parse_multiple_models() {
        let html = minimal_pricing_html(&[
            (
                "Claude Opus 4.7",
                "$15.00/MTok",
                "$18.75/MTok",
                "$1.50/MTok",
                "$75.00/MTok",
            ),
            (
                "Claude Haiku 4.5",
                "$0.80/MTok",
                "$1.00/MTok",
                "$0.08/MTok",
                "$4.00/MTok",
            ),
        ]);

        let prices = AnthropicPricingScraper::parse(&html);
        assert_eq!(prices.len(), 2);
        assert_eq!(prices[0].model_name, "Claude Opus 4.7");
        assert_eq!(prices[1].model_name, "Claude Haiku 4.5");
    }

    #[test]
    fn test_sanity_check_rejects_zero_input() {
        // input = 0 should be rejected
        let html = minimal_pricing_html(&[(
            "Bad Model",
            "$0.00/MTok",
            "$0.00/MTok",
            "$0.00/MTok",
            "$5.00/MTok",
        )]);

        let prices = AnthropicPricingScraper::parse(&html);
        assert!(prices.is_empty());
    }

    #[test]
    fn test_sanity_check_rejects_cache_read_ge_input() {
        // cache_read >= input should be rejected
        let html = minimal_pricing_html(&[(
            "Bad Model",
            "$3.00/MTok",
            "$3.75/MTok",
            "$5.00/MTok", // cache_read > input — invalid
            "$15.00/MTok",
        )]);

        let prices = AnthropicPricingScraper::parse(&html);
        assert!(prices.is_empty());
    }

    #[test]
    fn test_sanity_check_rejects_output_le_input() {
        // output <= input should be rejected
        let html = minimal_pricing_html(&[(
            "Bad Model",
            "$3.00/MTok",
            "$3.75/MTok",
            "$0.30/MTok",
            "$2.00/MTok", // output < input — invalid
        )]);

        let prices = AnthropicPricingScraper::parse(&html);
        assert!(prices.is_empty());
    }

    #[test]
    fn test_table_without_pricing_header_ignored() {
        let html = r#"<table>
            <tr><th>Model</th><th>Context</th></tr>
            <tr><td>Claude Foo</td><td>200k</td></tr>
        </table>"#;

        let prices = AnthropicPricingScraper::parse(html);
        assert!(prices.is_empty());
    }

    #[test]
    fn test_parse_price_strips_formatting() {
        assert!((parse_price("$3.00/MTok") - 3.0).abs() < 1e-9);
        assert!((parse_price("  $15.00 /MTok ") - 15.0).abs() < 1e-9);
        assert!((parse_price("0.30") - 0.30).abs() < 1e-9);
    }

    /// Range-formatted strings like "$3.00 - $3.50/MTok" must never produce a
    /// negative result. Without the fix, the `-` in the filter set yields the
    /// cleaned string "3.00-3.50" which f64::parse rejects → 0.0 (acceptable),
    /// but sub-strings like "-.50" could produce negative values. After the fix
    /// only digits and '.' pass through, guaranteeing non-negative output.
    #[test]
    fn test_parse_price_range_string_no_negative() {
        let result = parse_price("$3.00 - $3.50/MTok");
        assert!(
            result >= 0.0,
            "parse_price must never return a negative number, got {result}"
        );
        // Single-price form must still parse correctly.
        assert!((parse_price("$3.00/MTok") - 3.0).abs() < 1e-9);
    }

    /// Range strings must parse to the first (lower) price value, not 0.0
    /// or a concatenated garbage number like 3.003.50.
    #[test]
    fn test_parse_price_range_string_first_value() {
        // "$3.00 - $3.50/MTok" should yield 3.0, not 0.0 or 3.003
        let result = parse_price("$3.00 - $3.50/MTok");
        assert!(
            (result - 3.0).abs() < 1e-9,
            "expected 3.0 for range string, got {result}"
        );
    }

    /// Confirm split_tables is safe when HTML contains non-ASCII characters
    /// before the table tag (validates the ascii_lowercase byte-offset fix).
    #[test]
    fn test_split_tables_non_ascii_html_no_panic() {
        // \u{00e9} is 2 bytes in UTF-8; to_lowercase keeps it 2 bytes but
        // to_ascii_lowercase keeps it 1 ASCII byte — this mismatch caused panics.
        let html = "<p>\u{00e9}</p><table><tr><th>X</th></tr></table>";
        let tables = split_tables(html);
        assert_eq!(tables.len(), 1);
        assert!(tables[0].to_ascii_lowercase().starts_with("<table"));
    }
}
