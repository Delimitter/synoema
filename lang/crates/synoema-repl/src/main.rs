// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Synoema CLI — compiler, interpreter, and REPL
//!
//! Usage:
//!   synoema              — start REPL
//!   synoema run file.sno  — interpret a file
//!   synoema jit file.sno  — JIT-compile and run via Cranelift
//!   synoema eval "expr"  — evaluate an expression
//!
//! Error format:
//!   --errors human       — human-readable with source snippets (default)
//!   --errors json        — JSON for LLM/tool consumption

use std::io::{self, Write, BufRead};
use synoema_diagnostic::{Diagnostic, render_human, render_json, enrich_diagnostic};

// ── Error output ─────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum ErrorFormat { Human, Json }

fn print_diag(diag: &Diagnostic, source: Option<&str>, format: ErrorFormat) {
    let mut diag = diag.clone();
    enrich_diagnostic(&mut diag);
    match format {
        ErrorFormat::Human => eprint!("{}", render_human(&diag, source)),
        ErrorFormat::Json => eprintln!("{}", render_json(&diag)),
    }
}

// ── CLI ──────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse --errors flag anywhere in args
    let format = if args.iter().any(|a| a == "--errors" || a == "--error-format") {
        let pos = args.iter().position(|a| a == "--errors" || a == "--error-format");
        match pos.and_then(|i| args.get(i + 1)).map(|s| s.as_str()) {
            Some("json") => ErrorFormat::Json,
            _ => ErrorFormat::Human,
        }
    } else {
        ErrorFormat::Human
    };

    // Filter out --errors <val> from positional args
    let positional: Vec<&str> = {
        let mut result = Vec::new();
        let mut skip_next = false;
        for arg in args.iter().skip(1) {
            if skip_next { skip_next = false; continue; }
            if arg == "--errors" || arg == "--error-format" { skip_next = true; continue; }
            result.push(arg.as_str());
        }
        result
    };

    match positional.first().copied() {
        Some("run") => {
            let path = positional.get(1).unwrap_or_else(|| {
                eprintln!("Usage: synoema run <file.sno>");
                std::process::exit(1);
            });
            run_file(path, format);
        }
        Some("jit") => {
            let path = positional.get(1).unwrap_or_else(|| {
                eprintln!("Usage: synoema jit <file.sno>");
                std::process::exit(1);
            });
            jit_file(path, format);
        }
        Some("eval") => {
            let expr = positional.get(1).unwrap_or_else(|| {
                eprintln!("Usage: synoema eval \"<expression>\"");
                std::process::exit(1);
            });
            eval_one(expr, format);
        }
        Some("doc") => {
            let path = positional.get(1).unwrap_or_else(|| {
                eprintln!("Usage: synoema doc <file.sno | directory>");
                std::process::exit(1);
            });
            let fmt = if positional.iter().any(|a| *a == "--format") {
                positional.iter().position(|a| *a == "--format")
                    .and_then(|i| positional.get(i + 1))
                    .copied()
                    .unwrap_or("md")
            } else { "md" };
            generate_docs(path, fmt);
        }
        Some("test") => {
            let path = positional.get(1).unwrap_or_else(|| {
                eprintln!("Usage: synoema test <file.sno | directory> [--filter <str>]");
                std::process::exit(1);
            });
            let filter = positional.iter().position(|a| *a == "--filter")
                .and_then(|i| positional.get(i + 1))
                .map(|s| s.to_string());
            let ok = run_all_tests(path, format, filter.as_deref());
            if !ok { std::process::exit(1); }
        }
        Some("--help") | Some("-h") => {
            println!("Synoema v0.1 — A BPE-aligned programming language for LLM code generation");
            println!();
            println!("Usage:");
            println!("  synoema              Start interactive REPL");
            println!("  synoema run <file>   Interpret a source file");
            println!("  synoema jit <file>   JIT-compile and run via Cranelift (native speed)");
            println!("  synoema eval <expr>  Evaluate an expression");
            println!("  synoema test <path>  Run tests (doctests + test declarations)");
            println!("  synoema doc <path>   Generate documentation (Markdown)");
            println!();
            println!("Options:");
            println!("  --errors human       Human-readable errors with source snippets (default)");
            println!("  --errors json        JSON errors for LLM/tool consumption");
            println!();
            println!("REPL commands:");
            println!("  :type <expr>       Show inferred type");
            println!("  :load <file>       Load a source file");
            println!("  :quit              Exit REPL");
        }
        _ => repl(format),
    }
}

