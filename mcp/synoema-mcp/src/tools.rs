// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use serde_json::{json, Value};
use crate::protocol::ContentItem;

// ── Tool definitions ─────────────────────────────────────

pub fn list() -> Value {
    let mut tools = vec![
        json!({
            "name": "eval",
            "description": "Evaluate a Synoema expression. Returns value and inferred type, or a structured error.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "A Synoema expression (e.g. \"6 * 7\" or \"let x = 3 in x + 1\")"
                    }
                },
                "required": ["code"]
            }
        }),
        json!({
            "name": "typecheck",
            "description": "Type-check a full Synoema program. Returns the type of `main` or structured errors.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "A full Synoema program with a `main` binding"
                    }
                },
                "required": ["code"]
            }
        }),
        json!({
            "name": "run",
            "description": "Run a full Synoema program through the interpreter. Returns stdout output and final result.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "A full Synoema program with a `main` binding"
                    }
                },
                "required": ["code"]
            }
        }),
    ];

    // Dev intelligence tools
    tools.extend(crate::dev_tools::tool_definitions());
    tools.push(crate::recipes::tool_definition());

    // State-aware context tools
    tools.extend(crate::state::tool_definitions());

    json!({ "tools": tools })
}

// ── Tool call dispatch ───────────────────────────────────

pub fn call(name: &str, args: &Value) -> (Vec<ContentItem>, bool) {
    // State-aware context tools
    if name == "get_context" {
        return crate::state::tool_get_context();
    }
    if name == "get_state" {
        return crate::state::tool_get_state();
    }

    // Dev intelligence tools (no "code" param needed)
    if let Some(result) = crate::dev_tools::call(name, args) {
        return result;
    }
    if name == "recipe" {
        return crate::recipes::call(args);
    }

    // Language tools (require "code" param)
    let code = match args.get("code").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return (vec![ContentItem::text("missing required argument: code")], true),
    };

    match name {
        "eval"      => tool_eval(code),
        "typecheck" => tool_typecheck(code),
        "run"       => tool_run(code),
        other       => (vec![ContentItem::text(format!("unknown tool: {other}"))], true),
    }
}

// ── eval ─────────────────────────────────────────────────

fn tool_eval(code: &str) -> (Vec<ContentItem>, bool) {
    match synoema_eval::eval_expr(code) {
        Ok(val) => {
            // Try to show type alongside value (best-effort)
            let wrapped = format!("__eval_result = {code}");
            let output = match synoema_types::typecheck(&wrapped) {
                Ok(tenv) => {
                    if let Some(scheme) = tenv.lookup("__eval_result") {
                        format!("{val} : {}", scheme.ty)
                    } else {
                        val.to_string()
                    }
                }
                Err(_) => val.to_string(),
            };
            (vec![ContentItem::text(output)], false)
        }
        Err(diag) => {
            let text = synoema_diagnostic::render_json(&diag);
            (vec![ContentItem::text(text)], true)
        }
    }
}

// ── typecheck ────────────────────────────────────────────

fn tool_typecheck(code: &str) -> (Vec<ContentItem>, bool) {
    match synoema_types::typecheck(code) {
        Ok(tenv) => {
            let text = if let Some(scheme) = tenv.lookup("main") {
                format!("main : {}", scheme.ty)
            } else {
                "OK (no main binding)".to_string()
            };
            (vec![ContentItem::text(text)], false)
        }
        Err(e) => {
            let diag = synoema_eval::type_err_to_diagnostic(e);
            let text = synoema_diagnostic::render_json(&diag);
            (vec![ContentItem::text(text)], true)
        }
    }
}

// ── run ──────────────────────────────────────────────────

fn tool_run(code: &str) -> (Vec<ContentItem>, bool) {
    match synoema_eval::eval_main(code) {
        Ok((val, output)) => {
            let mut lines: Vec<String> = output;
            lines.push(val.to_string());
            (vec![ContentItem::text(lines.join("\n"))], false)
        }
        Err(diag) => {
            let text = synoema_diagnostic::render_json(&diag);
            (vec![ContentItem::text(text)], true)
        }
    }
}
