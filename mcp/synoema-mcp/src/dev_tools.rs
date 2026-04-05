// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use serde::Serialize;
use serde_json::{json, Value};
use crate::index;
use crate::protocol::ContentItem;

// ── doc_query serde types ───────────────────────────────

#[derive(Serialize)]
struct DocQueryResult {
    file: String,
    description: Option<String>,
    examples: Vec<DocExample>,
    modules: Vec<DocModule>,
    functions: Vec<DocFunction>,
    types: Vec<DocType>,
}

#[derive(Serialize)]
struct DocExample {
    expr: String,
    expected: Option<String>,
}

#[derive(Serialize)]
struct DocModule {
    name: String,
    doc: Vec<String>,
    functions: Vec<DocFunction>,
    types: Vec<DocType>,
}

#[derive(Serialize)]
struct DocFunction {
    name: String,
    comment: Option<String>,
    doc: Vec<String>,
    line: u32,
}

#[derive(Serialize)]
struct DocType {
    name: String,
    comment: Option<String>,
    doc: Vec<String>,
    variants: Vec<String>,
    line: u32,
}

// ── Tool definitions ─────────────────────────────────────

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "project_overview",
            "description": "Get Synoema project structure: crates, LOC, tests, dependencies. Returns compact overview ≤300 tokens.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        json!({
            "name": "crate_info",
            "description": "Get pub API surface of a Synoema crate: functions, types, structs with signatures.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "crate_name": {
                        "type": "string",
                        "description": "Crate name, e.g. \"synoema-types\" or \"synoema-eval\""
                    }
                },
                "required": ["crate_name"]
            }
        }),
        json!({
            "name": "file_summary",
            "description": "Get function list with signatures (no bodies) for a Rust source file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Path relative to repo root, e.g. \"lang/crates/synoema-eval/src/eval.rs\""
                    }
                },
                "required": ["file"]
            }
        }),
        json!({
            "name": "search_code",
            "description": "Search Synoema codebase by keyword. Returns top-5 matches with context.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search keyword (case-insensitive substring match)"
                    },
                    "scope": {
                        "type": "string",
                        "description": "Search scope: \"code\" (.rs only), \"docs\" (.md only), \"all\" (default)",
                        "enum": ["code", "docs", "all"]
                    }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "get_context_for_edit",
            "description": "Get focused code context around a specific line: enclosing function, ±20 lines, local variables.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Path relative to repo root"
                    },
                    "line": {
                        "type": "integer",
                        "description": "Line number (1-based)"
                    }
                },
                "required": ["file", "line"]
            }
        }),
        json!({
            "name": "doc_query",
            "description": "Extract structured documentation from a Synoema .sno file: module description, function/type list with inline comments and doc-comments, examples. Returns JSON ≤500 tokens.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Path to .sno file relative to repo root (e.g. 'lang/examples/sorting.sno')"
                    }
                },
                "required": ["file"]
            }
        }),
    ]
}

// ── Tool dispatch ────────────────────────────────────────

pub fn call(name: &str, args: &Value) -> Option<(Vec<ContentItem>, bool)> {
    match name {
        "project_overview"    => Some(tool_project_overview()),
        "crate_info"          => Some(tool_crate_info(args)),
        "file_summary"        => Some(tool_file_summary(args)),
        "search_code"         => Some(tool_search_code(args)),
        "get_context_for_edit"=> Some(tool_get_context(args)),
        "doc_query"           => Some(tool_doc_query(args)),
        _ => None,
    }
}

// ── project_overview ─────────────────────────────────────

