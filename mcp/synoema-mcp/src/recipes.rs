// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use serde_json::{json, Value};
use crate::index;
use crate::protocol::ContentItem;

// ── Tool definition ──────────────────────────────────────

pub fn tool_definition() -> Value {
    json!({
        "name": "recipe",
        "description": "Get a dynamic step-by-step recipe for common Synoema development tasks. Steps include current line numbers from AST analysis.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Recipe name: \"add_operator\", \"add_builtin\", \"add_type\", or \"fix_from_error\"",
                    "enum": ["add_operator", "add_builtin", "add_type", "fix_from_error"]
                },
                "file": {
                    "type": "string",
                    "description": "For fix_from_error: the file containing the error"
                },
                "line": {
                    "type": "integer",
                    "description": "For fix_from_error: the line number of the error"
                }
            },
            "required": ["task"]
        }
    })
}

// ── Tool dispatch ────────────────────────────────────────

pub fn call(args: &Value) -> (Vec<ContentItem>, bool) {
    let task = match args.get("task").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return (vec![ContentItem::text("missing required: task")], true),
    };

    match task {
        "add_operator"  => recipe_add_operator(),
        "add_builtin"   => recipe_add_builtin(),
        "add_type"      => recipe_add_type(),
        "fix_from_error" => {
            let file = args.get("file").and_then(|v| v.as_str()).unwrap_or("");
            let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            recipe_fix_from_error(file, line)
        }
        _ => (vec![ContentItem::text(json!({
            "error": "unknown recipe",
            "available": ["add_operator", "add_builtin", "add_type", "fix_from_error"]
        }).to_string())], true),
    }
}

// ── add_operator recipe ──────────────────────────────────

fn recipe_add_operator() -> (Vec<ContentItem>, bool) {
    let idx = index::global();
    let mut steps = Vec::new();

    // Step 1: Find Token enum in token.rs
    let token_file = "lang/crates/synoema-lexer/src/token.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(token_file)) {
        if let Some(token_enum) = fi.enums.iter().find(|e| e.name == "Token") {
            let last_variant = token_enum.variants.last().map(|v| v.as_str()).unwrap_or("???");
            steps.push(json!({
                "step": 1,
                "file": token_file,
                "action": "Add variant to enum Token",
                "location": format!("After last variant '{}' near line {}", last_variant, token_enum.line),
                "template": "OpMyOp,  // \"myop\" — description",
                "existing_variants_count": token_enum.variants.len()
            }));
        }
    }

    // Step 2: Find scanner match block
    let scanner_file = "lang/crates/synoema-lexer/src/scanner.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(scanner_file)) {
        let scan_fn = fi.functions.iter().find(|f| f.name == "scan_operator" || f.name == "scan" || f.name.contains("scan"));
        let line_hint = scan_fn.map(|f| f.line).unwrap_or(0);
        steps.push(json!({
            "step": 2,
            "file": scanner_file,
            "action": "Add scan rule for new operator",
            "location": format!("In scanner function near line {}", line_hint),
            "template": "\"myop\" => Token::OpMyOp,"
        }));
    }

    // Step 3: Find parser precedence
    let parser_file = "lang/crates/synoema-parser/src/parser.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(parser_file)) {
        let prec_fn = fi.functions.iter().find(|f| f.name.contains("precedence") || f.name.contains("prec"));
        let line_hint = prec_fn.map(|f| f.line).unwrap_or(0);
        steps.push(json!({
            "step": 3,
            "file": parser_file,
            "action": "Add precedence level for new operator",
            "location": format!("In precedence function near line {}", line_hint),
            "template": "Token::OpMyOp => 6,"
        }));
    }

    let result = json!({
        "task": "add_operator",
        "steps": steps,
        "verify": ["cargo test -p synoema-lexer", "cargo test -p synoema-parser"],
        "warnings": ["BPE: verify new operator is 1 token via tools/bpe-verify/verify_bpe.py"]
    });

    (vec![ContentItem::text(result.to_string())], false)
}

// ── add_builtin recipe ───────────────────────────────────

