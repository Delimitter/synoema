//! Synoema MCP Server
//!
//! Implements the Model Context Protocol (2024-11-05) over stdio.
//! Exposes eval/typecheck/run tools and language spec resources.
//!
//! Usage: pipe JSON-RPC requests to stdin, responses on stdout.
//! Configure in Claude Desktop (claude_desktop_config.json):
//!
//!   "synoema": { "command": "/path/to/synoema-mcp" }

mod protocol;
mod prompts;
mod resources;
mod tools;

use std::io::{self, BufRead, Write};
use protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{json, Value};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => l,
            Err(_) => break,
        };

        let response = handle_line(&line);

        // Notifications (no id) get no response
        if let Some(resp) = response {
            let json = serde_json::to_string(&resp).unwrap_or_default();
            let mut out = stdout.lock();
            writeln!(out, "{json}").ok();
            out.flush().ok();
        }
    }
}

fn handle_line(line: &str) -> Option<JsonRpcResponse> {
    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return Some(JsonRpcResponse::error(None, -32700, format!("parse error: {e}")));
        }
    };

    let id = req.id.clone();
    let params = req.params.unwrap_or(Value::Null);

    let result = match req.method.as_str() {
        "initialize"              => Some(handle_initialize()),
        "notifications/initialized" => return None, // no response for notifications
        "tools/list"              => Some(tools::list()),
        "tools/call"              => Some(handle_tools_call(&params)),
        "resources/list"          => Some(resources::list()),
        "resources/read"          => Some(handle_resources_read(&params)),
        "prompts/list"            => Some(prompts::list()),
        "prompts/get"             => Some(handle_prompts_get(&params)),
        other => {
            return Some(JsonRpcResponse::error(
                id, -32601, format!("method not found: {other}"),
            ));
        }
    };

    result.map(|r| JsonRpcResponse::result(id, r))
}

// ── initialize ────────────────────────────────────────────

fn handle_initialize() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": "synoema",
            "version": "0.1.0"
        }
    })
}

// ── tools/call ────────────────────────────────────────────

fn handle_tools_call(params: &Value) -> Value {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let args = params.get("arguments").unwrap_or(&Value::Null);

    let (content, is_error) = tools::call(name, args);
    let content_json: Vec<Value> = content.iter()
        .map(|c| json!({ "type": c.kind, "text": c.text }))
        .collect();

    if is_error {
        json!({ "content": content_json, "isError": true })
    } else {
        json!({ "content": content_json })
    }
}

// ── resources/read ───────────────────────────────────────

fn handle_resources_read(params: &Value) -> Value {
    let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
    match resources::read(uri) {
        Ok(v) => v,
        Err(e) => json!({ "contents": [{ "uri": uri, "mimeType": "text/plain", "text": e }] }),
    }
}

// ── prompts/get ───────────────────────────────────────────

fn handle_prompts_get(params: &Value) -> Value {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    match prompts::get(name) {
        Ok(v) => v,
        Err(e) => json!({ "error": e }),
    }
}