fn run_file(path: &str, format: ErrorFormat) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let base_dir = std::path::Path::new(path).parent().unwrap_or(std::path::Path::new("."));
    match synoema_eval::eval_main_with_base_dir(&source, Some(base_dir)) {
        Ok((val, output)) => {
            for line in &output {
                println!("{}", line);
            }
            println!("{}", val);
        }
        Err(diag) => {
            print_diag(&diag, Some(&source), format);
            std::process::exit(1);
        }
    }
}

fn jit_file(path: &str, format: ErrorFormat) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let base_dir = std::path::Path::new(path).parent().unwrap_or(std::path::Path::new("."));

    // Parse and resolve imports, then type-check the resolved program
    let program = match synoema_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            let diag = Diagnostic::error(synoema_diagnostic::codes::PARSE_UNEXPECTED_TOKEN, format!("{}", e));
            print_diag(&diag, Some(&source), format);
            std::process::exit(1);
        }
    };
    let program = match synoema_parser::resolve_imports(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            let code = match e.code {
                synoema_parser::ImportErrorCode::Cycle => synoema_diagnostic::codes::IMPORT_CYCLE,
                synoema_parser::ImportErrorCode::NotFound => synoema_diagnostic::codes::IMPORT_NOT_FOUND,
                synoema_parser::ImportErrorCode::ParseError => synoema_diagnostic::codes::PARSE_UNEXPECTED_TOKEN,
            };
            let diag = Diagnostic::error(code, e.message).with_span(e.span);
            print_diag(&diag, Some(&source), format);
            std::process::exit(1);
        }
    };
    if let Err(e) = synoema_types::typecheck_program(&program) {
        let diag = synoema_eval::type_err_to_diagnostic(e);
        print_diag(&diag, Some(&source), format);
        std::process::exit(1);
    }

    // JIT compile and run via Cranelift
    match synoema_codegen::compile_and_display_with_base_dir(&source, Some(base_dir)) {
        Ok(result) => println!("{}", result),
        Err(diag) => {
            print_diag(&diag, Some(&source), format);
            std::process::exit(1);
        }
    }
}

fn eval_one(expr: &str, format: ErrorFormat) {
    match synoema_eval::eval_expr(expr) {
        Ok(val) => println!("{}", val),
        Err(diag) => print_diag(&diag, Some(expr), format),
    }
}