fn recipe_add_builtin() -> (Vec<ContentItem>, bool) {
    let idx = index::global();
    let mut steps = Vec::new();

    // Step 1: eval.rs — find builtin dispatch
    let eval_file = "lang/crates/synoema-eval/src/eval.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(eval_file)) {
        let eval_fn = fi.functions.iter().find(|f| f.name == "eval_expr" || f.name.contains("builtin") || f.name.contains("apply"));
        let line_hint = eval_fn.map(|f| f.line).unwrap_or(0);
        steps.push(json!({
            "step": 1,
            "file": eval_file,
            "action": "Add builtin function to interpreter dispatch",
            "location": format!("In eval/apply function near line {}", line_hint),
            "template": "\"my_builtin\" => { /* implementation */ }"
        }));
    }

    // Step 2: runtime.rs — add FFI function for JIT
    let runtime_file = "lang/crates/synoema-codegen/src/runtime.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(runtime_file)) {
        let last_fn = fi.functions.last();
        let line_hint = last_fn.map(|f| f.line).unwrap_or(0);
        steps.push(json!({
            "step": 2,
            "file": runtime_file,
            "action": "Add extern \"C\" FFI function for JIT",
            "location": format!("After last function near line {}", line_hint),
            "template": "pub extern \"C\" fn rt_my_builtin(arg: i64) -> i64 { ... }"
        }));
    }

    // Step 3: compiler.rs — register FFI function
    let compiler_file = "lang/crates/synoema-codegen/src/compiler.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(compiler_file)) {
        let reg_fn = fi.functions.iter().find(|f| f.name.contains("register") || f.name.contains("builtin") || f.name.contains("import"));
        let line_hint = reg_fn.map(|f| f.line).unwrap_or(0);
        steps.push(json!({
            "step": 3,
            "file": compiler_file,
            "action": "Register FFI function in JIT compiler",
            "location": format!("In registration function near line {}", line_hint),
            "template": "self.register_ffi(\"my_builtin\", rt_my_builtin as *const u8, &sig);"
        }));
    }

    let result = json!({
        "task": "add_builtin",
        "steps": steps,
        "verify": ["cargo test -p synoema-eval", "cargo test -p synoema-codegen"],
        "warnings": ["Interpreter first, then JIT (RULES.md §3)"]
    });

    (vec![ContentItem::text(result.to_string())], false)
}

// ── add_type recipe ──────────────────────────────────────

fn recipe_add_type() -> (Vec<ContentItem>, bool) {
    let idx = index::global();
    let mut steps = Vec::new();

    // Step 1: types.rs — Type enum
    let types_file = "lang/crates/synoema-types/src/types.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(types_file)) {
        if let Some(type_enum) = fi.enums.iter().find(|e| e.name == "Type" || e.name == "Ty") {
            steps.push(json!({
                "step": 1,
                "file": types_file,
                "action": "Add variant to Type enum",
                "location": format!("In enum {} near line {}", type_enum.name, type_enum.line),
                "template": "TMyType,"
            }));
        }
    }

    // Step 2: infer.rs — unification
    let infer_file = "lang/crates/synoema-types/src/infer.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(infer_file)) {
        let unify_fn = fi.functions.iter().find(|f| f.name == "unify" || f.name.contains("unify"));
        let line_hint = unify_fn.map(|f| f.line).unwrap_or(0);
        steps.push(json!({
            "step": 2,
            "file": infer_file,
            "action": "Add unification rule for new type",
            "location": format!("In unify function near line {}", line_hint),
            "template": "(Type::TMyType, Type::TMyType) => Ok(()),"
        }));
    }

    // Step 3: core_ir.rs — desugar
    let core_file = "lang/crates/synoema-core/src/core_ir.rs";
    if let Some(fi) = idx.get_file(std::path::Path::new(core_file)) {
        if let Some(core_enum) = fi.enums.iter().find(|e| e.name.contains("Core") || e.name.contains("Expr")) {
            steps.push(json!({
                "step": 3,
                "file": core_file,
                "action": "Add CoreExpr variant if needed",
                "location": format!("In enum {} near line {}", core_enum.name, core_enum.line)
            }));
        }
    }

    // Step 4: eval.rs
    steps.push(json!({
        "step": 4,
        "file": "lang/crates/synoema-eval/src/eval.rs",
        "action": "Add evaluation rule for new type"
    }));

    // Step 5: codegen
    steps.push(json!({
        "step": 5,
        "file": "lang/crates/synoema-codegen/src/compiler.rs",
        "action": "Add JIT compilation for new type (after interpreter works)"
    }));

    let result = json!({
        "task": "add_type",
        "steps": steps,
        "verify": ["cargo test -p synoema-types", "cargo test -p synoema-eval", "cargo test -p synoema-codegen"],
        "warnings": ["Interpreter first, then JIT", "Update docs/llm/synoema.md"]
    });

    (vec![ContentItem::text(result.to_string())], false)
}

