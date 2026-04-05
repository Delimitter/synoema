// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use serde_json::{json, Value};
use std::path::PathBuf;

// Embedded at compile time — always consistent with the repo
const LANGUAGE_REFERENCE: &str =
    include_str!("../../../docs/specs/language_reference.md");

const LLM_REF: &str =
    include_str!("../../../docs/llm/synoema.md");

// ── Resource list ────────────────────────────────────────

pub fn list() -> Value {
    json!({
        "resources": [
            {
                "uri": "synoema://spec/language_reference",
                "name": "Language Reference",
                "description": "Full Synoema language specification",
                "mimeType": "text/markdown"
            },
            {
                "uri": "synoema://spec/llm_ref",
                "name": "LLM Quick Reference",
                "description": "Minified Synoema reference optimised for LLM code generation (≤1500 tokens)",
                "mimeType": "text/markdown"
            },
            {
                "uri": "synoema://examples",
                "name": "Examples index",
                "description": "List of available Synoema example programs",
                "mimeType": "text/plain"
            }
        ]
    })
}

// ── Resource read ─────────────────────────────────────────

pub fn read(uri: &str) -> Result<Value, String> {
    match uri {
        "synoema://spec/language_reference" => Ok(text_content(LANGUAGE_REFERENCE)),
        "synoema://spec/llm_ref"            => Ok(text_content(LLM_REF)),
        "synoema://examples"                => examples_index(),
        other => {
            if let Some(name) = other.strip_prefix("synoema://examples/") {
                read_example(name)
            } else {
                Err(format!("unknown resource URI: {other}"))
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────

fn text_content(text: &str) -> Value {
    json!({
        "contents": [{ "uri": "", "mimeType": "text/markdown", "text": text }]
    })
}

fn examples_index() -> Result<Value, String> {
    let root = synoema_root();
    let examples_dir = root.join("lang").join("examples");

    let entries = std::fs::read_dir(&examples_dir)
        .map_err(|e| format!("cannot read examples dir {}: {e}", examples_dir.display()))?;

    let mut names: Vec<String> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().into_string().ok()?;
            if name.ends_with(".sno") { Some(name) } else { None }
        })
        .collect();

    names.sort();

    let listing = names.iter()
        .map(|n| format!("synoema://examples/{n}"))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(json!({
        "contents": [{ "uri": "synoema://examples", "mimeType": "text/plain", "text": listing }]
    }))
}

fn read_example(name: &str) -> Result<Value, String> {
    // Sanitize: no path traversal
    if name.contains('/') || name.contains("..") {
        return Err(format!("invalid example name: {name}"));
    }

    let root = synoema_root();
    let path = root.join("lang").join("examples").join(name);

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    let uri = format!("synoema://examples/{name}");
    Ok(json!({
        "contents": [{ "uri": uri, "mimeType": "text/plain", "text": content }]
    }))
}

/// Resolve the repo root directory.
///
/// Priority:
/// 1. `SYNOEMA_ROOT` environment variable
/// 2. Walk up from the executable's directory until we find a `lang/` subdirectory
pub fn synoema_root() -> PathBuf {
    if let Ok(root) = std::env::var("SYNOEMA_ROOT") {
        return PathBuf::from(root);
    }

    // Walk up from exe
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        for _ in 0..8 {
            if dir.join("lang").exists() {
                return dir;
            }
            if let Some(parent) = dir.parent() {
                dir = parent.to_path_buf();
            } else {
                break;
            }
        }
    }

    // Fallback: current dir
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
