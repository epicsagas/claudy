use serde::{Deserialize, Serialize};

// ── Input types for store operations ──

#[derive(Debug, Clone)]
pub struct NewSession {
    pub session_uuid: String,
    pub project_id: i64,
    pub source_file: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub first_message: Option<String>,
    pub started_at: Option<String>,
    /// Neutral source label (e.g. "live", "archive") for R2 archive fallback.
    /// NULL for pre-R2 rows.
    pub source_kind: Option<String>,
    /// A sidechain transcript — one spawned by another session (a subagent),
    /// found nested under that session's directory rather than at the project
    /// top level. Stored so aggregations can separate delegated work from the
    /// sessions a person actually opened.
    pub is_sidechain: bool,
}

#[derive(Debug, Clone)]
pub struct NewTurn {
    pub session_id: i64,
    pub turn_number: i32,
    pub prompt_text: Option<String>,
    pub response_text: Option<String>,
    pub model: Option<String>,
    pub duration_ms: Option<i64>,
    pub started_at: Option<String>,
    /// Neutral "author" flag — true when this turn's prompt was a genuine
    /// human-typed message (not a meta/command injection). Code-authorship-level
    /// human-vs-AI (which needs git) is deferred to the downstream consumer.
    pub human_authored: bool,
}

#[derive(Debug, Clone)]
pub struct NewTokenUsage {
    pub turn_id: i64,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_input_tokens: i64,
    pub cache_read_input_tokens: i64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone)]
pub struct NewToolCall {
    pub turn_id: i64,
    pub tool_use_id: String,
    pub tool_name: String,
    pub input_summary: Option<String>,
    pub is_error: bool,
    pub result_summary: Option<String>,
    pub duration_ms: Option<i64>,
}

/// Session-level outcome counters, written into `session_outcomes`.
///
/// These are observed outcomes only — what the session actually did, not how
/// much it cost. Token and cost figures live in `token_usage`/`sessions` and
/// are deliberately absent here.
///
/// `repo` is the raw session cwd, stored verbatim. claudy does no repo
/// curation: every session gets a row, and any canonicalization, grouping, or
/// filtering is left to whatever reads the table.
#[derive(Debug, Clone)]
pub struct NewSessionOutcome {
    pub session_uuid: String,
    pub repo: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    /// Every tool invocation observed, whatever the tool. This is the
    /// denominator the failure count is read against, so it is deliberately
    /// not restricted to the tools the other counters look at.
    pub n_tool_calls: i64,
    /// Tool invocations whose result was reported as an error.
    pub n_tool_fail: i64,
    /// Commit-shaped git invocations that were not observed to fail.
    pub commits_made: i64,
    /// Revert-shaped git invocations that were not observed to fail.
    pub reverts_made: i64,
}