// ── fix_from_error recipe ────────────────────────────────

fn recipe_fix_from_error(file: &str, line: usize) -> (Vec<ContentItem>, bool) {
    if file.is_empty() || line == 0 {
        return (vec![ContentItem::text("fix_from_error requires: file and line parameters")], true);
    }

    let idx = index::global();
    let root = crate::resources::synoema_root();
    let abs = root.join(file);

    let content = match std::fs::read_to_string(&abs) {
        Ok(c) => c,
        Err(_) => return (vec![ContentItem::text(format!("cannot read: {file}"))], true),
    };

    let lines: Vec<&str> = content.lines().collect();
    if line == 0 || line > lines.len() {
        return (vec![ContentItem::text(format!("line {line} out of range"))], true);
    }

    // Find enclosing function
    let fi = idx.get_file(std::path::Path::new(file));
    let enclosing = fi.as_ref().and_then(|fi| {
        fi.functions.iter()
            .rfind(|f| f.line <= line)
    });

    let ctx_start = line.saturating_sub(5);
    let ctx_end = (line + 5).min(lines.len());
    let snippet: Vec<String> = (ctx_start..ctx_end).map(|i| {
        let marker = if i + 1 == line { ">>>" } else { "   " };
        format!("{marker} {:>4} | {}", i + 1, lines[i])
    }).collect();

    let mut steps = Vec::new();
    steps.push(json!({
        "step": 1,
        "action": "Locate error context",
        "file": file,
        "line": line,
        "enclosing_function": enclosing.map(|f| format!("{} {}{}", f.vis, f.name, f.sig)).unwrap_or_default(),
        "code_context": snippet.join("\n")
    }));
    steps.push(json!({
        "step": 2,
        "action": "Read the error message and understand the root cause"
    }));
    steps.push(json!({
        "step": 3,
        "action": "Apply fix at the identified location",
        "verify": format!("cargo test -p {}", guess_crate_from_path(file))
    }));

    let result = json!({
        "task": "fix_from_error",
        "steps": steps,
        "verify": [format!("cargo test -p {}", guess_crate_from_path(file))]
    });

    (vec![ContentItem::text(result.to_string())], false)
}

fn guess_crate_from_path(path: &str) -> &str {
    // Extract crate name from path like "lang/crates/synoema-eval/src/eval.rs"
    let parts: Vec<&str> = path.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "crates" && i + 1 < parts.len() {
            return parts[i + 1];
        }
    }
    "synoema"
}

// ── Tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_add_operator_returns_steps() {
        let (content, is_error) = recipe_add_operator();
        assert!(!is_error);
        let text = &content[0].text;
        let val: Value = serde_json::from_str(text).expect("valid JSON");
        assert_eq!(val["task"], "add_operator");
        let steps = val["steps"].as_array().expect("steps array");
        assert!(!steps.is_empty());
    }

    #[test]
    fn recipe_add_builtin_returns_steps() {
        let (content, is_error) = recipe_add_builtin();
        assert!(!is_error);
        let val: Value = serde_json::from_str(&content[0].text).expect("valid JSON");
        assert_eq!(val["task"], "add_builtin");
    }

    #[test]
    fn recipe_add_type_returns_steps() {
        let (content, is_error) = recipe_add_type();
        assert!(!is_error);
        let val: Value = serde_json::from_str(&content[0].text).expect("valid JSON");
        assert_eq!(val["task"], "add_type");
    }

    #[test]
    fn recipe_unknown_task() {
        let args = json!({"task": "nonexistent"});
        let (content, is_error) = call(&args);
        assert!(is_error);
        assert!(content[0].text.contains("unknown recipe"));
    }

    #[test]
    fn recipe_fix_from_error_missing_params() {
        let (_content, is_error) = recipe_fix_from_error("", 0);
        assert!(is_error);
    }

    #[test]
    fn guess_crate_from_path_works() {
        assert_eq!(guess_crate_from_path("lang/crates/synoema-eval/src/eval.rs"), "synoema-eval");
        assert_eq!(guess_crate_from_path("some/other/path.rs"), "synoema");
    }
}
