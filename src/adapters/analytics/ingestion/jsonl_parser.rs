use crate::domain::analytics::{
    JsonlEvent, NewSession, NewSessionOutcome, NewTokenUsage, NewToolCall, NewTurn,
    OutcomeWriteMode,
};
use crate::ports::analytics_ports::{AnalyticsStore, PricingStore};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

/// Commit- and revert-shaped git invocations found in a single Bash command.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct GitActions {
    commits: i64,
    reverts: i64,
}

impl GitActions {
    fn is_empty(self) -> bool {
        self.commits == 0 && self.reverts == 0
    }
}

/// Classify the git actions in a raw Bash command string.
///
/// The command is read as whitespace-separated tokens rather than matched as a
/// substring, which is what stops neighbouring names from being mistaken for the
/// action: `commit-tree`, `merge-base`, `merge-file` and `mergetool` are
/// distinct tokens from `commit` and `merge`, so they never match. Text inside a
/// quoted commit message is inert for the same reason — the quote stays attached
/// to the token, so `"git` is not `git`.
///
/// Each `git <action>` in the command counts, so a chained
/// `git commit … && git commit …` contributes two. Whether the shell actually
/// reached the second one cannot be known from the transcript; this is a
/// best-effort read of a command string, and a result reported as an error later
/// withdraws whatever it credited.
fn classify_git_actions(cmd: &str) -> GitActions {
    let mut found = GitActions::default();
    let mut tokens = cmd.split_whitespace().peekable();

    while let Some(tok) = tokens.next() {
        if tok != "git" {
            continue;
        }

        // Skip git's own options so `git -C /path commit` still reads as a
        // commit. `-C`/`-c`/`--git-dir`/`--work-tree` take a separate argument;
        // every other option is self-contained (including the `--opt=value`
        // spellings of those four).
        let mut sub = None;
        while let Some(next) = tokens.next() {
            if matches!(next, "-C" | "-c" | "--git-dir" | "--work-tree") {
                tokens.next();
            } else if next.starts_with('-') {
                continue;
            } else {
                sub = Some(next);
                break;
            }
        }

        match sub {
            Some("commit" | "merge") => found.commits += 1,
            Some("revert") => found.reverts += 1,
            // Only a hard reset discards work; a soft or mixed reset just moves
            // the index. Look ahead no further than this command's own
            // arguments, so a `--hard` belonging to a later command in the same
            // chain is not attributed here.
            Some("reset")
                if tokens
                    .clone()
                    .take_while(|t| !matches!(*t, "&&" | "||" | ";" | "|"))
                    .any(|t| t == "--hard") =>
            {
                found.reverts += 1;
            }
            _ => {}
        }
    }

    found
}

/// Session outcome counters accumulated across one parse of one transcript.
#[derive(Default)]
struct OutcomeCounters {
    tool_calls: i64,
    tool_fail: i64,
    commits: i64,
    reverts: i64,
    /// Git actions already credited, keyed by the `tool_use_id` that produced
    /// them, so a result reported as an error can withdraw the credit — a
    /// rejected commit is not a commit. Entries still here at end of file keep
    /// their credit: an invocation whose result the transcript never recorded is
    /// far rarer than one that simply succeeded.
    credited_git: HashMap<String, GitActions>,
}

impl OutcomeCounters {
    fn record_tool_use(
        &mut self,
        tool_name: &str,
        tool_id: &str,
        input: Option<&serde_json::Value>,
    ) {
        self.tool_calls += 1;

        // Read the raw `command` string rather than the stored `input_summary`:
        // that summary is truncated and JSON-escaped, which both hides long
        // commands and drags in sibling fields such as the tool's `description`.
        if tool_name != "Bash" {
            return;
        }
        let Some(cmd) = input
            .and_then(|i| i.get("command"))
            .and_then(|c| c.as_str())
        else {
            return;
        };

        let actions = classify_git_actions(cmd);
        if actions.is_empty() {
            return;
        }
        self.commits += actions.commits;
        self.reverts += actions.reverts;
        if !tool_id.is_empty() {
            self.credited_git.insert(tool_id.to_string(), actions);
        }
    }

    fn record_tool_result(&mut self, tool_use_id: &str, is_error: bool) {
        if is_error {
            self.tool_fail += 1;
        }
        if let Some(actions) = self.credited_git.remove(tool_use_id)
            && is_error
        {
            self.commits -= actions.commits;
            self.reverts -= actions.reverts;
        }
    }
}

/// Apply one tool result to the store and, when `count_outcome`, to the session
/// counters.
///
/// `obj` is whichever object carries the result: a `tool_result` content block
/// inside a `user` message, or the `message` of a standalone `tool_result`
/// event. Both spell the fields the same way.
fn apply_tool_result(
    store: &dyn AnalyticsStore,
    obj: &serde_json::Value,
    outcomes: &mut OutcomeCounters,
    count_outcome: bool,
) {
    let is_error = obj
        .get("is_error")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let result_summary = obj.get("content").and_then(|c| {
        c.as_str().map(|s| truncate_str(s, 500)).or_else(|| {
            c.as_array().map(|arr| {
                let text: Vec<String> = arr
                    .iter()
                    .filter_map(|b| {
                        b.get("text")
                            .and_then(|t| t.as_str())
                            .map(std::string::ToString::to_string)
                    })
                    .collect();
                text.join("\n")
            })
        })
    });
    let tool_use_id = obj.get("tool_use_id").and_then(|v| v.as_str()).or_else(|| {
        obj.get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| {
                arr.iter()
                    .find_map(|b| b.get("tool_use_id").and_then(|v| v.as_str()))
            })
    });

    if let Some(tuid) = tool_use_id
        && !tuid.is_empty()
    {
        if let Err(e) = store.update_tool_call_result(tuid, is_error, result_summary.as_deref()) {
            tracing::warn!(error = %e, tool_use_id = %tuid, "failed to update tool call result");
        }
        if count_outcome {
            outcomes.record_tool_result(tuid, is_error);
        }
    }
}

/// Epoch milliseconds from a transcript timestamp, or None when it doesn't
/// parse. Transcript timestamps are RFC 3339 (`2026-07-20T10:00:02.123Z`).
fn ts_ms(ts: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|t| t.timestamp_millis())
}

/// Write the open turn's duration — from its start to its last work event — and
/// clear it. A turn whose span never advanced past its start (no timestamped
/// work observed) is left NULL rather than written as 0: an unmeasured duration
/// and an instant answer must stay distinguishable. Best-effort, like every
/// other per-row write in this parser.
fn close_turn_duration(
    store: &dyn AnalyticsStore,
    session_id: Option<i64>,
    open_turn: &mut Option<(i32, i64, i64)>,
) {
    if let (Some(sid), Some((turn_no, started, last))) = (session_id, open_turn.take())
        && last > started
        && let Err(e) = store.update_turn_duration(sid, turn_no, last - started)
    {
        tracing::warn!(error = %e, session_id = %sid, turn_no, "failed to update turn duration");
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let boundary = s.ceil_char_boundary(max);
        s[..boundary].to_string()
    }
}

