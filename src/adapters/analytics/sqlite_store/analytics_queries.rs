use crate::domain::analytics::*;
use rusqlite::{Connection, params};

use super::SqliteAnalyticsStore;

pub(super) fn aggregate_token_trends_impl(
    store: &SqliteAnalyticsStore,
    days: u32,
    project_id: Option<i64>,
) -> anyhow::Result<Vec<TokenTrendPoint>> {
    let conn = store.lock()?;
    collect_rows_project(
        &conn,
        "SELECT date(sessions.started_at) as d, token_usage.model, SUM(input_tokens), SUM(output_tokens), SUM(estimated_cost_usd), COUNT(DISTINCT sessions.id)
         FROM token_usage
         JOIN turns ON token_usage.turn_id = turns.id
         JOIN sessions ON turns.session_id = sessions.id
         WHERE sessions.started_at > date('now', '-' || ?1 || ' days') AND sessions.project_id = ?2
         GROUP BY d, token_usage.model ORDER BY d ASC",
        "SELECT date(sessions.started_at) as d, token_usage.model, SUM(input_tokens), SUM(output_tokens), SUM(estimated_cost_usd), COUNT(DISTINCT sessions.id)
         FROM token_usage
         JOIN turns ON token_usage.turn_id = turns.id
         JOIN sessions ON turns.session_id = sessions.id
         WHERE sessions.started_at > date('now', '-' || ?1 || ' days')
         GROUP BY d, token_usage.model ORDER BY d ASC",
        days,
        project_id,
        |row| Ok(TokenTrendPoint {
            date: row.get(0)?,
            model: row.get(1)?,
            input_tokens: row.get(2)?,
            output_tokens: row.get(3)?,
            total_cost_usd: row.get(4)?,
            session_count: row.get(5)?,
        }),
    )
}