fn repl(format: ErrorFormat) {
    println!("Synoema v0.1 — Type :help for commands, :quit to exit");
    println!();

    let stdin = io::stdin();
    let mut env_source = String::new(); // accumulate definitions

    loop {
        print!("synoema> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 {
            println!();
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // ── REPL commands ────────────────────────
        if trimmed == ":quit" || trimmed == ":q" {
            break;
        }

        if trimmed == ":help" || trimmed == ":h" {
            println!("Commands:");
            println!("  :type <expr>   Show inferred type of expression");
            println!("  :load <file>   Load definitions from file");
            println!("  :reset         Clear all definitions");
            println!("  :env           Show current definitions");
            println!("  :quit          Exit");
            println!();
            println!("Enter expressions to evaluate, or definitions to add.");
            println!("  42 + 1                    -- evaluate expression");
            println!("  fac 0 = 1                 -- define function");
            println!("  fac n = n * fac (n - 1)   -- add equation");
            continue;
        }

        if let Some(expr) = trimmed.strip_prefix(":type ").or_else(|| trimmed.strip_prefix(":t ")) {
            type_of(expr, &env_source, format);
            continue;
        }

        if let Some(path) = trimmed.strip_prefix(":load ").or_else(|| trimmed.strip_prefix(":l ")) {
            match std::fs::read_to_string(path.trim()) {
                Ok(src) => {
                    env_source = src;
                    println!("Loaded {}", path.trim());
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            continue;
        }

        if trimmed == ":reset" {
            env_source.clear();
            println!("Environment cleared.");
            continue;
        }

        if trimmed == ":env" {
            if env_source.is_empty() {
                println!("(empty)");
            } else {
                println!("{}", env_source);
            }
            continue;
        }

        // ── Multi-line input (read indented continuation lines) ──
        let mut input = line.clone();
        while input.trim_end().ends_with('=') || {
            // Peek at next line — if it starts with spaces, it's continuation
            let _peek = String::new();
            // Can't easily peek with stdin, so check if current input
            // looks like it needs continuation
            false
        } {
            print!("    > ");
            io::stdout().flush().unwrap();
            let mut cont = String::new();
            if stdin.lock().read_line(&mut cont).unwrap() == 0 {
                break;
            }
            input.push_str(&cont);
        }

        let trimmed = input.trim();

        // ── Try as definition first ──────────────
        let is_def = trimmed.contains('=') && !trimmed.starts_with('?')
            && (trimmed.starts_with(|c: char| c.is_lowercase() || c.is_uppercase()));

        if is_def {
            // Try parsing as a definition
            let test_source = format!("{}\n{}", env_source, trimmed);
            match synoema_parser::parse(&test_source) {
                Ok(_) => {
                    env_source = test_source;
                    // Show the type of the defined name
                    if let Some(name) = trimmed.split_whitespace().next() {
                        let name_clean = name.trim();
                        match synoema_types::typecheck(&env_source) {
                            Ok(tenv) => {
                                if let Some(scheme) = tenv.lookup(name_clean) {
                                    println!("{} : {}", name_clean, scheme.ty);
                                } else {
                                    println!("defined");
                                }
                            }
                            Err(_) => println!("defined"),
                        }
                    }
                    continue;
                }
                Err(_) => {
                    // Fall through to try as expression
                }
            }
        }

        // ── Try as expression ────────────────────
        let expr_source = format!("{}\n__repl_expr = {}", env_source, trimmed);
        match synoema_eval::eval_main(&expr_source) {
            Ok((val, output)) => {
                for line in &output {
                    println!("{}", line);
                }
                // Also show type
                match synoema_types::typecheck(&expr_source) {
                    Ok(tenv) => {
                        if let Some(scheme) = tenv.lookup("__repl_expr") {
                            println!("{} : {}", val, scheme.ty);
                        } else {
                            println!("{}", val);
                        }
                    }
                    Err(_) => println!("{}", val),
                }
            }
            Err(diag) => print_diag(&diag, Some(&expr_source), format),
        }
    }
}

// ── Doctests ─────────────────────────────────────────────

/// Extract `--- example:` lines from parsed program.
struct Doctest {
    file: String,
    line: u32,
    expr_str: String,
    expected: Option<String>,
}

fn extract_doctests(path: &str, program: &synoema_parser::Program) -> Vec<Doctest> {
    let mut tests = Vec::new();
    let extract = |doc: &[String], tests: &mut Vec<Doctest>| {
        for (i, line) in doc.iter().enumerate() {
            if let Some(rest) = line.strip_prefix("example:") {
                let rest = rest.trim();
                // Split on top-level == (not inside parens)
                let (expr, expected) = split_example(rest);
                tests.push(Doctest {
                    file: path.to_string(),
                    line: i as u32 + 1,
                    expr_str: expr,
                    expected,
                });
            }
        }
    };

    for decl in &program.decls {
        match decl {
            synoema_parser::Decl::Func { doc, .. } => extract(doc, &mut tests),
            synoema_parser::Decl::TypeDef { doc, .. } => extract(doc, &mut tests),
            synoema_parser::Decl::TraitDecl { doc, .. } => extract(doc, &mut tests),
            _ => {}
        }
    }
    for module in &program.modules {
        extract(&module.doc, &mut tests);
        for decl in &module.body {
            match decl {
                synoema_parser::Decl::Func { doc, .. } => extract(doc, &mut tests),
                synoema_parser::Decl::TypeDef { doc, .. } => extract(doc, &mut tests),
                synoema_parser::Decl::TraitDecl { doc, .. } => extract(doc, &mut tests),
                _ => {}
            }
        }
    }
    tests
}

/// Split "expr == expected" on top-level == (respecting parens).
fn split_example(s: &str) -> (String, Option<String>) {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'=' if depth == 0 && i + 1 < bytes.len() && bytes[i + 1] == b'=' => {
                // Check it's not part of a longer operator (e.g., ===)
                let before_ok = i == 0 || bytes[i - 1] != b'=';
                let after_ok = i + 2 >= bytes.len() || bytes[i + 2] != b'=';
                if before_ok && after_ok {
                    let lhs = s[..i].trim().to_string();
                    let rhs = s[i + 2..].trim().to_string();
                    return (lhs, Some(rhs));
                }
            }
            _ => {}
        }
        i += 1;
    }
    (s.to_string(), None)
}

fn collect_sno_files(dir: &str, out: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_sno_files(path.to_str().unwrap_or(""), out);
            } else if path.extension().map(|e| e == "sno").unwrap_or(false) {
                out.push(path.to_string_lossy().to_string());
            }
        }
    }
}