/// How a batch of [`NewSessionOutcome`] counters relates to what is already stored.
///
/// A parse that started at byte 0 saw the whole transcript, so its counters
/// describe the entire session and supersede any stored row. A parse that
/// resumed from a checkpoint saw only the appended tail, so its counters are a
/// delta to add on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeWriteMode {
    /// Authoritative whole-session counts: insert, or replace what is stored.
    Replace,
    /// Tail-only counts: add to an existing row. Never creates a row, so a
    /// partial tail can't masquerade as a complete session.
    Accumulate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: i64,
    pub encoded_dir: String,
    pub display_name: String,
    pub resolved_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: i64,
    pub session_uuid: String,
    // NOTE: `is_sidechain` is stored on the sessions table but not carried on
    // this read model yet — no reader needs it; consumers that filter sidechains
    // do so in SQL.
    pub project_id: i64,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub total_turns: i32,
    pub total_cost_usd: f64,
    pub total_duration_ms: i64,
    pub first_message: Option<String>,
    pub source_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRecord {
    pub id: i64,
    pub session_id: i64,
    pub turn_number: i32,
    pub prompt_text: Option<String>,
    pub response_text: Option<String>,
    pub model: Option<String>,
    pub duration_ms: Option<i64>,
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageRecord {
    pub id: i64,
    pub turn_id: i64,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_input_tokens: i64,
    pub cache_read_input_tokens: i64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: i64,
    pub turn_id: i64,
    pub tool_use_id: String,
    pub tool_name: String,
    pub input_summary: Option<String>,
    pub is_error: bool,
    pub result_summary: Option<String>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMetricRecord {
    pub id: i64,
    pub session_id: Option<i64>,
    pub platform: String,
    pub channel_id: String,
    pub user_id: String,
    pub profile: Option<String>,
    pub stream_duration_ms: Option<i64>,
    pub first_byte_ms: Option<i64>,
    pub stream_timeout: bool,
    pub error_type: Option<String>,
}

// ── Analysis result types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTrendPoint {
    pub date: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_cost_usd: f64,
    pub session_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDistribution {
    pub tool_name: String,
    pub call_count: i64,
    pub error_count: i64,
    pub avg_duration_ms: Option<f64>,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    pub total_cost_usd: f64,
    pub avg_cost_per_session: f64,
    pub avg_cost_per_turn: f64,
    pub weekly_avg_cost: f64,
    pub total_sessions: i64,
    pub total_turns: i64,
    pub cache_savings_usd: f64,
    pub estimated_cost_portion: f64,
    pub by_model: Vec<ModelCostBreakdown>,
    pub most_expensive_session: Option<SessionCostHighlight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostBreakdown {
    pub model: String,
    pub total_cost_usd: f64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub session_count: i64,
    pub avg_cost_per_session: f64,
    pub percentage: f64,
    pub pricing_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCostHighlight {
    pub session_uuid: String,
    pub project_name: String,
    pub cost_usd: f64,
    pub turns: i32,
    pub model: Option<String>,
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_sessions: i64,
    pub total_cost_usd: f64,
    pub total_turns: i64,
    pub total_duration_ms: i64,
    pub avg_tokens_per_session: f64,
    pub cache_hit_ratio: f64,
    pub most_used_model: Option<String>,
    pub alert_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEfficiency {
    pub session_uuid: String,
    pub project_name: String,
    pub total_turns: i32,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub tool_call_count: i64,
    pub tool_overhead_ratio: f64,
    pub cache_hit_ratio: f64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPattern {
    pub sequence: Vec<String>,
    pub frequency: i64,
    pub error_rate: f64,
    pub is_anti_pattern: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model: String,
    pub avg_duration_ms: f64,
    pub avg_input_tokens: f64,
    pub avg_output_tokens: f64,
    pub avg_cost_per_session: f64,
    pub total_sessions: i64,
    pub cache_hit_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionComparison {
    pub session_uuid: String,
    pub project_name: String,
    pub started_at: Option<String>,
    pub duration_ms: i64,
    pub total_cost_usd: f64,
    pub total_turns: i32,
    pub model: Option<String>,
}

// ── Insights summary types (LLM-consumed) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsSummary {
    pub period: InsightsPeriod,
    pub overview: InsightsOverview,
    pub daily_costs: Vec<InsightsDailyCost>,
    pub model_distribution: Vec<ModelCostBreakdown>,
    pub tool_usage: Vec<ToolDistribution>,
    pub notable_sessions: Vec<SessionCostHighlight>,
    pub cost_analysis: InsightsCostAnalysis,
    pub cache_efficiency: InsightsCacheEfficiency,
    pub prompt_efficiency: Vec<PromptEfficiency>,
    pub tool_patterns: Vec<ToolPattern>,
    pub model_performance: Vec<ModelPerformance>,
    pub session_comparisons: Vec<SessionComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsPeriod {
    pub from: String,
    pub to: String,
    pub days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsOverview {
    pub total_sessions: i64,
    pub total_cost_usd: f64,
    pub total_turns: i64,
    pub avg_tokens_per_session: f64,
    pub most_used_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsDailyCost {
    pub date: String,
    pub cost_usd: f64,
    pub sessions: i64,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsCostAnalysis {
    pub total_cost_usd: f64,
    pub avg_cost_per_session: f64,
    pub avg_cost_per_turn: f64,
    pub weekly_avg_cost: f64,
    pub cache_savings_usd: f64,
    pub estimated_cost_portion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsCacheEfficiency {
    pub hit_ratio: f64,
    pub savings_usd: f64,
}

// ── Recommendation types ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationCategory {
    CostOptimization,
    PromptEfficiency,
    ModelSelection,
    ToolUsage,
    AntiPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub action: Option<String>,
}

// ── Ingestion types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionCheckpoint {
    pub file_path: String,
    pub file_modified: String,
    pub byte_offset: i64,
    pub line_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionResult {
    pub files_scanned: u32,
    pub files_ingested: u32,
    pub sessions_created: u32,
    pub turns_created: u32,
    pub token_records_created: u32,
    pub tool_calls_created: u32,
    /// Turns skipped due to a per-turn insert failure (the rest of the file still
    /// ingested). Surfaced so silent skips aren't hidden behind a success line.
    pub turns_skipped: u32,
    pub elapsed_ms: u64,
}

/// Freshness of ingested data, for staleness reporting (`analytics status`).
/// Domain-neutral: only timestamps + counts + per-source last-seen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessReport {
    /// Latest `turns.started_at` across all turns (RFC3339), or None if no turns.
    pub latest_turn_at: Option<String>,
    pub total_turns: i64,
    /// Per-source last-seen, keyed by the neutral `sessions.source_kind` label.
    /// Empty before R2 sources are tagged.
    pub per_source: Vec<FreshnessSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessSource {
    pub label: String,
    pub last_seen: Option<String>,
}

// ── Pricing types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model_id: String,
    pub input: f64,
    pub output: f64,
    pub cache_write: f64,
    pub cache_read: f64,
    pub source: String,
    pub synced_at: String,
}

// ── Parsed JSONL event (intermediate) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonlEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub session_id: Option<String>,
    pub timestamp: Option<String>,
    pub cwd: Option<String>,
    pub uuid: Option<String>,
    pub parent_uuid: Option<String>,
    pub model: Option<String>,
    pub version: Option<String>,
    pub is_meta: Option<bool>,
    pub message: Option<serde_json::Value>,
    pub subtype: Option<String>,
    pub duration_ms: Option<i64>,
    pub cost_usd: Option<f64>,
    pub num_turns: Option<i64>,
}