pub(super) fn aggregate_tool_distribution_impl(
    store: &SqliteAnalyticsStore,
    days: Option<u32>,
    project_id: Option<i64>,
) -> anyhow::Result<Vec<ToolDistribution>> {
    let conn = store.lock()?;
    let query = match (days, project_id) {
        (Some(_), Some(_)) =>
            "SELECT tool_name, COUNT(*), SUM(CASE WHEN is_error THEN 1 ELSE 0 END), AVG(tool_calls.duration_ms)
             FROM tool_calls
             JOIN turns ON tool_calls.turn_id = turns.id
             JOIN sessions ON turns.session_id = sessions.id
             WHERE sessions.started_at > date('now', '-' || ?1 || ' days') AND sessions.project_id = ?2
             GROUP BY tool_name",
        (Some(_), None) =>
            "SELECT tool_name, COUNT(*), SUM(CASE WHEN is_error THEN 1 ELSE 0 END), AVG(tool_calls.duration_ms)
             FROM tool_calls
             JOIN turns ON tool_calls.turn_id = turns.id
             JOIN sessions ON turns.session_id = sessions.id
             WHERE sessions.started_at > date('now', '-' || ?1 || ' days')
             GROUP BY tool_name",
        (None, Some(_)) =>
            "SELECT tool_name, COUNT(*), SUM(CASE WHEN is_error THEN 1 ELSE 0 END), AVG(tool_calls.duration_ms)
             FROM tool_calls
             JOIN turns ON tool_calls.turn_id = turns.id
             JOIN sessions ON turns.session_id = sessions.id
             WHERE sessions.project_id = ?1
             GROUP BY tool_name",
        (None, None) =>
            "SELECT tool_name, COUNT(*), SUM(CASE WHEN is_error THEN 1 ELSE 0 END), AVG(tool_calls.duration_ms)
             FROM tool_calls
             GROUP BY tool_name",
    };
    let mut stmt = conn.prepare(query)?;
    let mut rows = match (days, project_id) {
        (Some(d), Some(pid)) => stmt.query(params![d, pid])?,
        (Some(d), None) => stmt.query(params![d])?,
        (None, Some(pid)) => stmt.query(params![pid])?,
        (None, None) => stmt.query([])?,
    };

    let mut counts = Vec::new();
    let mut total_calls: i64 = 0;
    while let Some(row) = rows.next()? {
        let count: i64 = row.get(1)?;
        total_calls += count;
        counts.push((
            row.get::<_, String>(0)?,
            count,
            row.get::<_, i64>(2)?,
            row.get::<_, Option<f64>>(3)?,
        ));
    }

    Ok(counts
        .into_iter()
        .map(|(name, count, errors, avg_dur)| ToolDistribution {
            tool_name: name,
            call_count: count,
            error_count: errors,
            avg_duration_ms: avg_dur,
            percentage: if total_calls > 0 {
                (count as f64 / total_calls as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect())
}

pub(super) fn aggregate_dashboard_stats_impl(
    store: &SqliteAnalyticsStore,
    days: u32,
    project_id: Option<i64>,
) -> anyhow::Result<DashboardStats> {
    let conn = store.lock()?;

    let (total_sessions, total_cost_usd, total_turns, total_duration_ms): (i64, f64, i64, i64) =
        query_row_project(
            &conn,
            "SELECT COUNT(*), COALESCE(SUM(total_cost_usd), 0), COALESCE(SUM(total_turns), 0), COALESCE(SUM(total_duration_ms), 0)
             FROM sessions WHERE started_at > date('now', '-' || ?1 || ' days') AND project_id = ?2",
            "SELECT COUNT(*), COALESCE(SUM(total_cost_usd), 0), COALESCE(SUM(total_turns), 0), COALESCE(SUM(total_duration_ms), 0)
             FROM sessions WHERE started_at > date('now', '-' || ?1 || ' days')",
            days, project_id, |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

    let (total_input, total_output, cache_read, cache_creation): (i64, i64, i64, i64) =
        query_row_project(
            &conn,
            "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), COALESCE(SUM(cache_read_input_tokens), 0), COALESCE(SUM(cache_creation_input_tokens), 0)
             FROM token_usage
             JOIN turns ON token_usage.turn_id = turns.id
             JOIN sessions ON turns.session_id = sessions.id
             WHERE sessions.started_at > date('now', '-' || ?1 || ' days') AND sessions.project_id = ?2",
            "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), COALESCE(SUM(cache_read_input_tokens), 0), COALESCE(SUM(cache_creation_input_tokens), 0)
             FROM token_usage
             JOIN turns ON token_usage.turn_id = turns.id
             JOIN sessions ON turns.session_id = sessions.id
             WHERE sessions.started_at > date('now', '-' || ?1 || ' days')",
            days, project_id, |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

    let most_used_model: Option<String> = query_optional_project(
        &conn,
        "SELECT turns.model FROM turns
         JOIN sessions ON turns.session_id = sessions.id
         WHERE sessions.started_at > date('now', '-' || ?1 || ' days')
           AND sessions.project_id = ?2 AND turns.model IS NOT NULL
         GROUP BY turns.model ORDER BY COUNT(*) DESC LIMIT 1",
        "SELECT turns.model FROM turns
         JOIN sessions ON turns.session_id = sessions.id
         WHERE sessions.started_at > date('now', '-' || ?1 || ' days')
           AND turns.model IS NOT NULL
         GROUP BY turns.model ORDER BY COUNT(*) DESC LIMIT 1",
        days,
        project_id,
        |row| row.get(0),
    )?
    .flatten();

    let avg_tokens_per_session = if total_sessions > 0 {
        (total_input + total_output) as f64 / total_sessions as f64
    } else {
        0.0
    };
    let cache_hit_ratio = if (cache_read + cache_creation) > 0 {
        cache_read as f64 / (cache_read + cache_creation) as f64
    } else {
        0.0
    };

    Ok(DashboardStats {
        total_sessions,
        total_cost_usd,
        total_turns,
        total_duration_ms,
        avg_tokens_per_session,
        cache_hit_ratio,
        most_used_model,
        alert_count: 0,
    })
}

pub(super) fn aggregate_cost_metrics_impl(
    store: &SqliteAnalyticsStore,
    days: u32,
    project_id: Option<i64>,
) -> anyhow::Result<CostMetrics> {
    let conn = store.lock()?;

    let (total_cost, total_sessions, total_turns): (f64, i64, i64) = query_row_project(
        &conn,
        "SELECT COALESCE(SUM(total_cost_usd), 0), COUNT(*), COALESCE(SUM(total_turns), 0)
         FROM sessions WHERE started_at > date('now', '-' || ?1 || ' days') AND project_id = ?2",
        "SELECT COALESCE(SUM(total_cost_usd), 0), COUNT(*), COALESCE(SUM(total_turns), 0)
         FROM sessions WHERE started_at > date('now', '-' || ?1 || ' days')",
        days,
        project_id,
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let weekly_total: f64 = query_row_project_only(
        &conn,
        "SELECT COALESCE(SUM(total_cost_usd), 0) FROM sessions WHERE started_at > date('now', '-7 days') AND project_id = ?1",
        "SELECT COALESCE(SUM(total_cost_usd), 0) FROM sessions WHERE started_at > date('now', '-7 days')",
        project_id,
        |row| row.get(0),
    )?;

    let pricing_map = load_pricing_map(&conn)?;
    let (by_model, cache_savings_usd) =
        query_model_breakdown(&conn, days, project_id, total_cost, &pricing_map)?;

    let most_expensive_session = query_most_expensive_session(&conn, days, project_id)?;

    let avg_cost_per_session = if total_sessions > 0 {
        total_cost / total_sessions as f64
    } else {
        0.0
    };
    let avg_cost_per_turn = if total_turns > 0 {
        total_cost / total_turns as f64
    } else {
        0.0
    };
    let weekly_avg_cost = weekly_total / 7.0;

    let estimated_cost: f64 = by_model
        .iter()
        .filter(|m| m.pricing_source.as_deref() == Some("hardcoded"))
        .map(|m| m.total_cost_usd)
        .sum();
    let estimated_cost_portion = if total_cost > 0.0 {
        estimated_cost / total_cost
    } else {
        0.0
    };

    Ok(CostMetrics {
        total_cost_usd: total_cost,
        avg_cost_per_session,
        avg_cost_per_turn,
        weekly_avg_cost,
        total_sessions,
        total_turns,
        cache_savings_usd,
        estimated_cost_portion,
        by_model,
        most_expensive_session,
    })
}

fn query_row_project<T>(
    conn: &Connection,
    sql_with: &str,
    sql_without: &str,
    days: u32,
    project_id: Option<i64>,
    f: impl FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> anyhow::Result<T> {
    let sql = if project_id.is_some() {
        sql_with
    } else {
        sql_without
    };
    Ok(if let Some(pid) = project_id {
        conn.query_row(sql, params![days, pid], f)?
    } else {
        conn.query_row(sql, params![days], f)?
    })
}

fn query_row_project_only<T>(
    conn: &Connection,
    sql_with: &str,
    sql_without: &str,
    project_id: Option<i64>,
    f: impl FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> anyhow::Result<T> {
    let sql = if project_id.is_some() {
        sql_with
    } else {
        sql_without
    };
    Ok(if let Some(pid) = project_id {
        conn.query_row(sql, params![pid], f)?
    } else {
        conn.query_row(sql, [], f)?
    })
}

fn collect_rows_project<T>(
    conn: &Connection,
    sql_with: &str,
    sql_without: &str,
    days: u32,
    project_id: Option<i64>,
    map_row: impl Fn(&rusqlite::Row<'_>) -> anyhow::Result<T>,
) -> anyhow::Result<Vec<T>> {
    let sql = if project_id.is_some() {
        sql_with
    } else {
        sql_without
    };
    let mut stmt = conn.prepare(sql)?;
    let mut rows = if let Some(pid) = project_id {
        stmt.query(params![days, pid])?
    } else {
        stmt.query(params![days])?
    };
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(map_row(row)?);
    }
    Ok(result)
}

fn query_optional_project<T>(
    conn: &Connection,
    sql_with: &str,
    sql_without: &str,
    days: u32,
    project_id: Option<i64>,
    map_row: impl Fn(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> anyhow::Result<Option<T>> {
    let sql = if project_id.is_some() {
        sql_with
    } else {
        sql_without
    };
    let mut stmt = conn.prepare(sql)?;
    let mut rows = if let Some(pid) = project_id {
        stmt.query(params![days, pid])?
    } else {
        stmt.query(params![days])?
    };
    Ok(match rows.next()? {
        Some(row) => Some(map_row(row)?),
        None => None,
    })
}

fn load_pricing_map(
    conn: &Connection,
) -> anyhow::Result<std::collections::HashMap<String, (f64, f64)>> {
    let mut ps = conn.prepare("SELECT model_id, input, cache_read FROM model_pricing")?;
    let iter = ps.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;
    let mut map = std::collections::HashMap::new();
    for r in iter {
        let (id, inp, cr) = r?;
        map.insert(id, (inp, cr));
    }
    Ok(map)
}

fn query_model_breakdown(
    conn: &Connection,
    days: u32,
    project_id: Option<i64>,
    total_cost: f64,
    pricing_map: &std::collections::HashMap<String, (f64, f64)>,
) -> anyhow::Result<(Vec<ModelCostBreakdown>, f64)> {
    let sql_with = "SELECT tu.model,
            COALESCE(SUM(tu.estimated_cost_usd), 0),
            COALESCE(SUM(tu.input_tokens), 0),
            COALESCE(SUM(tu.output_tokens), 0),
            COALESCE(SUM(tu.cache_read_input_tokens), 0),
            COUNT(DISTINCT turns.session_id)
     FROM token_usage tu
     JOIN turns ON tu.turn_id = turns.id
     JOIN sessions s ON turns.session_id = s.id
     WHERE s.started_at > date('now', '-' || ?1 || ' days') AND s.project_id = ?2
     GROUP BY tu.model";
    let sql_without = "SELECT tu.model,
            COALESCE(SUM(tu.estimated_cost_usd), 0),
            COALESCE(SUM(tu.input_tokens), 0),
            COALESCE(SUM(tu.output_tokens), 0),
            COALESCE(SUM(tu.cache_read_input_tokens), 0),
            COUNT(DISTINCT turns.session_id)
     FROM token_usage tu
     JOIN turns ON tu.turn_id = turns.id
     JOIN sessions s ON turns.session_id = s.id
     WHERE s.started_at > date('now', '-' || ?1 || ' days')
     GROUP BY tu.model";

    let mut by_model = Vec::new();
    let mut cache_savings_usd: f64 = 0.0;

    let raw: Vec<(String, f64, i64, i64, i64, i64)> =
        collect_rows_project(conn, sql_with, sql_without, days, project_id, |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;

    for (model, cost, input, output, cache_read, session_count) in raw {
        let has_db_pricing = pricing_map.contains_key(&model);
        cache_savings_usd += if let Some(&(inp_rate, cr_rate)) = pricing_map.get(&model) {
            if inp_rate > cr_rate {
                #[allow(clippy::cast_precision_loss)]
                let tokens_m = cache_read as f64 / 1_000_000.0;
                tokens_m * (inp_rate - cr_rate)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let percentage = if total_cost > 0.0 {
            (cost / total_cost) * 100.0
        } else {
            0.0
        };
        let avg_cost_per_session = if session_count > 0 {
            cost / session_count as f64
        } else {
            0.0
        };

        by_model.push(ModelCostBreakdown {
            model,
            total_cost_usd: cost,
            total_input_tokens: input,
            total_output_tokens: output,
            total_cache_read_tokens: cache_read,
            session_count,
            avg_cost_per_session,
            percentage,
            pricing_source: Some(if has_db_pricing { "db" } else { "hardcoded" }.to_string()),
        });
    }

    Ok((by_model, cache_savings_usd))
}

fn query_most_expensive_session(
    conn: &Connection,
    days: u32,
    project_id: Option<i64>,
) -> anyhow::Result<Option<SessionCostHighlight>> {
    query_optional_project(
        conn,
        "SELECT s.session_uuid, p.display_name, s.total_cost_usd, s.total_turns, s.model, s.started_at
         FROM sessions s JOIN projects p ON s.project_id = p.id
         WHERE s.started_at > date('now', '-' || ?1 || ' days') AND s.project_id = ?2
         ORDER BY s.total_cost_usd DESC LIMIT 1",
        "SELECT s.session_uuid, p.display_name, s.total_cost_usd, s.total_turns, s.model, s.started_at
         FROM sessions s JOIN projects p ON s.project_id = p.id
         WHERE s.started_at > date('now', '-' || ?1 || ' days')
         ORDER BY s.total_cost_usd DESC LIMIT 1",
        days, project_id, |row| Ok(SessionCostHighlight {
            session_uuid: row.get(0)?,
            project_name: row.get(1)?,
            cost_usd: row.get(2)?,
            turns: row.get(3)?,
            model: row.get(4)?,
            started_at: row.get(5)?,
        }),
    )
}

// ---------------------------------------------------------------------------
// Analysis queries for the four previously-stubbed aggregations.
// ---------------------------------------------------------------------------

/// Per-session prompt efficiency: token/tool economics + cache utilization.
pub(super) fn aggregate_prompt_efficiency_impl(
    store: &SqliteAnalyticsStore,
    limit: u32,
) -> anyhow::Result<Vec<PromptEfficiency>> {
    let conn = store.lock()?;
    let mut stmt = conn.prepare(
        "SELECT s.session_uuid, p.display_name,
                COALESCE(s.total_turns, 0),
                COALESCE(SUM(tu.input_tokens), 0),
                COALESCE(SUM(tu.output_tokens), 0),
                (SELECT COUNT(*) FROM tool_calls tc JOIN turns t2 ON tc.turn_id = t2.id
                 WHERE t2.session_id = s.id),
                COALESCE(SUM(tu.cache_read_input_tokens), 0),
                COALESCE(SUM(tu.estimated_cost_usd), 0.0)
         FROM sessions s
         JOIN projects p ON s.project_id = p.id
         LEFT JOIN turns t ON t.session_id = s.id
         LEFT JOIN token_usage tu ON tu.turn_id = t.id
         WHERE s.total_turns > 0
         GROUP BY s.id
         ORDER BY COALESCE(SUM(tu.estimated_cost_usd), 0.0) DESC
         LIMIT ?1",
    )?;
    let mut out = Vec::new();
    let rows = stmt.query_map(params![limit], |row| {
        let input_tokens: i64 = row.get(3)?;
        let output_tokens: i64 = row.get(4)?;
        let tool_calls: i64 = row.get(5)?;
        let cache_read: i64 = row.get(6)?;
        let total_tok = (input_tokens + output_tokens).max(1) as f64;
        let cache_denom = (input_tokens + cache_read).max(1) as f64;
        Ok(PromptEfficiency {
            session_uuid: row.get(0)?,
            project_name: row.get(1)?,
            total_turns: row.get(2)?,
            total_input_tokens: input_tokens,
            total_output_tokens: output_tokens,
            tool_call_count: tool_calls,
            tool_overhead_ratio: tool_calls as f64 / total_tok,
            cache_hit_ratio: cache_read as f64 / cache_denom,
            cost_usd: row.get(7)?,
        })
    })?;
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Most frequent adjacent tool-name bigrams; flags known anti-patterns.
pub(super) fn detect_tool_patterns_impl(
    store: &SqliteAnalyticsStore,
    min_frequency: u32,
) -> anyhow::Result<Vec<ToolPattern>> {
    let conn = store.lock()?;
    // Build per-turn ordered tool-name sequences, then count adjacent bigrams.
    let mut stmt = conn.prepare(
        "SELECT turn_id, tool_name FROM tool_calls ORDER BY turn_id, id",
    )?;
    let mut cur_turn: Option<i64> = None;
    let mut prev_name: Option<String> = None;
    let mut counts: std::collections::HashMap<(String, String), (i64, i64)> =
        std::collections::HashMap::new(); // (freq, errors)
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    // error lookup per turn
    let mut err_stmt = conn.prepare(
        "SELECT turn_id, SUM(is_error) FROM tool_calls GROUP BY turn_id",
    )?;
    let mut turn_errors: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let err_rows = err_stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
    for r in err_rows {
        let (t, e) = r?;
        turn_errors.insert(t, e);
    }
    drop(err_stmt);
    for r in rows {
        let (turn, name) = r?;
        if Some(turn) != cur_turn {
            cur_turn = Some(turn);
            prev_name = None;
        }
        if let Some(prev) = prev_name.take() {
            let e = *turn_errors.get(&turn).unwrap_or(&0);
            let entry = counts.entry((prev, name.clone())).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += e;
        }
        prev_name = Some(name);
    }
    let mut out: Vec<ToolPattern> = counts
        .into_iter()
        .filter(|(_, v)| v.0 >= min_frequency as i64)
        .map(|((a, b), (freq, errs))| {
            // Anti-pattern heuristic: repeated identical tool, or edit-read churn.
            let anti = a == b || (a == "Edit" && b == "Read") || (a == "Read" && b == "Edit");
            ToolPattern {
                sequence: vec![a, b],
                frequency: freq,
                error_rate: if freq > 0 { errs as f64 / freq as f64 } else { 0.0 },
                is_anti_pattern: anti,
            }
        })
        .collect();
    out.sort_by(|x, y| y.frequency.cmp(&x.frequency));
    Ok(out)
}

/// Per-model averages over sessions that used it.
pub(super) fn compare_model_performance_impl(
    store: &SqliteAnalyticsStore,
) -> anyhow::Result<Vec<ModelPerformance>> {
    let conn = store.lock()?;
    let mut stmt = conn.prepare(
        "SELECT tu.model,
                AVG(t.duration_ms),
                AVG(tu.input_tokens),
                AVG(tu.output_tokens),
                AVG(s.total_cost_usd),
                COUNT(DISTINCT s.id),
                AVG(tu.cache_read_input_tokens)
         FROM token_usage tu
         JOIN turns t ON tu.turn_id = t.id
         JOIN sessions s ON t.session_id = s.id
         WHERE tu.model IS NOT NULL AND tu.model <> '<synthetic>'
         GROUP BY tu.model
         ORDER BY COUNT(DISTINCT s.id) DESC",
    )?;
    let mut out = Vec::new();
    let rows = stmt.query_map([], |row| {
        let avg_in: f64 = row.get::<_, Option<f64>>(2)?.unwrap_or(0.0);
        let cache: f64 = row.get::<_, Option<f64>>(6)?.unwrap_or(0.0);
        let denom = avg_in + cache;
        Ok(ModelPerformance {
            model: row.get(0)?,
            avg_duration_ms: row.get::<_, Option<f64>>(1)?.unwrap_or(0.0),
            avg_input_tokens: avg_in,
            avg_output_tokens: row.get::<_, Option<f64>>(3)?.unwrap_or(0.0),
            avg_cost_per_session: row.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
            total_sessions: row.get::<_, i64>(5)?,
            cache_hit_ratio: if denom > 0.0 { cache / denom } else { 0.0 },
        })
    })?;
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Top sessions by cost/duration for cross-session comparison.
pub(super) fn aggregate_session_comparisons_impl(
    store: &SqliteAnalyticsStore,
    limit: u32,
) -> anyhow::Result<Vec<SessionComparison>> {
    let conn = store.lock()?;
    let mut stmt = conn.prepare(
        "SELECT s.session_uuid, p.display_name, s.started_at,
                COALESCE(s.total_duration_ms, 0), COALESCE(s.total_cost_usd, 0.0),
                COALESCE(s.total_turns, 0), s.model
         FROM sessions s JOIN projects p ON s.project_id = p.id
         WHERE s.total_cost_usd IS NOT NULL
         ORDER BY s.total_cost_usd DESC LIMIT ?1",
    )?;
    let mut out = Vec::new();
    let rows = stmt.query_map(params![limit], |row| {
        Ok(SessionComparison {
            session_uuid: row.get(0)?,
            project_name: row.get(1)?,
            started_at: row.get(2)?,
            duration_ms: row.get(3)?,
            total_cost_usd: row.get(4)?,
            total_turns: row.get(5)?,
            model: row.get(6)?,
        })
    })?;
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