fn redact_secrets(s: &str) -> String {
    let patterns = [
        ("sk-ant-api", "***REDACTED***"),
        ("ghp_", "***REDACTED***"),
        ("AKIA", "***REDACTED***"),
        ("xoxb-", "***REDACTED***"),
        ("xoxp-", "***REDACTED***"),
        ("xoxa-", "***REDACTED***"),
        ("xoxs-", "***REDACTED***"),
    ];
    let mut result = s.to_string();
    for (pat, repl) in patterns {
        if let Some(idx) = result.find(pat) {
            let end = (idx + 40).min(result.len());
            result.replace_range(idx..end, repl);
        }
    }
    result
}

pub struct IngestionStats {
    pub sessions_created: u32,
    pub turns_created: u32,
    pub token_records_created: u32,
    pub tool_calls_created: u32,
    /// Turns whose insert failed (e.g. a transient FK violation) and were skipped
    /// so the rest of the file still ingests. Surfaced to the user so silent
    /// skips aren't hidden behind a green "ingest complete" line.
    pub turns_skipped: u32,
    /// Byte offset to resume from on the next ingest of this file (R1/R3).
    /// Only fully-read, newline-terminated lines advance it; a trailing partial
    /// line is left for the next run to re-read.
    pub byte_offset: i64,
}

/// Per-file inputs to [`parse_and_ingest`], grouped to keep the function's
/// argument count under clippy's threshold now that R1 added `start_byte_offset`.
pub struct IngestFileArgs<'a> {
    pub project_id: i64,
    pub file_path: &'a Path,
    pub path_str: &'a str,
    pub full: bool,
    pub source_kind: Option<&'a str>,
    pub start_byte_offset: i64,
    /// The file is a sidechain (subagent) transcript, found nested under a
    /// session's directory rather than at the project top level. Its session
    /// row is flagged, and its turns are never human-authored — the "user"
    /// messages in a sidechain are the parent agent's prompts.
    pub is_sidechain: bool,
}