// ── Test Declarations Runner ────────────────────────────

/// Simple LCG random number generator (no external crate needed).
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self { Self(seed) }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn gen_range(&mut self, lo: i64, hi: i64) -> i64 {
        let range = (hi - lo + 1) as u64;
        lo + (self.next() % range) as i64
    }
    fn gen_bool(&mut self) -> bool { self.next() % 2 == 0 }
    fn gen_ascii(&mut self, max_len: usize) -> String {
        let len = (self.next() % (max_len as u64 + 1)) as usize;
        (0..len).map(|_| (b'a' + (self.next() % 26) as u8) as char).collect()
    }
}

/// Generate a random Value for a given inferred type string.
fn generate_value(ty_str: &str, rng: &mut Lcg) -> synoema_eval::Value {
    use synoema_eval::Value;
    let ty_str = ty_str.trim();
    match ty_str {
        "Int" => Value::Int(rng.gen_range(-100, 100)),
        "Bool" => Value::Bool(rng.gen_bool()),
        "String" => Value::Str(rng.gen_ascii(8)),
        _ if ty_str.starts_with("List ") => {
            let inner = &ty_str[5..];
            let len = rng.gen_range(0, 10) as usize;
            let elems: Vec<Value> = (0..len).map(|_| generate_value(inner, rng)).collect();
            Value::List(elems)
        }
        _ => Value::Int(rng.gen_range(-100, 100)), // fallback to Int
    }
}

