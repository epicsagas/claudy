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

// ── Domain types for analytics ──

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
    pub elapsed_ms: u64,
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