pub fn parse_and_ingest(
    store: &dyn AnalyticsStore,
    pricing_store: Option<&dyn PricingStore>,
    args: IngestFileArgs<'_>,
) -> anyhow::Result<IngestionStats> {
    let IngestFileArgs {
        project_id,
        file_path,
        path_str,
        full,
        source_kind,
        start_byte_offset,
        is_sidechain,
    } = args;

    // R1: resume from the last committed byte offset. Clamp to file length so a
    // file that shrank since the last ingest re-parses from a safe point.
    let mut file = std::fs::File::open(file_path)?;
    let file_len = file.metadata()?.len() as i64;
    let start = start_byte_offset.clamp(0, file_len) as u64;
    file.seek(SeekFrom::Start(start))?;
    let mut reader = BufReader::new(&mut file);

    let session_uuid = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut cursor: u64 = start;
    let mut stats = IngestionStats {
        sessions_created: 0,
        turns_created: 0,
        token_records_created: 0,
        tool_calls_created: 0,
        turns_skipped: 0,
        // Default to the *clamped* resume point, not the raw requested offset:
        // if the file shrank since the last ingest, persisting the unclamped
        // offset would leave the checkpoint stuck past EOF forever (a later
        // grow-back would then resume too far in and skip the gap). Only
        // fully-read, newline-terminated lines advance it further (R3).
        byte_offset: start as i64,
    };

    let mut turn_number: i32 = 0;
    let mut current_model: Option<String> = None;
    let mut session_cwd: Option<String> = None;
    let mut session_id: Option<i64> = None;
    let mut first_message: Option<String> = None;
    let mut total_cost: f64 = 0.0;
    let mut total_duration: i64 = 0;
    let mut last_timestamp: Option<String> = None;
    let mut first_timestamp: Option<String> = None;
    let mut pending_turn_id: Option<i64> = None;

    // Anchor for the session-duration fallback. Real transcripts carry no
    // terminal `result` event, so `total_duration` above never gets a value from
    // the transcript itself; when that happens the session's duration is the
    // span from this anchor to the last event seen. A parse from byte 0 anchors
    // on the first timestamp it reads; a resume anchors on the stored session
    // start, so the tail's span is measured from the session's true beginning
    // rather than from wherever the checkpoint happened to fall.
    // `saw_result_duration` records that THIS parse read an authoritative
    // duration from a `result` event — only then does the span defer to it.
    let mut span_anchor: Option<String> = None;
    let mut saw_result_duration = false;

    // The turn currently being answered: (turn_number, started_ms, last_work_ms).
    // `last_work_ms` advances on assistant events and tool results — the events
    // that ARE the work of answering — and deliberately not on the next user
    // message, whose arrival time measures the human's think time, not the
    // turn's. The turn's duration is written when the next accepted user event
    // opens a new turn, or at end of file. Keyed by (session, turn_number), not
    // the inserted row id, so a full re-ingest backfills turns that already
    // exist (their insert returns no id).
    let mut open_turn: Option<(i32, i64, i64)> = None;

    // Session outcome counters, written once at the end of the parse.
    //
    // `whole_file` distinguishes the two things a parse can be, and decides both
    // what gets counted and how it is written:
    //
    //   - starting at byte 0, everything observed is counted, because the counts
    //     will replace the stored row and must describe the whole session;
    //   - resuming from a checkpoint, only activity belonging to a turn this run
    //     newly inserted is counted, because the counts will be *added* to the
    //     stored row. Re-reading lines a previous run already ingested is
    //     possible (a checkpoint can lag), and those lines produce no new turn,
    //     so gating on that is what keeps the addition from double-counting.
    //
    // The cost of that gate is that a resume whose chunk opens midway through a
    // turn skips that turn's remaining tool calls, since no new turn is created
    // for them. That undercounts by at most one turn per resume and is corrected
    // by any later full re-ingest — the trade taken deliberately, because the
    // alternative risks silently inflating counts instead.
    let whole_file = start == 0;
    let mut outcomes = OutcomeCounters::default();

    let mut buf = Vec::new();
    loop {
        buf.clear();
        let n = reader.read_until(b'\n', &mut buf)?;
        if n == 0 {
            break;
        }
        let ended_with_newline = buf.last() == Some(&b'\n');
        if !ended_with_newline {
            // Partial trailing line (a flush in progress): do NOT process it
            // this run. Leave byte_offset at the start of this line so the
            // next run re-reads it whole (R3) — never silently drop a line.
            break;
        }
        let line = String::from_utf8_lossy(&buf).into_owned();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            cursor += n as u64;
            stats.byte_offset = cursor as i64;
            continue;
        }

        let event: JsonlEvent = match serde_json::from_str(trimmed) {
            Ok(e) => e,
            Err(_) => {
                // Malformed JSON on a full line: skip it but advance so we
                // don't re-parse it forever.
                cursor += n as u64;
                stats.byte_offset = cursor as i64;
                continue;
            }
        };

        // The line is fully read, so commit its offset now rather than at the
        // bottom of the loop. That is what makes the early `continue`s below
        // safe: an event the parser chooses to skip must still advance the
        // checkpoint, or the next run resumes short and re-reads lines it has
        // already ingested.
        cursor += n as u64;
        stats.byte_offset = cursor as i64;

        // Timestamp and cwd bookkeeping runs BEFORE the event is interpreted,
        // so an event the arms below skip early (a meta injection, a tool-result
        // carrier with no user text) still counts toward the session's span and
        // can still resolve its cwd. When it sat after the match, every early
        // `continue` silently dropped its event from the session's end time.
        if event.timestamp.is_some() {
            last_timestamp = event.timestamp.clone();
            if first_timestamp.is_none() {
                first_timestamp = event.timestamp.clone();
            }
        }
        // Claude Code emits cwd on several event types (assistant, attachment,
        // user, …), not only the terminal `result` event — which real
        // transcripts do not carry at all. First non-empty value wins; the
        // `result` arm below still overwrites when one does appear.
        if session_cwd.is_none()
            && let Some(c) = event.cwd.as_ref()
            && !c.is_empty()
        {
            session_cwd = Some(c.clone());
        }

        match event.event_type.as_str() {
            "user" => {
                // Tool results arrive as `tool_result` blocks inside the user
                // message. They are handled before the guards below because such
                // an event carries no user text of its own and would otherwise be
                // skipped whole — which is why no tool result was ever recorded.
                let mut carried_tool_result = false;
                if let Some(blocks) = event
                    .message
                    .as_ref()
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array())
                {
                    for block in blocks {
                        if block.get("type").is_some_and(|t| t == "tool_result") {
                            carried_tool_result = true;
                            apply_tool_result(
                                store,
                                block,
                                &mut outcomes,
                                whole_file || pending_turn_id.is_some(),
                            );
                        }
                    }
                }
                // A tool result is part of answering the open turn, so it
                // advances the turn's work clock (a plain user message does not
                // — its timestamp measures think time, and it closes the turn
                // below instead).
                if carried_tool_result
                    && let Some((_, _, last)) = &mut open_turn
                    && let Some(ms) = event.timestamp.as_deref().and_then(ts_ms)
                {
                    *last = ms.max(*last);
                }

                let is_meta = event.is_meta.unwrap_or(false);
                let is_command = event
                    .message
                    .as_ref()
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .is_some_and(|c| c.starts_with('/'));
                // R4: neutral "author" flag derived purely from transcript
                // metadata the parser already inspects. Meta/command injections
                // are skipped below, so persisted turns are human-authored by
                // construction; this records that invariant explicitly.
                // A sidechain's "user" messages are the parent agent's prompts,
                // never a person's.
                let human_authored = !(is_meta || is_command || is_sidechain);

                if is_meta || is_command {
                    continue;
                }

                let text = extract_text_from_message(&event.message);
                if text.is_none() {
                    continue;
                }

                if session_id.is_none() {
                    let ts = event
                        .timestamp
                        .as_deref()
                        .or(first_timestamp.as_deref())
                        .map(std::string::ToString::to_string);
                    let sid = match store.upsert_session(&NewSession {
                        session_uuid: session_uuid.clone(),
                        project_id,
                        source_file: path_str.to_string(),
                        cwd: session_cwd.clone(),
                        model: current_model.clone(),
                        first_message: text.as_ref().map(|t| redact_secrets(t)),
                        started_at: ts,
                        source_kind: source_kind.map(|s| s.to_string()),
                        is_sidechain,
                    }) {
                        Ok(sid) => sid,
                        Err(e) => {
                            // Without a session row every downstream turn/token/tool
                            // insert would FK-violate. Abort just this file with what
                            // we have so far; the caller logs the file and continues
                            // to the next one instead of aborting the whole run.
                            tracing::warn!(
                                error = %e,
                                %session_uuid,
                                source_file = path_str,
                                "failed to create session; skipping file",
                            );
                            return Ok(stats);
                        }
                    };
                    session_id = Some(sid);
                    stats.sessions_created += 1;
                    if !full {
                        // Incremental resume: continue numbering after the turns
                        // already stored for this session so appended turns don't
                        // collide with turn_number 1 and get gated away (R1/#53).
                        turn_number = store.get_turn_count(sid)? as i32;
                        // Seed the running cost/duration totals from the session's
                        // previously-persisted values. Only this run's appended
                        // lines are parsed, so without this the end-of-function
                        // `update_session_completion` call would overwrite the
                        // session's true totals with just the appended portion.
                        // A "result" event later in this run (if any) still wins,
                        // since it assigns the authoritative cumulative value.
                        if let Some(existing) = store.get_session_by_uuid(&session_uuid)? {
                            total_cost = existing.total_cost_usd;
                            total_duration = existing.total_duration_ms;
                            // Anchor the duration fallback on the session's true
                            // start: this resume only sees the appended tail, and
                            // a span measured from the tail's first event would
                            // shrink the session to its last append.
                            span_anchor = existing.started_at.clone();
                        }
                    }
                }

                if first_message.is_none() && text.is_some() {
                    first_message = text.as_ref().map(|t| redact_secrets(t));
                }

                // This user message opens a new turn, which closes the one being
                // answered: its duration runs from its own start to its last
                // work event, not to this message's arrival.
                close_turn_duration(store, session_id, &mut open_turn);
                if let Some(ms) = event.timestamp.as_deref().and_then(ts_ms) {
                    open_turn = Some((turn_number + 1, ms, ms));
                }

                turn_number += 1;
                let ts = event
                    .timestamp
                    .as_deref()
                    .map(std::string::ToString::to_string);
                let sid = session_id
                    .ok_or_else(|| anyhow::anyhow!("session_id not set before turn insertion"))?;
                match store.insert_turn(&NewTurn {
                    session_id: sid,
                    turn_number,
                    prompt_text: text.as_ref().map(|t| redact_secrets(t)),
                    response_text: None,
                    model: current_model.clone(),
                    duration_ms: None,
                    started_at: ts,
                    human_authored,
                }) {
                    Ok(Some(tid)) => {
                        // New turn — remember its id so the assistant/tool_result
                        // events that follow attach token-usage and tool-calls to it.
                        pending_turn_id = Some(tid);
                        stats.turns_created += 1;
                    }
                    Ok(None) => {
                        // Already ingested in a prior run (UNIQUE(session_id,
                        // turn_number) conflict on a re-parsed file). Drop the
                        // pending id so this turn's children are NOT re-inserted.
                        pending_turn_id = None;
                    }
                    Err(e) => {
                        // Skip just this turn's token usage / tool calls and
                        // continue with the next event, instead of letting a single
                        // insert failure (e.g. a transient FK violation) abort the
                        // whole ingestion run. Following assistant blocks see
                        // pending_turn_id == None and already skip their inserts.
                        stats.turns_skipped += 1;
                        tracing::warn!(
                            error = %e,
                            %sid,
                            turn_number,
                            "failed to insert turn; skipping turn"
                        );
                        pending_turn_id = None;
                    }
                }
                let _ = text; // text already used in insert_turn above
            }
            "assistant" => {
                // Assistant output is the work of answering the open turn.
                if let Some((_, _, last)) = &mut open_turn
                    && let Some(ms) = event.timestamp.as_deref().and_then(ts_ms)
                {
                    *last = ms.max(*last);
                }
                if let Some(msg) = &event.message {
                    if let Some(model) = msg.get("model").and_then(|m| m.as_str()) {
                        current_model = Some(model.to_string());
                    }

                    if let Some(usage) = msg.get("usage") {
                        let input = usage
                            .get("input_tokens")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let output = usage
                            .get("output_tokens")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let cache_creation = usage
                            .get("cache_creation_input_tokens")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let cache_read = usage
                            .get("cache_read_input_tokens")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let model_str = current_model.as_deref().unwrap_or("unknown");
                        let cost = if let Some(ps) = pricing_store {
                            crate::adapters::analytics::analysis::cost::estimate_cost_with_store(
                                ps,
                                model_str,
                                input,
                                output,
                                cache_creation,
                                cache_read,
                            )
                        } else {
                            crate::adapters::analytics::analysis::cost::estimate_cost(
                                model_str,
                                input,
                                output,
                                cache_creation,
                                cache_read,
                            )
                        };
                        total_cost += cost;

                        if let Some(tid) = pending_turn_id {
                            if let Err(e) = store.insert_token_usage(&NewTokenUsage {
                                turn_id: tid,
                                model: model_str.to_string(),
                                input_tokens: input,
                                output_tokens: output,
                                cache_creation_input_tokens: cache_creation,
                                cache_read_input_tokens: cache_read,
                                estimated_cost_usd: cost,
                            }) {
                                tracing::warn!(error = %e, "failed to insert token usage");
                            } else {
                                stats.token_records_created += 1;
                            }
                        }
                    }

                    // Extract tool_use content blocks
                    if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                        for block in content {
                            if block.get("type").is_some_and(|t| t == "tool_use") {
                                let tool_name = block
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("unknown");
                                let tool_id =
                                    block.get("id").and_then(|i| i.as_str()).unwrap_or_default();
                                let input = block.get("input");
                                let input_summary =
                                    input.map(|i| truncate_str(&i.to_string(), 500));

                                // Counted whether or not a tool_calls row is
                                // written below: on a re-ingest the row already
                                // exists (pending_turn_id is None), but the
                                // session's activity was still observed and the
                                // count must still reflect it.
                                if whole_file || pending_turn_id.is_some() {
                                    outcomes.record_tool_use(tool_name, tool_id, input);
                                }
                                if let Some(tid) = pending_turn_id {
                                    if let Err(e) = store.insert_tool_call(&NewToolCall {
                                        turn_id: tid,
                                        tool_use_id: tool_id.to_string(),
                                        tool_name: tool_name.to_string(),
                                        input_summary: input_summary.clone(),
                                        is_error: false,
                                        result_summary: None,
                                        duration_ms: None,
                                    }) {
                                        tracing::warn!(error = %e, tool = %tool_name, "failed to insert tool call");
                                    } else {
                                        stats.tool_calls_created += 1;
                                    }
                                }
                            }
                        }
                    }

                    // Extract response text for turn
                    if let (Some(_tid), Some(content)) = (pending_turn_id, msg.get("content")) {
                        let _response_text = extract_response_text(content);
                    }
                }
            }
            // Alternate shape, where the whole event is one tool result rather
            // than a block inside a user message. Handled identically.
            "tool_result" => {
                if let Some((_, _, last)) = &mut open_turn
                    && let Some(ms) = event.timestamp.as_deref().and_then(ts_ms)
                {
                    *last = ms.max(*last);
                }
                if let Some(msg) = &event.message {
                    apply_tool_result(
                        store,
                        msg,
                        &mut outcomes,
                        whole_file || pending_turn_id.is_some(),
                    );
                }
            }
            "result" => {
                if let Some(cost) = event.cost_usd {
                    total_cost = cost;
                }
                if let Some(dur) = event.duration_ms {
                    total_duration = dur;
                    saw_result_duration = true;
                }
                if let Some(cwd) = event.cwd.as_ref() {
                    session_cwd = Some(cwd.clone());
                }
            }
            _ => {}
        }
    }

    // Update session completion
    if let Some(sid) = session_id {
        // End of file closes the last turn.
        close_turn_duration(store, session_id, &mut open_turn);

        // Duration fallback: real transcripts carry no terminal `result` event,
        // so without this every session's duration would stay 0. When THIS
        // parse read no authoritative duration, extend the stored value to the
        // span from the session's start (anchor) to the last event seen —
        // extend, never shrink, so a resume that appends an hour of activity
        // moves the duration forward with `ended_at` instead of leaving it
        // frozen at the value seeded from the previous parse. A `result` event,
        // when one does appear, remains authoritative.
        if !saw_result_duration
            && let Some(start) = span_anchor.as_deref().or(first_timestamp.as_deref())
            && let (Some(s), Some(e)) = (ts_ms(start), last_timestamp.as_deref().and_then(ts_ms))
            && e - s > total_duration
        {
            total_duration = e - s;
        }

        let ended = last_timestamp.as_deref().unwrap_or("unknown");
        if let Err(e) =
            store.update_session_completion(sid, ended, turn_number, total_cost, total_duration)
        {
            tracing::warn!(error = %e, session_id = %sid, "failed to update session completion");
        }

        // R4: best-effort backfill of any turns whose model is NULL (e.g. the
        // opening turn created before the first assistant event set the model)
        // from the session model captured during parsing.
        if let Some(model) = current_model.as_deref()
            && let Err(e) = store.backfill_null_turn_models(sid, model)
        {
            tracing::warn!(error = %e, session_id = %sid, "failed to backfill turn models");
        }

        // Write this session's outcome counters. A parse that read the file from
        // byte 0 describes the whole session and replaces the stored row; a
        // resumed parse contributes only its tail and is added to that row.
        //
        // `repo` is the raw session cwd. claudy stores a row for every session
        // and leaves any grouping or canonicalization to whatever reads the
        // table. `ended_at` carries the last timestamp actually seen, or NULL —
        // a placeholder string would be indistinguishable from a real value.
        //
        // Best-effort: a failure here never aborts the rest of the ingestion.
        let mode = if whole_file {
            OutcomeWriteMode::Replace
        } else {
            OutcomeWriteMode::Accumulate
        };
        // A resume that observed nothing new has nothing to add.
        let has_delta = outcomes.tool_calls != 0
            || outcomes.tool_fail != 0
            || outcomes.commits != 0
            || outcomes.reverts != 0;
        if (whole_file || has_delta)
            && let Err(e) = store.upsert_session_outcome(
                &NewSessionOutcome {
                    session_uuid: session_uuid.clone(),
                    repo: session_cwd.clone().unwrap_or_default(),
                    started_at: first_timestamp.clone(),
                    ended_at: last_timestamp.clone(),
                    n_tool_calls: outcomes.tool_calls,
                    n_tool_fail: outcomes.tool_fail,
                    commits_made: outcomes.commits,
                    reverts_made: outcomes.reverts,
                },
                mode,
            )
        {
            tracing::warn!(error = %e, %session_uuid, "failed to write session outcome counters");
        }
    }

    Ok(stats)
}