/// Run test declarations from a single file. Returns (passed, failed).
fn run_test_decls_file(
    path: &str,
    source: &str,
    program: &synoema_parser::Program,
    format: ErrorFormat,
    filter: Option<&str>,
) -> (usize, usize) {
    let mut passed = 0usize;
    let mut failed = 0usize;

    // Collect test declarations
    let tests: Vec<_> = program.decls.iter().filter_map(|d| {
        if let synoema_parser::Decl::Test { name, body, span } = d {
            if let Some(f) = filter {
                if !name.contains(f) { return None; }
            }
            Some((name.clone(), body.clone(), *span))
        } else { None }
    }).collect();

    for (name, body, _span) in &tests {
        // Check if the body is a Prop expression
        if let synoema_parser::ExprKind::Prop(vars, prop_body) = &body.kind {
            // Property-based test: generate 100 random inputs
            let (p, f) = run_property_test(path, source, &name, vars, prop_body, format);
            passed += p;
            failed += f;
        } else {
            // Simple test: evaluate body, expect Bool(true)
            let test_source = format!("{}\n__test_result = {}", source, expr_to_source(&body));
            match synoema_eval::eval_main(&test_source) {
                Ok((val, _)) => {
                    match &val {
                        synoema_eval::Value::Bool(true) => {
                            passed += 1;
                        }
                        synoema_eval::Value::Bool(false) => {
                            failed += 1;
                            eprintln!("  FAIL: {} — test \"{}\" returned false", path, name);
                        }
                        other => {
                            failed += 1;
                            eprintln!("  FAIL: {} — test \"{}\" returned {} (expected Bool)", path, name, other);
                        }
                    }
                }
                Err(diag) => {
                    failed += 1;
                    eprintln!("  FAIL: {} — test \"{}\" error:", path, name);
                    print_diag(&diag, Some(&test_source), format);
                }
            }
        }
    }

    (passed, failed)
}

/// Convert an Expr back to source text (minimal, for eval embedding).
fn expr_to_source(expr: &synoema_parser::Expr) -> String {
    use synoema_parser::ExprKind;
    match &expr.kind {
        ExprKind::Lit(lit) => match lit {
            synoema_parser::Lit::Int(n) => n.to_string(),
            synoema_parser::Lit::Float(f) => f.to_string(),
            synoema_parser::Lit::Str(s) => format!("\"{}\"", s),
            synoema_parser::Lit::Char(c) => format!("'{}'", c),
            synoema_parser::Lit::Bool(b) => b.to_string(),
            synoema_parser::Lit::Unit => "()".to_string(),
        },
        ExprKind::Var(name) => name.clone(),
        ExprKind::Con(name) => name.clone(),
        ExprKind::App(f, x) => format!("({} {})", expr_to_source(f), expr_to_source(x)),
        ExprKind::BinOp(op, l, r) => format!("({} {} {})", expr_to_source(l), op.symbol(), expr_to_source(r)),
        ExprKind::Neg(e) => format!("(- {})", expr_to_source(e)),
        ExprKind::Paren(e) => format!("({})", expr_to_source(e)),
        ExprKind::List(es) => {
            let inner: Vec<_> = es.iter().map(|e| expr_to_source(e)).collect();
            format!("[{}]", inner.join(" "))
        }
        ExprKind::Cond(c, t, e) => format!("(? {} -> {} : {})", expr_to_source(c), expr_to_source(t), expr_to_source(e)),
        ExprKind::When(body, cond) => format!("({} when {})", expr_to_source(body), expr_to_source(cond)),
        _ => format!("({{complex expr}})"),
    }
}

