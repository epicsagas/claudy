use crate::domain::analytics::ModelPricing;
use crate::ports::analytics_ports::PricingStore;
use rusqlite::{OptionalExtension, params};

use super::SqliteAnalyticsStore;

impl PricingStore for SqliteAnalyticsStore {
    fn upsert_model_pricing(&self, pricing: &ModelPricing) -> anyhow::Result<()> {
        self.lock()?.execute(
            "INSERT INTO model_pricing (model_id, input, output, cache_write, cache_read, source, synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(model_id) DO UPDATE SET
               input       = excluded.input,
               output      = excluded.output,
               cache_write = excluded.cache_write,
               cache_read  = excluded.cache_read,
               source      = excluded.source,
               synced_at   = excluded.synced_at",
            params![
                pricing.model_id,
                pricing.input,
                pricing.output,
                pricing.cache_write,
                pricing.cache_read,
                pricing.source,
                pricing.synced_at,
            ],
        )?;
        Ok(())
    }

    fn batch_upsert_model_pricing(&self, pricings: &[ModelPricing]) -> anyhow::Result<()> {
        self.batch_upsert_model_pricing_impl(pricings)
    }

    fn get_model_pricing(&self, model_id: &str) -> anyhow::Result<Option<ModelPricing>> {
        let conn = self.lock()?;
        let result = conn
            .query_row(
                "SELECT model_id, input, output, cache_write, cache_read, source, synced_at
                 FROM model_pricing WHERE model_id = ?1",
                params![model_id],
                |row| {
                    Ok(ModelPricing {
                        model_id: row.get(0)?,
                        input: row.get(1)?,
                        output: row.get(2)?,
                        cache_write: row.get(3)?,
                        cache_read: row.get(4)?,
                        source: row.get(5)?,
                        synced_at: row.get(6)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    fn list_model_pricing(&self) -> anyhow::Result<Vec<ModelPricing>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT model_id, input, output, cache_write, cache_read, source, synced_at
             FROM model_pricing ORDER BY model_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ModelPricing {
                model_id: row.get(0)?,
                input: row.get(1)?,
                output: row.get(2)?,
                cache_write: row.get(3)?,
                cache_read: row.get(4)?,
                source: row.get(5)?,
                synced_at: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

pub(super) fn recalculate_costs_impl(store: &SqliteAnalyticsStore) -> anyhow::Result<u64> {
    let pricing_rows = store.list_model_pricing()?;
    let pricing_map: std::collections::HashMap<String, (f64, f64, f64, f64)> = pricing_rows
        .into_iter()
        .map(|p| (p.model_id, (p.input, p.output, p.cache_write, p.cache_read)))
        .collect();

    let conn = store.lock()?;

    let mut stmt = conn.prepare(
        "SELECT tu.id, tu.model, tu.input_tokens, tu.output_tokens,
                tu.cache_creation_input_tokens, tu.cache_read_input_tokens
         FROM token_usage tu",
    )?;
    let mut rows = stmt.query([])?;
    let mut updated: u64 = 0;

    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let model: String = row.get(1)?;
        let input: i64 = row.get(2)?;
        let output: i64 = row.get(3)?;
        let cache_creation: i64 = row.get(4)?;
        let cache_read: i64 = row.get(5)?;

        let new_cost = if let Some(&(inp, out, cw, cr)) = pricing_map.get(&model) {
            #[allow(clippy::cast_precision_loss)]
            let cost = (input as f64 / 1_000_000.0) * inp
                + (output as f64 / 1_000_000.0) * out
                + (cache_creation as f64 / 1_000_000.0) * cw
                + (cache_read as f64 / 1_000_000.0) * cr;
            cost
        } else {
            crate::adapters::analytics::analysis::cost::estimate_cost(
                &model,
                input,
                output,
                cache_creation,
                cache_read,
            )
        };

        conn.execute(
            "UPDATE token_usage SET estimated_cost_usd = ?1 WHERE id = ?2",
            params![new_cost, id],
        )?;
        updated += 1;
    }
    drop(rows);
    drop(stmt);

    // Recompute session totals from token_usage
    conn.execute(
        "UPDATE sessions SET total_cost_usd = (
            SELECT COALESCE(SUM(tu.estimated_cost_usd), 0)
            FROM token_usage tu
            JOIN turns t ON tu.turn_id = t.id
            WHERE t.session_id = sessions.id
        )",
        [],
    )?;

    Ok(updated)
}