fn tool_project_overview() -> (Vec<ContentItem>, bool) {
    let idx = index::global();
    let crates = idx.all_crates();

    let total_loc: usize = crates.iter().map(|c| c.loc).sum();
    let total_tests: usize = crates.iter().map(|c| c.tests).sum();

    let crate_lines: Vec<String> = crates.iter().map(|c| {
        format!("  {}: {} LOC, {} tests{}", c.name, c.loc, c.tests,
            if c.purpose.is_empty() { String::new() } else { format!(" — {}", c.purpose) })
    }).collect();

    let text = format!(
        "Synoema: {} crates, {} LOC, {} tests\n\nCrates:\n{}",
        crates.len(), total_loc, total_tests, crate_lines.join("\n")
    );

    (vec![ContentItem::text(text)], false)
}

// ── crate_info ───────────────────────────────────────────

fn tool_crate_info(args: &Value) -> (Vec<ContentItem>, bool) {
    let name = match args.get("crate_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return (vec![ContentItem::text("missing required: crate_name")], true),
    };

    let idx = index::global();
    let ci = match idx.get_crate(name) {
        Some(c) => c,
        None => return (vec![ContentItem::text(format!("unknown crate: {name}"))], true),
    };

    let mut pub_fns = Vec::new();
    let mut pub_types = Vec::new();
    let mut pub_structs = Vec::new();

    for (path, fi) in &ci.files {
        let fname = path.file_name().unwrap_or_default().to_string_lossy();
        for f in &fi.functions {
            if f.vis == "pub" {
                pub_fns.push(format!("  {}:{} fn {}{}", fname, f.line, f.name, f.sig));
            }
        }
        for e in &fi.enums {
            if e.vis == "pub" {
                let variants = if e.variants.len() <= 8 {
                    e.variants.join(", ")
                } else {
                    format!("{}, ... ({} total)", e.variants[..5].join(", "), e.variants.len())
                };
                pub_types.push(format!("  {}:{} enum {} {{ {} }}", fname, e.line, e.name, variants));
            }
        }
        for s in &fi.structs {
            if s.vis == "pub" {
                let fields = if s.fields.len() <= 5 {
                    s.fields.join(", ")
                } else {
                    format!("{}, ... ({} total)", s.fields[..3].join(", "), s.fields.len())
                };
                pub_structs.push(format!("  {}:{} struct {} {{ {} }}", fname, s.line, s.name, fields));
            }
        }
    }

    let mut parts = vec![format!("{name}: {} LOC, {} tests", ci.total_loc, ci.total_tests)];
    if !ci.purpose.is_empty() {
        parts.push(format!("Purpose: {}", ci.purpose));
    }
    if !ci.internal_deps.is_empty() {
        parts.push(format!("Deps: {}", ci.internal_deps.join(", ")));
    }
    if !pub_fns.is_empty() {
        parts.push(format!("Functions:\n{}", pub_fns.join("\n")));
    }
    if !pub_types.is_empty() {
        parts.push(format!("Types:\n{}", pub_types.join("\n")));
    }
    if !pub_structs.is_empty() {
        parts.push(format!("Structs:\n{}", pub_structs.join("\n")));
    }

    // Truncate to ~2000 chars (~500 tokens)
    let mut text = parts.join("\n\n");
    if text.len() > 2000 {
        text.truncate(1950);
        text.push_str("\n... (truncated)");
    }

    (vec![ContentItem::text(text)], false)
}

// ── file_summary ─────────────────────────────────────────