/// Run a property-based test with 100 random inputs.
fn run_property_test(
    path: &str,
    source: &str,
    name: &str,
    vars: &[String],
    body: &synoema_parser::Expr,
    format: ErrorFormat,
) -> (usize, usize) {
    // First, infer types of the prop variables using typecheck
    let var_types = match infer_prop_var_types(source, vars, body) {
        Some(types) => types,
        None => {
            // Fallback: assume all vars are Int
            vars.iter().map(|_| "Int".to_string()).collect()
        }
    };

    let mut rng = Lcg::new(42); // deterministic seed for reproducibility
    let num_trials = 100usize;

    for trial in 0..num_trials {
        // Generate random values for each variable
        let vals: Vec<_> = var_types.iter().map(|ty| generate_value(ty, &mut rng)).collect();

        // Build source with unique variable assignments to avoid name conflicts
        let mut assignments = String::new();
        let mut body_str = expr_to_source(body);
        for (var, val) in vars.iter().zip(&vals) {
            let unique = format!("__pv_{}", var);
            assignments.push_str(&format!("\n{} = {}", unique, value_to_source(val)));
            // Word-boundary replacement: only replace standalone identifiers
            body_str = replace_word(&body_str, var, &unique);
        }

        let test_source = format!("{}{}\n__prop_test = {}", source, assignments, body_str);
        match synoema_eval::eval_main(&test_source) {
            Ok((val, _)) => {
                match &val {
                    synoema_eval::Value::Bool(true) => { /* pass */ }
                    synoema_eval::Value::Bool(false) => {
                        // Counterexample found
                        let bindings: Vec<_> = vars.iter().zip(&vals)
                            .map(|(v, val)| format!("{} = {}", v, val))
                            .collect();
                        eprintln!("  FAIL: {} — test \"{}\" counterexample (trial {}):", path, name, trial + 1);
                        eprintln!("    {}", bindings.join(", "));
                        return (0, 1);
                    }
                    _ => {
                        eprintln!("  FAIL: {} — test \"{}\" returned non-Bool: {}", path, name, val);
                        return (0, 1);
                    }
                }
            }
            Err(diag) => {
                eprintln!("  FAIL: {} — test \"{}\" error at trial {}:", path, name, trial + 1);
                print_diag(&diag, Some(&test_source), format);
                return (0, 1);
            }
        }
    }

    (1, 0)
}

/// Infer types of prop variables by typechecking wrapper functions.
fn infer_prop_var_types(source: &str, vars: &[String], body: &synoema_parser::Expr) -> Option<Vec<String>> {
    // Strategy: create a function `__prop_fn v1 v2 ... = body` and infer its type.
    // The function type `T1 -> T2 -> ... -> Bool` reveals each variable's type.
    let fn_def = format!("{}\n__prop_fn {} = {}", source, vars.join(" "), expr_to_source(body));
    match synoema_types::typecheck(&fn_def) {
        Ok(env) => {
            if let Some(scheme) = env.lookup("__prop_fn") {
                let mut types = Vec::new();
                let mut ty = scheme.ty.clone();
                for _ in 0..vars.len() {
                    if let synoema_types::Type::Arrow(param, ret) = ty {
                        types.push(format!("{}", param));
                        ty = *ret;
                    } else {
                        types.push("Int".to_string());
                        break;
                    }
                }
                if types.len() == vars.len() {
                    return Some(types);
                }
            }
            None
        }
        Err(_) => None,
    }
}

/// Convert a Value to Synoema source for embedding in test source.
fn value_to_source(val: &synoema_eval::Value) -> String {
    use synoema_eval::Value;
    match val {
        Value::Int(n) => if *n < 0 { format!("(0 - {})", n.unsigned_abs()) } else { n.to_string() },
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => format!("\"{}\"", s),
        Value::List(elems) => {
            let inner: Vec<_> = elems.iter().map(|v| value_to_source(v)).collect();
            format!("[{}]", inner.join(" "))
        }
        _ => "()".to_string(),
    }
}

