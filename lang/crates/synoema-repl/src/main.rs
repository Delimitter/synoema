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
use synoema_diagnostic::{Diagnostic, render_human, render_json};

// ── Error output ─────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum ErrorFormat { Human, Json }

fn print_diag(diag: &Diagnostic, source: Option<&str>, format: ErrorFormat) {
    match format {
        ErrorFormat::Human => eprint!("{}", render_human(diag, source)),
        ErrorFormat::Json => eprintln!("{}", render_json(diag)),
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
        Some("--help") | Some("-h") => {
            println!("Synoema v0.1 — A BPE-aligned programming language for LLM code generation");
            println!();
            println!("Usage:");
            println!("  synoema              Start interactive REPL");
            println!("  synoema run <file>   Interpret a source file");
            println!("  synoema jit <file>   JIT-compile and run via Cranelift (native speed)");
            println!("  synoema eval <expr>  Evaluate an expression");
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

    match synoema_eval::eval_main(&source) {
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

    // Type check first (use the same precise code dispatch as the interpreter path)
    if let Err(e) = synoema_types::typecheck(&source) {
        let diag = synoema_eval::type_err_to_diagnostic(e);
        print_diag(&diag, Some(&source), format);
        std::process::exit(1);
    }

    // JIT compile and run via Cranelift
    match synoema_codegen::compile_and_display(&source) {
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
