// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use std::sync::Mutex;
use serde_json::{json, Value};
use crate::protocol::ContentItem;

// Embedded at compile time — used for Create baseline
const LLM_REF: &str = include_str!("../../../docs/llm/synoema.md");

// ── AppState ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Create,
    Check,
    Run,
    Debug,
}

impl AppState {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppState::Create => "Create",
            AppState::Check  => "Check",
            AppState::Run    => "Run",
            AppState::Debug  => "Debug",
        }
    }
}

// ── HistoryEntry ────────────────────────────────────────

#[derive(Debug, Clone)]
struct HistoryEntry {
    state: AppState,
    trigger: String,
}

// ── StateTracker ────────────────────────────────────────

pub struct StateTracker {
    current: AppState,
    history: Vec<HistoryEntry>,
    last_error: Option<String>,
}

const MAX_HISTORY: usize = 5;
const MAX_ERROR_LEN: usize = 500;

impl StateTracker {
    fn new() -> Self {
        StateTracker {
            current: AppState::Create,
            history: Vec::new(),
            last_error: None,
        }
    }

    fn transition(&mut self, next: AppState, trigger: &str) {
        if self.history.len() >= MAX_HISTORY {
            self.history.remove(0);
        }
        self.history.push(HistoryEntry {
            state: next,
            trigger: trigger.to_string(),
        });
        self.current = next;
    }

    pub fn on_tool_result(&mut self, tool: &str, is_error: bool, error_text: Option<&str>) {
        if is_error {
            if let Some(text) = error_text {
                if text.len() > MAX_ERROR_LEN {
                    let mut truncated = text[..MAX_ERROR_LEN].to_string();
                    truncated.push_str("... (truncated)");
                    self.last_error = Some(truncated);
                } else {
                    self.last_error = Some(text.to_string());
                }
            }
        }

        let next = match (tool, is_error) {
            ("eval", false)              => AppState::Create,
            ("eval", true)               => AppState::Check,
            ("typecheck", false)         => AppState::Check,
            ("typecheck", true)          => AppState::Check,
            ("run", false)               => AppState::Run,
            ("run", true)                => AppState::Debug,
            ("search_code", _)           => AppState::Create,
            ("get_context_for_edit", _)  => AppState::Create,
            ("recipe", _)                => AppState::Create,
            ("project_overview", _)      => AppState::Create,
            ("crate_info", _)            => AppState::Create,
            ("file_summary", _)          => AppState::Create,
            _ => return, // no transition for get_context, get_state, unknown
        };

        self.transition(next, tool);
    }
}

// ── Global singleton ────────────────────────────────────

static TRACKER: Mutex<Option<StateTracker>> = Mutex::new(None);

fn with_tracker<F, R>(f: F) -> R
where
    F: FnOnce(&mut StateTracker) -> R,
{
    let mut guard = TRACKER.lock().unwrap_or_else(|e| e.into_inner());
    let tracker = guard.get_or_insert_with(StateTracker::new);
    f(tracker)
}

// ── Public API ──────────────────────────────────────────

pub fn on_tool_result(tool: &str, is_error: bool, error_text: Option<&str>) {
    with_tracker(|t| t.on_tool_result(tool, is_error, error_text));
}

pub fn tool_get_state() -> (Vec<ContentItem>, bool) {
    let (state, history_json) = with_tracker(|t| {
        let hist: Vec<Value> = t.history.iter().map(|e| {
            json!({ "state": e.state.as_str(), "trigger": e.trigger })
        }).collect();
        (t.current.as_str().to_string(), json!(hist))
    });

    let result = json!({
        "state": state,
        "history": history_json
    });
    (vec![ContentItem::text(result.to_string())], false)
}

pub fn tool_get_context() -> (Vec<ContentItem>, bool) {
    let text = with_tracker(|t| baseline_context(t));
    (vec![ContentItem::text(text)], false)
}

// ── Baseline context per state ──────────────────────────

fn baseline_context(tracker: &StateTracker) -> String {
    match tracker.current {
        AppState::Create => context_create(),
        AppState::Check  => context_check(tracker),
        AppState::Run    => context_run(),
        AppState::Debug  => context_debug(tracker),
    }
}

fn context_create() -> String {
    let examples_hint = "Examples: synoema://examples (use resources/read to browse)";
    let tools_hint = "Tools: eval (test expr), typecheck (verify types), run (execute program)";
    format!("{LLM_REF}\n\n{examples_hint}\n{tools_hint}")
}

fn context_check(tracker: &StateTracker) -> String {
    let mut parts = vec![
        "State: Check — fixing type/parse errors".to_string(),
        "Tools: eval (test fix), typecheck (re-check), search_code (find related)".to_string(),
    ];

    if let Some(ref err) = tracker.last_error {
        parts.insert(0, format!("Last error:\n{err}"));
    }

    parts.push("Common fixes: check operator types, verify function arity, ensure main binding exists".to_string());

    parts.join("\n\n")
}

fn context_run() -> String {
    "State: Run — program executing successfully\n\n\
     Tools: eval (test changes), run (re-execute), typecheck (verify after edits)"
        .to_string()
}

fn context_debug(tracker: &StateTracker) -> String {
    let mut parts = vec![
        "State: Debug — investigating runtime error".to_string(),
        "Tools: search_code (find related code), get_context_for_edit (inspect source), eval (test hypothesis)".to_string(),
    ];

    if let Some(ref err) = tracker.last_error {
        parts.insert(0, format!("Last error:\n{err}"));
    }

    parts.push("Debug strategy: read error → find source location → understand context → fix → re-run".to_string());

    parts.join("\n\n")
}