/// Replace a word (identifier) in source, respecting word boundaries.
fn replace_word(source: &str, word: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let word_bytes = word.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + word_bytes.len() <= bytes.len() && &bytes[i..i+word_bytes.len()] == word_bytes {
            let before_ok = i == 0 || !bytes[i-1].is_ascii_alphanumeric() && bytes[i-1] != b'_';
            let after_ok = i + word_bytes.len() >= bytes.len() || !bytes[i+word_bytes.len()].is_ascii_alphanumeric() && bytes[i+word_bytes.len()] != b'_';
            if before_ok && after_ok {
                result.push_str(replacement);
                i += word_bytes.len();
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Unified test runner: runs both doctests and test declarations.
fn run_all_tests(path: &str, format: ErrorFormat, filter: Option<&str>) -> bool {
    let meta = std::fs::metadata(path);
    let files: Vec<String> = if meta.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
        let mut sno_files = Vec::new();
        collect_sno_files(path, &mut sno_files);
        sno_files.sort();
        sno_files
    } else {
        vec![path.to_string()]
    };

    let mut total_doc_passed = 0usize;
    let mut total_doc_failed = 0usize;
    let mut total_test_passed = 0usize;
    let mut total_test_failed = 0usize;

    for file in &files {
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  Error reading '{}': {}", file, e);
                total_doc_failed += 1;
                continue;
            }
        };

        let program = match synoema_parser::parse(&source) {
            Ok(p) => p,
            Err(e) => {
                let diag = synoema_diagnostic::Diagnostic::error(
                    synoema_diagnostic::codes::PARSE_UNEXPECTED_TOKEN,
                    format!("{}", e),
                );
                print_diag(&diag, Some(&source), format);
                total_doc_failed += 1;
                continue;
            }
        };

        // Run doctests (if no filter, or filter matches)
        let doctests = extract_doctests(file, &program);
        for dt in &doctests {
            if let Some(f) = filter {
                if !dt.expr_str.contains(f) { continue; }
            }
            let test_source = format!("{}\n__doctest_val = {}", source, dt.expr_str);
            match synoema_eval::eval_main(&test_source) {
                Ok((val, _)) => {
                    if let Some(ref expected_str) = dt.expected {
                        let exp_source = format!("{}\n__doctest_exp = {}", source, expected_str);
                        match synoema_eval::eval_main(&exp_source) {
                            Ok((exp_val, _)) => {
                                if val.to_string() == exp_val.to_string() {
                                    total_doc_passed += 1;
                                } else {
                                    total_doc_failed += 1;
                                    eprintln!("  FAIL: {}:{} — example: {} == {}", dt.file, dt.line, dt.expr_str, expected_str);
                                    eprintln!("    Left:  {}", val);
                                    eprintln!("    Right: {}", exp_val);
                                }
                            }
                            Err(diag) => {
                                total_doc_failed += 1;
                                eprintln!("  FAIL: {}:{} — error evaluating expected: {}", dt.file, dt.line, expected_str);
                                print_diag(&diag, Some(&exp_source), format);
                            }
                        }
                    } else {
                        total_doc_passed += 1;
                    }
                }
                Err(diag) => {
                    total_doc_failed += 1;
                    eprintln!("  FAIL: {}:{} — error evaluating: {}", dt.file, dt.line, dt.expr_str);
                    print_diag(&diag, Some(&test_source), format);
                }
            }
        }

        // Run test declarations
        let (tp, tf) = run_test_decls_file(file, &source, &program, format, filter);
        total_test_passed += tp;
        total_test_failed += tf;

        let doc_count = doctests.len();
        let test_count = tp + tf;
        if doc_count + test_count > 0 {
            let status = if total_doc_failed + tf == 0 { "ok" } else { "FAILED" };
            if doc_count > 0 && test_count > 0 {
                eprintln!("  {} — {} doctests, {} tests ({})", file, doc_count, test_count, status);
            } else if doc_count > 0 {
                eprintln!("  {} — {} doctests ({})", file, doc_count, status);
            } else {
                eprintln!("  {} — {} tests ({})", file, test_count, status);
            }
        }
    }

    let total_passed = total_doc_passed + total_test_passed;
    let total_failed = total_doc_failed + total_test_failed;

    if total_passed + total_failed == 0 {
        eprintln!("  No tests found.");
    } else {
        let status = if total_failed == 0 { "ok" } else { "FAILED" };
        eprintln!("  Total: {}/{} tests {}", total_passed, total_passed + total_failed, status);
    }

    total_failed == 0
}

// ── Doc Generation ───────────────────────────────────────