fn tool_file_summary(args: &Value) -> (Vec<ContentItem>, bool) {
    let file = match args.get("file").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return (vec![ContentItem::text("missing required: file")], true),
    };

    let idx = index::global();
    let fi = match idx.get_file(std::path::Path::new(file)) {
        Some(f) => f,
        None => return (vec![ContentItem::text(format!("file not found: {file}"))], true),
    };

    let fn_lines: Vec<String> = fi.functions.iter().map(|f| {
        format!("  {}:{} {} fn {}{}", file.rsplit('/').next().unwrap_or(file), f.line, f.vis, f.name, f.sig)
    }).collect();

    let enum_lines: Vec<String> = fi.enums.iter().map(|e| {
        format!("  {}:{} {} enum {} [{} variants]", file.rsplit('/').next().unwrap_or(file), e.line, e.vis, e.name, e.variants.len())
    }).collect();

    let struct_lines: Vec<String> = fi.structs.iter().map(|s| {
        format!("  {}:{} {} struct {} [{} fields]", file.rsplit('/').next().unwrap_or(file), s.line, s.vis, s.name, s.fields.len())
    }).collect();

    let mut parts = vec![format!("{file}: {} LOC, {} tests", fi.loc, fi.test_count)];
    if !fn_lines.is_empty() {
        parts.push(format!("Functions ({}):\n{}", fn_lines.len(), fn_lines.join("\n")));
    }
    if !enum_lines.is_empty() {
        parts.push(format!("Enums:\n{}", enum_lines.join("\n")));
    }
    if !struct_lines.is_empty() {
        parts.push(format!("Structs:\n{}", struct_lines.join("\n")));
    }

    let mut text = parts.join("\n\n");
    if text.len() > 1200 {
        text.truncate(1150);
        text.push_str("\n... (truncated)");
    }

    (vec![ContentItem::text(text)], false)
}

// ── search_code ──────────────────────────────────────────

fn tool_search_code(args: &Value) -> (Vec<ContentItem>, bool) {
    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return (vec![ContentItem::text("missing required: query")], true),
    };
    let scope = args.get("scope").and_then(|v| v.as_str()).unwrap_or("all");

    let idx = index::global();
    let results = idx.search(query, scope);

    if results.is_empty() {
        return (vec![ContentItem::text(format!("no matches for \"{query}\" in scope={scope}"))], false);
    }

    let lines: Vec<String> = results.iter().map(|r| {
        format!("{}:{}\n{}", r.file, r.line, r.context)
    }).collect();

    let text = format!("Found {} match(es) for \"{}\":\n\n{}", results.len(), query, lines.join("\n\n"));

    (vec![ContentItem::text(text)], false)
}

// ── get_context_for_edit ─────────────────────────────────

fn tool_get_context(args: &Value) -> (Vec<ContentItem>, bool) {
    let file = match args.get("file").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return (vec![ContentItem::text("missing required: file")], true),
    };
    let line = match args.get("line").and_then(|v| v.as_u64()) {
        Some(l) => l as usize,
        None => return (vec![ContentItem::text("missing required: line")], true),
    };

    let idx = index::global();
    let root = &crate::resources::synoema_root();
    let abs = if std::path::Path::new(file).is_absolute() {
        std::path::PathBuf::from(file)
    } else {
        root.join(file)
    };

    let content = match std::fs::read_to_string(&abs) {
        Ok(c) => c,
        Err(_) => return (vec![ContentItem::text(format!("cannot read: {file}"))], true),
    };

    let lines: Vec<&str> = content.lines().collect();
    if line == 0 || line > lines.len() {
        return (vec![ContentItem::text(format!("line {line} out of range (1..{})", lines.len()))], true);
    }

    // Find enclosing function from index
    let fi = idx.get_file(std::path::Path::new(file));
    let enclosing = fi.as_ref().and_then(|fi| {
        fi.functions.iter()
            .rfind(|f| f.line <= line)
    });

    let ctx_start = line.saturating_sub(20);
    let ctx_end = (line + 20).min(lines.len());
    let snippet: Vec<String> = (ctx_start..ctx_end).map(|i| {
        let marker = if i + 1 == line { ">>>" } else { "   " };
        format!("{marker} {:>4} | {}", i + 1, lines[i])
    }).collect();

    let mut parts = Vec::new();
    if let Some(func) = enclosing {
        parts.push(format!("In function: {} {}{}", func.vis, func.name, func.sig));
    }
    parts.push(format!("File: {file}, line {line}"));
    parts.push(snippet.join("\n"));

    let mut text = parts.join("\n\n");
    if text.len() > 2000 {
        text.truncate(1950);
        text.push_str("\n... (truncated)");
    }

    (vec![ContentItem::text(text)], false)
}

// ── doc_query ───────────────────────────────────────────