// ── Tool definitions ────────────────────────────────────

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "get_context",
            "description": "Get state-aware baseline context for current development phase (Create/Check/Run/Debug). Returns relevant docs, errors, and tool hints.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        json!({
            "name": "get_state",
            "description": "Get current development state and recent transition history. States: Create (writing code), Check (fixing errors), Run (executing), Debug (investigating runtime errors).",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
    ]
}

// ── Tests ───────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_create() {
        let t = StateTracker::new();
        assert_eq!(t.current, AppState::Create);
    }

    #[test]
    fn eval_success_stays_create() {
        let mut t = StateTracker::new();
        t.on_tool_result("eval", false, None);
        assert_eq!(t.current, AppState::Create);
    }

    #[test]
    fn eval_error_transitions_to_check() {
        let mut t = StateTracker::new();
        t.on_tool_result("eval", true, Some(r#"{"code":"type_mismatch"}"#));
        assert_eq!(t.current, AppState::Check);
        assert_eq!(t.last_error.as_deref(), Some(r#"{"code":"type_mismatch"}"#));
    }

    #[test]
    fn typecheck_success_transitions_to_check() {
        let mut t = StateTracker::new();
        t.on_tool_result("typecheck", false, None);
        assert_eq!(t.current, AppState::Check);
    }

    #[test]
    fn typecheck_error_stays_check() {
        let mut t = StateTracker::new();
        t.on_tool_result("typecheck", true, Some("err"));
        assert_eq!(t.current, AppState::Check);
    }

    #[test]
    fn run_success_transitions_to_run() {
        let mut t = StateTracker::new();
        t.on_tool_result("run", false, None);
        assert_eq!(t.current, AppState::Run);
    }

    #[test]
    fn run_error_transitions_to_debug() {
        let mut t = StateTracker::new();
        t.on_tool_result("run", true, Some("runtime panic"));
        assert_eq!(t.current, AppState::Debug);
        assert_eq!(t.last_error.as_deref(), Some("runtime panic"));
    }

    #[test]
    fn search_code_transitions_to_create() {
        let mut t = StateTracker::new();
        t.on_tool_result("run", true, None); // Debug
        t.on_tool_result("search_code", false, None);
        assert_eq!(t.current, AppState::Create);
    }

    #[test]
    fn get_context_for_edit_transitions_to_create() {
        let mut t = StateTracker::new();
        t.on_tool_result("typecheck", true, None); // Check
        t.on_tool_result("get_context_for_edit", false, None);
        assert_eq!(t.current, AppState::Create);
    }

    #[test]
    fn unknown_tool_no_transition() {
        let mut t = StateTracker::new();
        t.on_tool_result("run", false, None); // Run
        t.on_tool_result("get_context", false, None); // should not change
        assert_eq!(t.current, AppState::Run);
    }

    #[test]
    fn history_max_five() {
        let mut t = StateTracker::new();
        for _ in 0..10 {
            t.on_tool_result("eval", false, None);
        }
        assert_eq!(t.history.len(), MAX_HISTORY);
    }

    #[test]
    fn history_records_trigger() {
        let mut t = StateTracker::new();
        t.on_tool_result("run", true, None);
        assert_eq!(t.history.last().unwrap().trigger, "run");
        assert_eq!(t.history.last().unwrap().state, AppState::Debug);
    }

    #[test]
    fn baseline_create_contains_llm_ref() {
        let t = StateTracker::new();
        let ctx = baseline_context(&t);
        assert!(ctx.contains("Synoema") || ctx.contains("synoema"));
    }

    #[test]
    fn baseline_check_contains_error_when_set() {
        let mut t = StateTracker::new();
        t.on_tool_result("eval", true, Some("test error"));
        let ctx = baseline_context(&t);
        assert!(ctx.contains("test error"));
    }

    #[test]
    fn baseline_debug_contains_error_when_set() {
        let mut t = StateTracker::new();
        t.on_tool_result("run", true, Some("panic at line 5"));
        let ctx = baseline_context(&t);
        assert!(ctx.contains("panic at line 5"));
    }

    #[test]
    fn baseline_run_is_minimal() {
        let mut t = StateTracker::new();
        t.on_tool_result("run", false, None);
        let ctx = baseline_context(&t);
        assert!(ctx.len() < 200);
    }

    #[test]
    fn full_cycle_create_check_debug_create() {
        let mut t = StateTracker::new();
        assert_eq!(t.current, AppState::Create);

        t.on_tool_result("typecheck", true, Some("type error"));
        assert_eq!(t.current, AppState::Check);

        t.on_tool_result("run", true, Some("runtime error"));
        assert_eq!(t.current, AppState::Debug);

        t.on_tool_result("eval", false, None);
        assert_eq!(t.current, AppState::Create);
    }

    #[test]
    fn long_error_is_truncated() {
        let mut t = StateTracker::new();
        let long_error = "x".repeat(1000);
        t.on_tool_result("eval", true, Some(&long_error));
        let stored = t.last_error.as_ref().unwrap();
        assert!(stored.len() <= MAX_ERROR_LEN + "... (truncated)".len());
        assert!(stored.ends_with("... (truncated)"));
    }

    #[test]
    fn short_error_not_truncated() {
        let mut t = StateTracker::new();
        t.on_tool_result("eval", true, Some("short error"));
        assert_eq!(t.last_error.as_deref(), Some("short error"));
    }
}