fn extract_text_from_message(message: &Option<serde_json::Value>) -> Option<String> {
    let msg = message.as_ref()?;
    let content = msg.get("content")?;

    if let Some(s) = content.as_str() {
        return Some(truncate_str(s, 500));
    }

    if let Some(arr) = content.as_array() {
        let texts: Vec<String> = arr
            .iter()
            .filter(|b| b.get("type").is_some_and(|t| t == "text"))
            .filter_map(|b| {
                b.get("text")
                    .and_then(|t| t.as_str())
                    .map(std::string::ToString::to_string)
            })
            .collect();
        if !texts.is_empty() {
            return Some(truncate_str(&texts.join("\n"), 500));
        }
    }

    None
}

fn extract_response_text(content: &serde_json::Value) -> Option<String> {
    if let Some(s) = content.as_str() {
        return Some(truncate_str(s, 500));
    }

    if let Some(arr) = content.as_array() {
        let texts: Vec<String> = arr
            .iter()
            .filter(|b| b.get("type").is_some_and(|t| t == "text"))
            .filter_map(|b| {
                b.get("text")
                    .and_then(|t| t.as_str())
                    .map(std::string::ToString::to_string)
            })
            .collect();
        if !texts.is_empty() {
            return Some(truncate_str(&texts.join("\n"), 500));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::analytics::sqlite_store::SqliteAnalyticsStore;
    use crate::ports::analytics_ports::AnalyticsStore;
    use rusqlite::params;
    use tempfile::TempDir;

    const CWD: &str = "/home/dev/projects/claudy";

    /// A user event carrying one tool result, in the shape transcripts actually
    /// use: a `tool_result` block inside the user message, not an event of its
    /// own. The event has no user text, so everything about it has to be read
    /// before the parser's no-text guard.
    fn tool_result_line(ts: &str, tool_use_id: &str, is_error: bool) -> String {
        format!(
            r#"{{"type":"user","timestamp":"{ts}","cwd":"{CWD}","message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"{tool_use_id}","is_error":{is_error},"content":"r"}}]}}}}
"#
        )
    }

    fn user_line(ts: &str, text: &str) -> String {
        format!(
            r#"{{"type":"user","timestamp":"{ts}","cwd":"{CWD}","message":{{"role":"user","content":"{text}"}}}}
"#
        )
    }

    /// Injected metadata rather than something the user typed — the parser skips
    /// these without creating a turn.
    fn meta_user_line(ts: &str) -> String {
        format!(
            r#"{{"type":"user","isMeta":true,"timestamp":"{ts}","cwd":"{CWD}","message":{{"role":"user","content":"noise"}}}}
"#
        )
    }

    /// An assistant event whose content is the given raw `tool_use` blocks.
    fn assistant_line(ts: &str, blocks: &str) -> String {
        format!(
            r#"{{"type":"assistant","timestamp":"{ts}","cwd":"{CWD}","message":{{"role":"assistant","model":"m","content":[{blocks}],"usage":{{"input_tokens":1,"output_tokens":1,"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}}}}
"#
        )
    }

    fn bash_block(id: &str, command: &str) -> String {
        let escaped = command.replace('\\', r"\\").replace('"', r#"\""#);
        format!(
            r#"{{"type":"tool_use","id":"{id}","name":"Bash","input":{{"command":"{escaped}"}}}}"#
        )
    }

    fn read_block(id: &str) -> String {
        format!(r#"{{"type":"tool_use","id":"{id}","name":"Read","input":{{"file_path":"y.rs"}}}}"#)
    }

    fn store_in(dir: &TempDir) -> SqliteAnalyticsStore {
        let db = dir.path().join("analytics.db");
        let store = SqliteAnalyticsStore::open(db.to_str().unwrap()).unwrap();
        store.initialize_schema().unwrap();
        store
    }

    /// The one session_outcomes row for a session_uuid, or None.
    /// (repo, n_tool_calls, n_tool_fail, commits_made, reverts_made).
    fn outcome_row(
        store: &SqliteAnalyticsStore,
        uuid: &str,
    ) -> Option<(String, i64, i64, i64, i64)> {
        let conn = store.lock().unwrap();
        conn.query_row(
            "SELECT repo, n_tool_calls, n_tool_fail, commits_made, reverts_made
             FROM session_outcomes WHERE session_uuid = ?1",
            params![uuid],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                ))
            },
        )
        .ok()
    }

    fn count_outcome_rows(store: &SqliteAnalyticsStore, uuid: &str) -> i64 {
        store
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM session_outcomes WHERE session_uuid = ?1",
                params![uuid],
                |r| r.get(0),
            )
            .unwrap_or(0)
    }

    fn ingest(
        store: &SqliteAnalyticsStore,
        file: &Path,
        path_str: &str,
        project_id: i64,
        full: bool,
        start_byte_offset: i64,
    ) -> IngestionStats {
        parse_and_ingest(
            store,
            None,
            IngestFileArgs {
                project_id,
                file_path: file,
                path_str,
                full,
                source_kind: None,
                start_byte_offset,
                is_sidechain: false,
            },
        )
        .unwrap()
    }

    fn write(path: &Path, s: &str) {
        std::fs::write(path, s).unwrap();
    }

    fn append(path: &Path, s: &str) {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().append(true).open(path).unwrap();
        f.write_all(s.as_bytes()).unwrap();
    }

    // ── the git classifier ──

    /// Tokens, not substrings: an action is only an action when it stands as its
    /// own word. Commands that merely share a prefix (`merge-base`, `merge-file`,
    /// `mergetool`, `commit-tree`) inspect or plumb rather than commit, and
    /// counting them would inflate every commit-derived figure.
    #[test]
    fn test_classify_git_actions_token_boundary() {
        let commits = |c: &str| classify_git_actions(c).commits;
        let reverts = |c: &str| classify_git_actions(c).reverts;

        assert_eq!(commits("git commit -m ship"), 1);
        assert_eq!(commits("cd x && git merge --no-ff main"), 1);
        assert_eq!(commits("git commit"), 1, "action as the final token");
        assert_eq!(commits("git -C /srv/repo commit -m x"), 1, "global option");
        assert_eq!(commits("git --no-pager commit -m x"), 1);
        assert_eq!(reverts("git revert HEAD"), 1);
        assert_eq!(reverts("git reset --hard origin/main"), 1);
        assert_eq!(reverts("git reset HEAD~1 --hard"), 1, "flag after the ref");

        // Prefix lookalikes must not count.
        assert_eq!(commits("git merge-base main dev"), 0);
        assert_eq!(commits("git merge-file a b c"), 0);
        assert_eq!(commits("git mergetool"), 0);
        assert_eq!(commits("git commit-tree HEAD^{tree}"), 0);
        assert_eq!(
            reverts("git reset --soft HEAD~1"),
            0,
            "soft reset keeps work"
        );
        assert_eq!(reverts("git reset"), 0, "mixed reset keeps work");

        // A `--hard` belonging to a later command is not attributed backwards.
        assert_eq!(reverts("git reset HEAD~1 && git checkout --hard-ish"), 0);

        // Text inside a quoted message is not a command.
        assert_eq!(commits(r#"echo "git commit is next""#), 0);
        assert_eq!(reverts(r#"git commit -m "git revert is not needed""#), 0);

        // Each invocation counts.
        assert_eq!(commits("git commit -m a && git commit -m b"), 2);
        assert_eq!(
            classify_git_actions("git commit -m a && git revert HEAD"),
            GitActions {
                commits: 1,
                reverts: 1
            }
        );
    }

    // ── collection ──

    /// A row is written for every session whatever its cwd — there is no list of
    /// interesting directories. `repo` is the cwd verbatim.
    #[test]
    fn test_outcome_row_written_for_any_cwd() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-any.jsonl");
        write(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line(
                    "2026-07-20T10:00:01Z",
                    &bash_block("tu1", "git commit -m x")
                ),
                tool_result_line("2026-07-20T10:00:02Z", "tu1", false),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-any.jsonl", pid, true, 0);

        let (repo, _calls, _fail, commits, _reverts) =
            outcome_row(&store, "sess-any").expect("a row is written regardless of cwd");
        assert_eq!(repo, CWD, "repo is the raw cwd — stored, not curated");
        assert_eq!(commits, 1);
    }

    /// `n_tool_calls` counts every tool, not just the ones the other counters
    /// look at, because it is the denominator the failure count is read against.
    /// `n_tool_fail` counts results reported as errors — which only works if
    /// `tool_result` blocks inside user messages are read, since that is the
    /// shape transcripts use.
    #[test]
    fn test_outcome_counts_all_tools_and_records_failures() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-counts.jsonl");
        let blocks = format!(
            "{},{},{},{}",
            bash_block("tu1", "git commit -m ship"),
            bash_block("tu2", "git merge-base main dev"),
            bash_block("tu3", "git revert HEAD"),
            read_block("tu4"),
        );
        write(
            &file,
            &format!(
                "{}{}{}{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line("2026-07-20T10:00:01Z", &blocks),
                tool_result_line("2026-07-20T10:00:02Z", "tu1", false),
                tool_result_line("2026-07-20T10:00:03Z", "tu2", true),
                tool_result_line("2026-07-20T10:00:04Z", "tu3", false),
                tool_result_line("2026-07-20T10:00:05Z", "tu4", false),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();

        let stats = ingest(&store, &file, "sess-counts.jsonl", pid, true, 0);
        assert_eq!(stats.tool_calls_created, 4);

        let (repo, calls, fail, commits, reverts) =
            outcome_row(&store, "sess-counts").expect("row");
        assert_eq!(repo, CWD);
        assert_eq!(calls, 4, "every tool counts, not just Bash");
        assert_eq!(fail, 1, "the errored result is counted");
        assert_eq!(commits, 1, "git commit counts; git merge-base does not");
        assert_eq!(reverts, 1);

        // The same results reach the tool_calls rows they belong to.
        let errored: i64 = store
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM tool_calls WHERE is_error = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(errored, 1, "the result is recorded on the tool call too");
    }

    /// A commit that the transcript shows failing is not a commit. The credit
    /// taken when the command was issued is withdrawn once the result says it
    /// errored.
    #[test]
    fn test_outcome_failed_commit_credit_is_withdrawn() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-fail.jsonl");
        write(
            &file,
            &format!(
                "{}{}{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line(
                    "2026-07-20T10:00:01Z",
                    &format!(
                        "{},{}",
                        bash_block("tu1", "git commit -m rejected"),
                        bash_block("tu2", "git commit -m accepted"),
                    ),
                ),
                tool_result_line("2026-07-20T10:00:02Z", "tu1", true),
                tool_result_line("2026-07-20T10:00:03Z", "tu2", false),
                user_line("2026-07-20T10:00:04Z", "done"),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-fail.jsonl", pid, true, 0);

        let (_repo, calls, fail, commits, _reverts) =
            outcome_row(&store, "sess-fail").expect("row");
        assert_eq!(calls, 2);
        assert_eq!(fail, 1);
        assert_eq!(commits, 1, "only the commit that did not fail counts");
    }

    /// The action is read from the raw command, so a long command still counts.
    /// Matching the stored `input_summary` instead would miss this one, since
    /// that summary is cut off well before the `git commit`.
    #[test]
    fn test_outcome_counts_commit_past_the_input_summary_cutoff() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-long.jsonl");
        let long_prefix = "echo ".to_string() + &"x".repeat(600);
        let command = format!("{long_prefix} && git commit -m x");
        write(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line("2026-07-20T10:00:01Z", &bash_block("tu1", &command)),
                tool_result_line("2026-07-20T10:00:02Z", "tu1", false),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-long.jsonl", pid, true, 0);

        let (_repo, _calls, _fail, commits, _reverts) =
            outcome_row(&store, "sess-long").expect("row");
        assert_eq!(commits, 1, "the command is read in full, not truncated");
    }

    /// The alternate shape — a standalone `tool_result` event — is read the same
    /// way, so transcripts written that way keep working.
    #[test]
    fn test_outcome_reads_standalone_tool_result_events() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-legacy.jsonl");
        write(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line("2026-07-20T10:00:01Z", &read_block("tu1")),
                r#"{"type":"tool_result","timestamp":"2026-07-20T10:00:02Z","message":{"tool_use_id":"tu1","is_error":true,"content":"boom"}}
"#,
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-legacy.jsonl", pid, true, 0);

        let (_repo, calls, fail, _commits, _reverts) =
            outcome_row(&store, "sess-legacy").expect("row");
        assert_eq!((calls, fail), (1, 1));
    }

    // ── re-ingest and resume ──

    /// A full re-ingest replaces the counts from its own reading of the file:
    /// no doubling, and no zeroing either, even though every turn already exists
    /// by then and so no tool_calls row is written the second time round.
    #[test]
    fn test_outcome_full_reingest_replaces_counts() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-reingest.jsonl");
        write(
            &file,
            &format!(
                "{}{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line(
                    "2026-07-20T10:00:01Z",
                    &format!(
                        "{},{}",
                        bash_block("tu1", "git commit -m x"),
                        read_block("tu2")
                    ),
                ),
                tool_result_line("2026-07-20T10:00:02Z", "tu1", false),
                tool_result_line("2026-07-20T10:00:03Z", "tu2", false),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();

        ingest(&store, &file, "sess-reingest.jsonl", pid, true, 0);
        assert_eq!(
            outcome_row(&store, "sess-reingest").unwrap(),
            (CWD.to_string(), 2, 0, 1, 0)
        );

        ingest(&store, &file, "sess-reingest.jsonl", pid, true, 0);
        assert_eq!(
            outcome_row(&store, "sess-reingest").unwrap(),
            (CWD.to_string(), 2, 0, 1, 0),
            "a re-read of the same file yields the same counts"
        );
        assert_eq!(
            count_outcome_rows(&store, "sess-reingest"),
            1,
            "one row per session"
        );
    }

    /// A resume adds what it read to the stored row, so activity appended after
    /// the first ingest is reflected instead of being lost until someone runs a
    /// full re-ingest.
    #[test]
    fn test_outcome_resume_accumulates_appended_activity() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-resume.jsonl");
        let head = format!(
            "{}{}{}",
            user_line("2026-07-20T10:00:00Z", "go"),
            assistant_line(
                "2026-07-20T10:00:01Z",
                &bash_block("tu1", "git commit -m x")
            ),
            tool_result_line("2026-07-20T10:00:02Z", "tu1", false),
        );
        write(&file, &head);
        let pid = store.upsert_project("-x", "x", None).unwrap();

        let stats = ingest(&store, &file, "sess-resume.jsonl", pid, true, 0);
        assert_eq!(
            outcome_row(&store, "sess-resume").unwrap(),
            (CWD.to_string(), 1, 0, 1, 0)
        );

        // Append a second turn and resume exactly where the last run stopped.
        append(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T11:00:00Z", "more"),
                assistant_line("2026-07-20T11:00:01Z", &read_block("tu2")),
                tool_result_line("2026-07-20T11:00:02Z", "tu2", true),
            ),
        );
        ingest(
            &store,
            &file,
            "sess-resume.jsonl",
            pid,
            false,
            stats.byte_offset,
        );

        assert_eq!(
            outcome_row(&store, "sess-resume").unwrap(),
            (CWD.to_string(), 2, 1, 1, 0),
            "the appended turn is added to the stored counts"
        );
    }

    /// A resume that re-reads lines an earlier run already ingested must not
    /// count them twice. Those lines produce no new turn, which is exactly the
    /// signal used to leave them out of the delta.
    #[test]
    fn test_outcome_resume_does_not_double_count_replayed_lines() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-replay.jsonl");
        let first = user_line("2026-07-20T10:00:00Z", "go");
        let head = format!(
            "{}{}{}",
            first,
            assistant_line(
                "2026-07-20T10:00:01Z",
                &bash_block("tu1", "git commit -m x")
            ),
            tool_result_line("2026-07-20T10:00:02Z", "tu1", false),
        );
        write(&file, &head);
        let pid = store.upsert_project("-x", "x", None).unwrap();

        ingest(&store, &file, "sess-replay.jsonl", pid, true, 0);
        assert_eq!(
            outcome_row(&store, "sess-replay").unwrap(),
            (CWD.to_string(), 1, 0, 1, 0)
        );

        append(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T11:00:00Z", "more"),
                assistant_line("2026-07-20T11:00:01Z", &read_block("tu2")),
                tool_result_line("2026-07-20T11:00:02Z", "tu2", false),
            ),
        );
        // Resume from a lagging offset: everything after the first line gets
        // read a second time, including the commit already counted.
        ingest(
            &store,
            &file,
            "sess-replay.jsonl",
            pid,
            false,
            first.len() as i64,
        );

        assert_eq!(
            outcome_row(&store, "sess-replay").unwrap(),
            (CWD.to_string(), 2, 0, 1, 0),
            "replayed lines are not counted again"
        );
    }

    /// A resume never creates a row. Counts from a tail alone would look like a
    /// whole session while describing a fragment of one; the row is created by
    /// the next parse that reads the file from the start.
    #[test]
    fn test_outcome_resume_does_not_create_a_row() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-nocreate.jsonl");
        let head = user_line("2026-07-20T10:00:00Z", "go");
        write(&file, &head);
        let pid = store.upsert_project("-x", "x", None).unwrap();
        let stats = ingest(&store, &file, "sess-nocreate.jsonl", pid, true, 0);

        // Drop the row, as if this session predated outcome collection.
        store
            .lock()
            .unwrap()
            .execute("DELETE FROM session_outcomes", [])
            .unwrap();

        append(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T11:00:00Z", "more"),
                assistant_line("2026-07-20T11:00:01Z", &read_block("tu2")),
                tool_result_line("2026-07-20T11:00:02Z", "tu2", false),
            ),
        );
        ingest(
            &store,
            &file,
            "sess-nocreate.jsonl",
            pid,
            false,
            stats.byte_offset,
        );
        assert_eq!(
            count_outcome_rows(&store, "sess-nocreate"),
            0,
            "a tail is not a session"
        );

        // A parse from the start builds the whole row.
        ingest(&store, &file, "sess-nocreate.jsonl", pid, true, 0);
        assert_eq!(
            outcome_row(&store, "sess-nocreate").unwrap(),
            (CWD.to_string(), 1, 0, 0, 0)
        );
    }

    /// `ended_at` holds a timestamp that was actually seen, or nothing at all —
    /// never a placeholder that reads like data.
    #[test]
    fn test_outcome_ended_at_is_null_when_no_timestamp_was_seen() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-nots.jsonl");
        write(
            &file,
            &format!(
                r#"{{"type":"user","cwd":"{CWD}","message":{{"role":"user","content":"go"}}}}
"#
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-nots.jsonl", pid, true, 0);

        let ended: Option<String> = store
            .lock()
            .unwrap()
            .query_row(
                "SELECT ended_at FROM session_outcomes WHERE session_uuid = 'sess-nots'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ended, None);
    }

    // ── resume offset ──

    /// Events the parser skips still advance the resume offset. When they did
    /// not, the checkpoint fell short of the file and every later run re-read
    /// lines it had already ingested.
    #[test]
    fn test_byte_offset_covers_skipped_events() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-off.jsonl");
        // The middle two events are skipped: injected metadata, and a slash
        // command.
        let body = format!(
            "{}{}{}{}",
            user_line("2026-07-20T10:00:00Z", "go"),
            meta_user_line("2026-07-20T10:00:01Z"),
            user_line("2026-07-20T10:00:02Z", "/status"),
            assistant_line("2026-07-20T10:00:03Z", &read_block("tu1")),
        );
        write(&file, &body);
        let pid = store.upsert_project("-x", "x", None).unwrap();

        let stats = ingest(&store, &file, "sess-off.jsonl", pid, true, 0);
        assert_eq!(
            stats.byte_offset,
            body.len() as i64,
            "the offset reaches the end of the file"
        );
    }

    // ── durations ──

    fn session_duration(store: &SqliteAnalyticsStore, uuid: &str) -> i64 {
        store
            .lock()
            .unwrap()
            .query_row(
                "SELECT total_duration_ms FROM sessions WHERE session_uuid = ?1",
                params![uuid],
                |r| r.get(0),
            )
            .unwrap()
    }

    fn turn_durations(store: &SqliteAnalyticsStore, uuid: &str) -> Vec<Option<i64>> {
        let conn = store.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT t.duration_ms FROM turns t
                 JOIN sessions s ON s.id = t.session_id
                 WHERE s.session_uuid = ?1 ORDER BY t.turn_number",
            )
            .unwrap();
        stmt.query_map(params![uuid], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    /// Transcripts carry no terminal `result` event, so a session's duration
    /// must come from the span of its own timestamps — without the fallback,
    /// every session's duration is 0 forever.
    #[test]
    fn test_session_duration_falls_back_to_timestamp_span() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-span.jsonl");
        write(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line("2026-07-20T10:00:01Z", &read_block("tu1")),
                tool_result_line("2026-07-20T10:00:06Z", "tu1", false),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-span.jsonl", pid, true, 0);
        assert_eq!(
            session_duration(&store, "sess-span"),
            6_000,
            "duration is the first..last timestamp span"
        );
    }

    /// A `result` event, when one does appear, stays authoritative — the
    /// fallback only fills the gap it leaves.
    #[test]
    fn test_result_event_duration_stays_authoritative() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-res.jsonl");
        let result_line = format!(
            r#"{{"type":"result","timestamp":"2026-07-20T10:00:09Z","duration_ms":1234,"cost_usd":0.0,"cwd":"{CWD}"}}
"#
        );
        write(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line("2026-07-20T10:00:01Z", &read_block("tu1")),
                result_line,
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-res.jsonl", pid, true, 0);
        assert_eq!(session_duration(&store, "sess-res"), 1234);
    }

    /// A turn's duration runs from its user message to its last work event
    /// (assistant output, tool results) — NOT to the next user message, whose
    /// arrival time measures the human's think time. The last turn closes at
    /// end of file.
    #[test]
    fn test_turn_duration_ends_at_last_work_event() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-td.jsonl");
        write(
            &file,
            &format!(
                "{}{}{}{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line("2026-07-20T10:00:01Z", &read_block("tu1")),
                tool_result_line("2026-07-20T10:00:02Z", "tu1", false),
                // One hour of human think time before the next ask.
                user_line("2026-07-20T11:00:00Z", "more"),
                assistant_line("2026-07-20T11:00:01Z", &read_block("tu2")),
                tool_result_line("2026-07-20T11:00:05Z", "tu2", false),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-td.jsonl", pid, true, 0);
        assert_eq!(
            turn_durations(&store, "sess-td"),
            vec![Some(2_000), Some(5_000)],
            "turn 1 excludes think time; turn 2 closes at EOF"
        );
    }

    /// A growing session's duration advances with its resumes. The resume seeds
    /// `total_duration` from the stored row, so a `== 0` gate would freeze the
    /// duration at the first parse's span forever while `ended_at` kept moving;
    /// the fallback must extend to the new span instead (and never shrink).
    #[test]
    fn test_incremental_resume_extends_session_span() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-grow.jsonl");
        write(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line("2026-07-20T10:00:01Z", &read_block("tu1")),
                tool_result_line("2026-07-20T10:00:02Z", "tu1", false),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        let stats = ingest(&store, &file, "sess-grow.jsonl", pid, true, 0);
        assert_eq!(session_duration(&store, "sess-grow"), 2_000);

        // An hour later the session continues; the resume sees only the tail.
        append(
            &file,
            &format!(
                "{}{}",
                user_line("2026-07-20T11:00:00Z", "more"),
                assistant_line("2026-07-20T11:00:01Z", &read_block("tu2")),
            ),
        );
        ingest(
            &store,
            &file,
            "sess-grow.jsonl",
            pid,
            false,
            stats.byte_offset,
        );
        assert_eq!(
            session_duration(&store, "sess-grow"),
            3_601_000,
            "duration advances to the new span, anchored on the session start"
        );
    }

    /// A full re-ingest backfills durations onto turns that already exist —
    /// the update is keyed by (session, turn_number), which a re-parse can
    /// always reconstruct, not by the inserted row id, which it never sees.
    #[test]
    fn test_full_reingest_backfills_turn_durations() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let file = dir.path().join("sess-tdb.jsonl");
        write(
            &file,
            &format!(
                "{}{}{}",
                user_line("2026-07-20T10:00:00Z", "go"),
                assistant_line("2026-07-20T10:00:01Z", &read_block("tu1")),
                tool_result_line("2026-07-20T10:00:03Z", "tu1", false),
            ),
        );
        let pid = store.upsert_project("-x", "x", None).unwrap();
        ingest(&store, &file, "sess-tdb.jsonl", pid, true, 0);
        assert_eq!(turn_durations(&store, "sess-tdb"), vec![Some(3_000)]);

        // Simulate rows written by a build that predates duration tracking.
        store
            .lock()
            .unwrap()
            .execute("UPDATE turns SET duration_ms = NULL", [])
            .unwrap();
        store
            .lock()
            .unwrap()
            .execute("UPDATE sessions SET total_duration_ms = 0", [])
            .unwrap();

        ingest(&store, &file, "sess-tdb.jsonl", pid, true, 0);
        assert_eq!(
            turn_durations(&store, "sess-tdb"),
            vec![Some(3_000)],
            "re-ingest restores durations for pre-existing turns"
        );
        assert_eq!(
            session_duration(&store, "sess-tdb"),
            3_000,
            "re-ingest restores the session span too"
        );
    }
}