fn tool_doc_query(args: &Value) -> (Vec<ContentItem>, bool) {
    let file = match args.get("file").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return (vec![ContentItem::text("missing required: file")], true),
    };

    let root = crate::resources::synoema_root();
    let abs = if std::path::Path::new(file).is_absolute() {
        std::path::PathBuf::from(file)
    } else {
        root.join(file)
    };

    let source = match std::fs::read_to_string(&abs) {
        Ok(s) => s,
        Err(_) => return (vec![ContentItem::text(format!("cannot read: {file}"))], true),
    };

    let program = match synoema_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => return (vec![ContentItem::text(format!("parse error: {e}"))], true),
    };

    // Extract -- comments from source by line
    let lines_vec: Vec<&str> = source.lines().collect();

    let find_comment = |decl_line: u32| -> Option<String> {
        if decl_line <= 1 { return None; }
        let mut collected = Vec::new();
        let mut check = decl_line - 1; // 1-based
        loop {
            if check == 0 { break; }
            let idx = (check - 1) as usize;
            if idx >= lines_vec.len() { break; }
            let trimmed = lines_vec[idx].trim();
            if trimmed.starts_with("--") && !trimmed.starts_with("---") {
                let text = trimmed.strip_prefix("--").unwrap().trim();
                if !text.starts_with("SPDX-") {
                    collected.push(text.to_string());
                }
                check -= 1;
            } else {
                break;
            }
        }
        if collected.is_empty() { return None; }
        collected.reverse();
        Some(collected.join("\n"))
    };

    let filter_doc = |doc: &[String]| -> Vec<String> {
        doc.iter()
            .filter(|l| !l.starts_with("example:") && !l.starts_with("guide:")
                && !l.starts_with("order:") && !l.starts_with("requires:"))
            .cloned()
            .collect()
    };

    // Top-level description from first doc-comment block
    let top_doc: Vec<String> = program.modules.first()
        .filter(|m| !m.doc.is_empty())
        .map(|m| m.doc.clone())
        .unwrap_or_else(|| {
            for decl in &program.decls {
                let doc = match decl {
                    synoema_parser::Decl::Func { doc, .. } => doc,
                    synoema_parser::Decl::TypeDef { doc, .. } => doc,
                    synoema_parser::Decl::TraitDecl { doc, .. } => doc,
                    _ => continue,
                };
                if !doc.is_empty() { return doc.clone(); }
            }
            Vec::new()
        });

    let description = top_doc.iter()
        .find(|l| !l.starts_with("guide:") && !l.starts_with("order:")
            && !l.starts_with("requires:") && !l.starts_with("example:"))
        .cloned();

    // Examples
    let examples: Vec<DocExample> = top_doc.iter()
        .filter_map(|l| l.strip_prefix("example:").map(|r| r.trim().to_string()))
        .map(|ex| {
            if let Some(pos) = ex.find("==") {
                DocExample {
                    expr: ex[..pos].trim().to_string(),
                    expected: Some(ex[pos + 2..].trim().to_string()),
                }
            } else {
                DocExample { expr: ex, expected: None }
            }
        })
        .collect();

    let make_func = |name: &str, doc: &[String], line: u32| -> DocFunction {
        DocFunction {
            name: name.to_string(),
            comment: find_comment(line),
            doc: filter_doc(doc),
            line,
        }
    };

    let make_type = |name: &str, doc: &[String], variants: &[synoema_parser::Variant], line: u32| -> DocType {
        DocType {
            name: name.to_string(),
            comment: find_comment(line),
            doc: doc.iter().filter(|l| !l.starts_with("example:")).cloned().collect(),
            variants: variants.iter().map(|v| v.name.clone()).collect(),
            line,
        }
    };

    // Modules
    let modules: Vec<DocModule> = program.modules.iter().map(|module| {
        let mut funcs = Vec::new();
        let mut types = Vec::new();
        for decl in &module.body {
            match decl {
                synoema_parser::Decl::Func { name, doc, span, .. } =>
                    funcs.push(make_func(name, doc, span.start.line)),
                synoema_parser::Decl::TypeDef { name, doc, variants, span, .. } =>
                    types.push(make_type(name, doc, variants, span.start.line)),
                _ => {}
            }
        }
        DocModule {
            name: module.name.clone(),
            doc: filter_doc(&module.doc),
            functions: funcs,
            types,
        }
    }).collect();

    // Top-level
    let mut top_funcs = Vec::new();
    let mut top_types = Vec::new();
    for decl in &program.decls {
        match decl {
            synoema_parser::Decl::Func { name, doc, span, .. } =>
                top_funcs.push(make_func(name, doc, span.start.line)),
            synoema_parser::Decl::TypeDef { name, doc, variants, span, .. } =>
                top_types.push(make_type(name, doc, variants, span.start.line)),
            _ => {}
        }
    }

    let result = DocQueryResult {
        file: file.to_string(),
        description,
        examples,
        modules,
        functions: top_funcs,
        types: top_types,
    };

    let mut text = serde_json::to_string(&result).unwrap_or_default();
    if text.len() > 2000 {
        text.truncate(1950);
        text.push_str("... (truncated)\"}");
    }

    (vec![ContentItem::text(text)], false)
}

// ── Tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_overview_returns_crates() {
        let (content, is_error) = tool_project_overview();
        assert!(!is_error);
        let text = &content[0].text;
        // Should mention at least some known crate
        assert!(text.contains("synoema") || text.contains("crates") || text.contains("LOC"));
    }

    #[test]
    fn crate_info_unknown_returns_error() {
        let args = json!({"crate_name": "nonexistent-crate-xyz"});
        let (content, is_error) = tool_crate_info(&args);
        assert!(is_error);
        assert!(content[0].text.contains("unknown crate"));
    }

    #[test]
    fn file_summary_missing_arg() {
        let args = json!({});
        let (content, is_error) = tool_file_summary(&args);
        assert!(is_error);
        assert!(content[0].text.contains("missing required"));
    }

    #[test]
    fn search_code_no_results() {
        let args = json!({"query": "xyznonexistent_totally_unique_string_42"});
        let (content, is_error) = tool_search_code(&args);
        assert!(!is_error);
        assert!(content[0].text.contains("no matches"));
    }

    #[test]
    fn get_context_missing_args() {
        let args = json!({});
        let (_content, is_error) = tool_get_context(&args);
        assert!(is_error);
    }

    #[test]
    fn doc_query_missing_file() {
        let args = json!({});
        let (content, is_error) = tool_doc_query(&args);
        assert!(is_error);
        assert!(content[0].text.contains("missing required"));
    }

    #[test]
    fn doc_query_nonexistent_file() {
        let args = json!({"file": "nonexistent_xyz.sno"});
        let (content, is_error) = tool_doc_query(&args);
        assert!(is_error);
        assert!(content[0].text.contains("cannot read"));
    }

    #[test]
    fn doc_query_valid_file() {
        let args = json!({"file": "lang/examples/sorting.sno"});
        let (content, is_error) = tool_doc_query(&args);
        assert!(!is_error);
        let text = &content[0].text;
        assert!(text.contains("\"file\":\"lang/examples/sorting.sno\""));
        assert!(text.contains("\"name\":\"insert\""));
        assert!(text.contains("\"name\":\"msort\""));
    }

    #[test]
    fn doc_query_returns_valid_json() {
        let args = json!({"file": "lang/examples/sorting.sno"});
        let (content, is_error) = tool_doc_query(&args);
        assert!(!is_error);
        let text = &content[0].text;
        // Must parse as valid JSON (serde roundtrip)
        let parsed: Value = serde_json::from_str(text).expect("doc_query must return valid JSON");
        assert!(parsed.get("file").is_some());
        assert!(parsed.get("functions").is_some());
    }
}