struct GuideMeta {
    title: Option<String>,
    order: f64,
}

fn parse_guide_meta(doc: &[String]) -> GuideMeta {
    let mut meta = GuideMeta { title: None, order: 0.0 };
    for line in doc {
        if let Some(rest) = line.strip_prefix("guide:") {
            meta.title = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("order:") {
            meta.order = rest.trim().parse().unwrap_or(0.0);
        }
    }
    meta
}

fn generate_doc_file(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            return;
        }
    };

    let program = match synoema_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error in '{}': {}", path, e);
            return;
        }
    };

    // Check for guide metadata in top-level (first module or program-level docs)
    let top_doc: Vec<String> = program.modules.first()
        .map(|m| m.doc.clone())
        .unwrap_or_default();
    let meta = parse_guide_meta(&top_doc);

    if let Some(ref title) = meta.title {
        println!("# {}", title);
        println!();
    } else {
        println!("# {}", path);
        println!();
    }

    // Render: interleave doc-comments as prose and declarations as code
    let render_decl_docs = |doc: &[String], name: &str| {
        let prose: Vec<&String> = doc.iter()
            .filter(|l| !l.starts_with("guide:") && !l.starts_with("order:") && !l.starts_with("requires:"))
            .collect();
        for line in &prose {
            if line.starts_with("# ") || line.starts_with("## ") {
                println!("{}", line);
            } else if line.starts_with("example:") {
                let rest = line.strip_prefix("example:").unwrap().trim();
                println!("```synoema");
                println!("-- example:");
                println!("{}", rest);
                println!("```");
            } else {
                println!("{}", line);
            }
        }
        if !prose.is_empty() { println!(); }
        let _ = name;
    };

    // Render modules
    for module in &program.modules {
        let mod_prose: Vec<&String> = module.doc.iter()
            .filter(|l| !l.starts_with("guide:") && !l.starts_with("order:") && !l.starts_with("requires:"))
            .collect();
        for line in &mod_prose {
            println!("{}", line);
        }
        if !mod_prose.is_empty() { println!(); }

        for decl in &module.body {
            match decl {
                synoema_parser::Decl::Func { doc, name, .. } => render_decl_docs(doc, name),
                synoema_parser::Decl::TypeDef { doc, name, .. } => render_decl_docs(doc, name),
                synoema_parser::Decl::TraitDecl { doc, name, .. } => render_decl_docs(doc, name),
                _ => {}
            }
        }
    }

    // Render top-level decls
    for decl in &program.decls {
        match decl {
            synoema_parser::Decl::Func { doc, name, .. } => render_decl_docs(doc, name),
            synoema_parser::Decl::TypeDef { doc, name, .. } => render_decl_docs(doc, name),
            synoema_parser::Decl::TraitDecl { doc, name, .. } => render_decl_docs(doc, name),
            _ => {}
        }
    }
}

fn generate_docs(path: &str, _fmt: &str) {
    let meta = std::fs::metadata(path);
    if meta.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
        let mut files = Vec::new();
        collect_sno_files(path, &mut files);
        files.sort();
        for file in &files {
            generate_doc_file(file);
            println!("---");
            println!();
        }
    } else {
        generate_doc_file(path);
    }
}

fn type_of(expr: &str, env_source: &str, format: ErrorFormat) {
    let source = format!("{}\n__type_query = {}", env_source, expr.trim());
    match synoema_types::typecheck(&source) {
        Ok(tenv) => {
            if let Some(scheme) = tenv.lookup("__type_query") {
                println!("{}", scheme.ty);
            } else {
                eprintln!("Could not infer type");
            }
        }
        Err(e) => {
            let diag = Diagnostic::error(
                synoema_diagnostic::codes::TYPE_OTHER,
                format!("{}", e),
            ).maybe_span(e.span);
            print_diag(&diag, Some(&source), format);
        }
    }
}
