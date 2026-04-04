// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Synoema CLI — compiler, interpreter, and REPL
//!
//! Usage:
//!   synoema              — start REPL
//!   synoema init [name]  — scaffold a new project
//!   synoema run file.sno  — interpret a file
//!   synoema jit file.sno  — JIT-compile and run via Cranelift
//!   synoema eval "expr"  — evaluate an expression
//!
//! Error format:
//!   --errors human       — human-readable with source snippets (default)
//!   --errors json        — JSON for LLM/tool consumption

use std::io::{self, Write, BufRead};
use synoema_diagnostic::{Diagnostic, render_human, render_json, enrich_diagnostic};

mod fmt;

// ── Project init templates ────────────────────────────────

const TMPL_MAIN: &str = include_str!("../../../templates/main.sno.tmpl");
const TMPL_TEST: &str = include_str!("../../../templates/test.sno.tmpl");
const TMPL_PROJECT: &str = include_str!("../../../templates/project.sno.tmpl");
const TMPL_AGENTS: &str = include_str!("../../../templates/AGENTS.md.tmpl");
const TMPL_CLAUDE: &str = include_str!("../../../templates/CLAUDE.md.tmpl");
const TMPL_GITIGNORE: &str = include_str!("../../../templates/gitignore.tmpl");

// Tool-specific agent configs
const TMPL_CURSORRULES: &str = include_str!("../../../templates/cursorrules.tmpl");
const TMPL_COPILOT_INSTRUCTIONS: &str = include_str!("../../../templates/copilot-instructions.md.tmpl");
const TMPL_WINDSURFRULES: &str = include_str!("../../../templates/windsurfrules.tmpl");
const TMPL_CLINERULES: &str = include_str!("../../../templates/clinerules.tmpl");

// MCP config templates

// LLM documentation (embedded into generated projects)
const LLM_SYNOEMA_REF: &str = include_str!("../../../../docs/llm/synoema.md");
const LLM_STDLIB_REF: &str = include_str!("../../../../docs/llm/stdlib.md");

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

    // Parse `--` separator: everything after `--` becomes script_args
    let dash_dash = positional.iter().position(|a| *a == "--");
    let script_args: Vec<String> = dash_dash
        .map(|i| positional[i + 1..].iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();
    let positional = dash_dash.map(|i| &positional[..i]).unwrap_or(&positional);

    match positional.first().copied() {
        Some("run") => {
            let path = positional.get(1).unwrap_or_else(|| {
                eprintln!("Usage: synoema run <file.sno>");
                std::process::exit(1);
            });
            run_file(path, format, script_args);
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
        Some("init") => {
            let force = positional.iter().any(|a| *a == "--force");
            let no_git = positional.iter().any(|a| *a == "--no-git");
            let mcp_binary = positional.iter().any(|a| *a == "--mcp-binary");
            let ai_target = positional.iter().position(|a| *a == "--ai")
                .and_then(|i| positional.get(i + 1))
                .copied();
            // Name arg: first positional after "init" that isn't a flag
            let name_arg = positional.iter().skip(1)
                .find(|a| !a.starts_with('-') && {
                    // Skip the value after --ai
                    let ai_pos = positional.iter().position(|x| *x == "--ai");
                    let is_ai_val = ai_pos.map(|p| positional.get(p + 1) == Some(a)).unwrap_or(false);
                    !is_ai_val
                })
                .copied();
            init_project(name_arg, force, no_git, ai_target, mcp_binary);
        }
        Some("mcp-install") => {
            let prefix = positional.iter().position(|a| *a == "--prefix")
                .and_then(|i| positional.get(i + 1))
                .copied();
            let from_source = positional.iter().any(|a| *a == "--from-source");
            mcp_install(prefix, from_source);
        }
        Some("mcp-update") => {
            mcp_update();
        }
        Some("fmt") => {
            let path = positional.get(1).unwrap_or_else(|| {
                eprintln!("Usage: synoema fmt <file.sno | directory> [--check]");
                std::process::exit(1);
            });
            let check = positional.iter().any(|a| *a == "--check");
            fmt_command(path, check, format);
        }
        Some("build") => {
            let path = positional.get(1).unwrap_or_else(|| {
                eprintln!("Usage: synoema build [OPTIONS] <file.sno>");
                std::process::exit(1);
            });
            let output = positional.iter().position(|a| *a == "-o" || *a == "--output")
                .and_then(|i| positional.get(i + 1))
                .copied();
            let check_only = positional.iter().any(|a| *a == "--check");
            let verbose = positional.iter().any(|a| *a == "-v" || *a == "--verbose");
            build_file(path, output, check_only, verbose, format);
        }
        Some("--help") | Some("-h") => {
            println!("Synoema v0.1 — A BPE-aligned programming language for LLM code generation");
            println!();
            println!("Usage:");
            println!("  synoema                    Start interactive REPL");
            println!("  synoema init [name]        Scaffold a new project");
            println!("  synoema run <file>         Interpret a source file");
            println!("  synoema jit <file>         JIT-compile and run via Cranelift (native speed)");
            println!("  synoema eval <expr>        Evaluate an expression");
            println!("  synoema fmt <path>         Format source files (in-place or --check)");
            println!("  synoema build <file>       Build and compile to bytecode");
            println!("  synoema test <path>        Run tests (doctests + test declarations)");
            println!("  synoema doc <path>         Generate documentation (Markdown)");
            println!("  synoema mcp-install        Install MCP server binary (no Node.js needed)");
            println!("  synoema mcp-update         Update MCP server to latest version");
            println!();
            println!("Init options:");
            println!("  --force                    Init even if directory is non-empty");
            println!("  --no-git                   Skip .gitignore creation");
            println!("  --ai <target>              Setup AI agent config + MCP server");
            println!("                             Targets: claude, cursor, copilot, windsurf, cline, all");
            println!("  --mcp-binary               Use local binary path instead of npx in MCP configs");
            println!();
            println!("MCP install options:");
            println!("  --prefix <path>            Install to <path>/bin/ instead of ~/.synoema/bin/");
            println!("  --from-source              Build from source instead of downloading binary");
            println!();
            println!("Options:");
            println!("  --errors human             Human-readable errors with source snippets (default)");
            println!("  --errors json              JSON errors for LLM/tool consumption");
            println!();
            println!("REPL commands:");
            println!("  :type <expr>             Show inferred type");
            println!("  :load <file>             Load a source file");
            println!("  :quit                    Exit REPL");
        }
        _ => repl(format),
    }
}

// ── Project Init ─────────────────────────────────────────

fn init_project(name_arg: Option<&str>, force: bool, no_git: bool, ai_target: Option<&str>, mcp_binary: bool) {
    // Validate --ai target
    let valid_targets = ["claude", "cursor", "copilot", "windsurf", "cline", "all"];
    if let Some(target) = ai_target {
        if !valid_targets.contains(&target) {
            eprintln!("Error: unknown --ai target '{}'. Valid: {}", target, valid_targets.join(", "));
            std::process::exit(1);
        }
    }

    // Determine project name and root directory
    let (name, root) = match name_arg {
        Some(n) => {
            let p = std::path::Path::new(n);
            let root = if p.is_absolute() {
                p.to_path_buf()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join(n)
            };
            let name = root.file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(n)
                .to_string();
            (name, root)
        }
        None => {
            let cwd = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let name = cwd.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string();
            (name, cwd)
        }
    };

    // Check emptiness
    if name_arg.is_some() && root.exists() && !force && is_dir_non_empty(&root) {
        eprintln!("Error: '{}' already exists and is not empty. Use --force to overwrite.", root.display());
        std::process::exit(1);
    } else if name_arg.is_none() && !force && is_dir_non_empty(&root) {
        eprintln!("Error: current directory is not empty. Use --force to overwrite.");
        std::process::exit(1);
    }

    let src_dir = root.join("src");
    let tests_dir = root.join("tests");

    write_dir(&src_dir);
    write_dir(&tests_dir);

    // Core project files
    write_file(&src_dir.join("main.sno"), &apply_tmpl(TMPL_MAIN, &name));
    write_file(&tests_dir.join("test.sno"), &apply_tmpl(TMPL_TEST, &name));
    write_file(&root.join("project.sno"), &apply_tmpl(TMPL_PROJECT, &name));
    if !no_git {
        write_file(&root.join(".gitignore"), TMPL_GITIGNORE);
    }

    // Always generate AGENTS.md (universal standard)
    write_file(&root.join("AGENTS.md"), &apply_tmpl(TMPL_AGENTS, &name));

    // Always copy LLM docs into the project
    let docs_llm = root.join("docs").join("llm");
    write_dir(&docs_llm);
    write_file(&docs_llm.join("synoema.md"), LLM_SYNOEMA_REF);
    write_file(&docs_llm.join("stdlib.md"), LLM_STDLIB_REF);

    // AI agent-specific setup
    let target = ai_target.unwrap_or("");
    let wants = |t: &str| target == t || target == "all";

    // Claude Code: CLAUDE.md + .claude/settings.json
    if wants("claude") || ai_target.is_none() {
        // Always create CLAUDE.md (thin pointer)
        write_file(&root.join("CLAUDE.md"), &apply_tmpl(TMPL_CLAUDE, &name));
    }
    let mcp_mode = if mcp_binary { "binary" } else { "npx" };

    if wants("claude") {
        let claude_dir = root.join(".claude");
        write_dir(&claude_dir);
        write_file(&claude_dir.join("settings.json"), &mcp_config_json("mcpServers", mcp_binary));
        println!("  Claude Code: CLAUDE.md + .claude/settings.json (MCP {mcp_mode})");
    }

    // Cursor: .cursorrules + .cursor/mcp.json
    if wants("cursor") {
        write_file(&root.join(".cursorrules"), &apply_tmpl(TMPL_CURSORRULES, &name));
        let cursor_dir = root.join(".cursor");
        write_dir(&cursor_dir);
        write_file(&cursor_dir.join("mcp.json"), &mcp_config_json("mcpServers", mcp_binary));
        println!("  Cursor: .cursorrules + .cursor/mcp.json (MCP {mcp_mode})");
    }

    // GitHub Copilot: .github/copilot-instructions.md + .github/copilot/mcp.json
    if wants("copilot") {
        let github_dir = root.join(".github");
        write_dir(&github_dir);
        write_file(&github_dir.join("copilot-instructions.md"), &apply_tmpl(TMPL_COPILOT_INSTRUCTIONS, &name));
        let copilot_dir = github_dir.join("copilot");
        write_dir(&copilot_dir);
        write_file(&copilot_dir.join("mcp.json"), &mcp_config_json("servers", mcp_binary));
        println!("  Copilot: .github/copilot-instructions.md + .github/copilot/mcp.json (MCP {mcp_mode})");
    }

    // Windsurf: .windsurfrules
    if wants("windsurf") {
        write_file(&root.join(".windsurfrules"), &apply_tmpl(TMPL_WINDSURFRULES, &name));
        println!("  Windsurf: .windsurfrules (add MCP via Windsurf settings)");
    }

    // Cline: .clinerules + .vscode/mcp.json
    if wants("cline") {
        write_file(&root.join(".clinerules"), &apply_tmpl(TMPL_CLINERULES, &name));
        let vscode_dir = root.join(".vscode");
        write_dir(&vscode_dir);
        write_file(&vscode_dir.join("mcp.json"), &mcp_config_json("servers", mcp_binary));
        println!("  Cline: .clinerules + .vscode/mcp.json (MCP {mcp_mode})");
    }

    println!("Created Synoema project '{}'", name);
    if ai_target.is_some() {
        let version = env!("CARGO_PKG_VERSION");
        if mcp_binary {
            println!("MCP server: {} (binary mode)", mcp_default_bin_path());
            println!("  Install binary: synoema mcp-install");
        } else {
            println!("MCP server: npx synoema-mcp@{} (auto-configured)", version);
        }
    }
    println!();
    if name_arg.is_some() {
        println!("Next steps:");
        println!("  cd {}", name);
        println!("  synoema run src/main.sno");
    } else {
        println!("Next steps:");
        println!("  synoema run src/main.sno");
    }
}

fn apply_tmpl(tmpl: &str, name: &str) -> String {
    tmpl.replace("{{name}}", name)
        .replace("{{version}}", env!("CARGO_PKG_VERSION"))
}

fn mcp_config_json(key: &str, mcp_binary: bool) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let (cmd, args) = if mcp_binary {
        let bin = mcp_default_bin_path();
        (bin, "[]".to_string())
    } else {
        ("npx".to_string(), format!("[\"synoema-mcp@{}\"]", version))
    };
    // key is "mcpServers" or "servers"
    let type_field = if key == "servers" {
        "\n      \"type\": \"stdio\","
    } else {
        ""
    };
    format!(
        "{{\n  \"{key}\": {{\n    \"synoema\": {{{type_field}\n      \"command\": \"{cmd}\",\n      \"args\": {args}\n    }}\n  }}\n}}\n"
    )
}

fn mcp_default_bin_path() -> String {
    if cfg!(windows) {
        let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "~".into());
        format!("{}\\.synoema\\bin\\synoema-mcp.exe", home)
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".into());
        format!("{}/.synoema/bin/synoema-mcp", home)
    }
}

// ── MCP Install / Update ─────────────────────────────────

fn detect_platform() -> Option<(&'static str, &'static str)> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("macos", "aarch64") => Some(("darwin-arm64", "")),
        ("macos", "x86_64")  => Some(("darwin-x64", "")),
        ("linux", "x86_64")  => Some(("linux-x64", "")),
        ("windows", "x86_64") => Some(("win32-x64", ".exe")),
        _ => None,
    }
}

fn mcp_install(prefix: Option<&str>, from_source: bool) {
    let version = env!("CARGO_PKG_VERSION");

    if from_source {
        println!("Building synoema-mcp from source...");
        // Try to find mcp/ relative to executable
        let exe = std::env::current_exe().unwrap_or_default();
        let mcp_dir = exe.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.join("mcp"))
            .filter(|p| p.join("Cargo.toml").exists());

        let mcp_dir = match mcp_dir {
            Some(d) => d,
            None => {
                eprintln!("Error: cannot find mcp/ workspace. Clone the repo and run from there.");
                eprintln!("  git clone https://github.com/Delimitter/synoema");
                eprintln!("  cd synoema/mcp && cargo build --release");
                std::process::exit(1);
            }
        };

        let status = std::process::Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&mcp_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                let built = mcp_dir.join("target/release/synoema-mcp");
                let dest = install_dest(prefix);
                copy_binary(&built, &dest);
                println!("Installed: {} (built from source)", dest.display());
            }
            _ => {
                eprintln!("Error: cargo build failed. Ensure Rust toolchain is installed.");
                std::process::exit(1);
            }
        }
        return;
    }

    let (platform, ext) = match detect_platform() {
        Some(p) => p,
        None => {
            eprintln!("Error: unsupported platform {}/{}",
                std::env::consts::OS, std::env::consts::ARCH);
            eprintln!("Use --from-source to build from source code.");
            std::process::exit(1);
        }
    };

    let binary_name = format!("synoema-mcp-{}-{}{}", version, platform, ext);
    let url = format!(
        "https://github.com/Delimitter/synoema/releases/download/v{}/{}",
        version, binary_name
    );

    println!("Downloading {} ...", binary_name);
    let dest = install_dest(prefix);
    let dest_dir = dest.parent().unwrap();
    std::fs::create_dir_all(dest_dir).unwrap_or_else(|e| {
        eprintln!("Error creating directory '{}': {}", dest_dir.display(), e);
        std::process::exit(1);
    });

    let ok = download_file(&url, &dest);
    if !ok {
        eprintln!("Error: download failed. Check your internet connection.");
        eprintln!("Manual download: {}", url);
        std::process::exit(1);
    }

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
    }

    println!("Installed: {} (v{})", dest.display(), version);
    println!();
    println!("Add to your MCP config:");
    println!("  \"command\": \"{}\"", dest.display());
}

fn install_dest(prefix: Option<&str>) -> std::path::PathBuf {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    if let Some(p) = prefix {
        std::path::PathBuf::from(p).join("bin").join(format!("synoema-mcp{}", ext))
    } else {
        let bin_name = format!("synoema-mcp{}", ext);
        if cfg!(windows) {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
            std::path::PathBuf::from(home).join(".synoema").join("bin").join(bin_name)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            std::path::PathBuf::from(home).join(".synoema").join("bin").join(bin_name)
        }
    }
}

fn download_file(url: &str, dest: &std::path::Path) -> bool {
    let dest_str = dest.to_str().unwrap_or("output");

    // Try curl first (macOS, most Linux)
    if let Ok(status) = std::process::Command::new("curl")
        .args(["-fsSL", "-o", dest_str, url])
        .status()
    {
        if status.success() { return true; }
    }

    // Fallback: wget (some Linux)
    if let Ok(status) = std::process::Command::new("wget")
        .args(["-qO", dest_str, url])
        .status()
    {
        if status.success() { return true; }
    }

    // Fallback: PowerShell (Windows)
    if cfg!(windows) {
        let ps_cmd = format!(
            "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
            url, dest_str
        );
        if let Ok(status) = std::process::Command::new("powershell")
            .args(["-Command", &ps_cmd])
            .status()
        {
            if status.success() { return true; }
        }
    }

    false
}

fn copy_binary(src: &std::path::Path, dest: &std::path::Path) {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("Error creating directory '{}': {}", parent.display(), e);
            std::process::exit(1);
        });
    }
    std::fs::copy(src, dest).unwrap_or_else(|e| {
        eprintln!("Error copying binary: {}", e);
        std::process::exit(1);
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755));
    }
}

fn mcp_update() {
    // Find installed MCP binary
    let default_bin = install_dest(None);
    let bin_path = if default_bin.exists() {
        default_bin
    } else {
        // Try PATH
        match which_mcp() {
            Some(p) => p,
            None => {
                eprintln!("No synoema-mcp binary found.");
                eprintln!("Install first: synoema mcp-install");
                std::process::exit(1);
            }
        }
    };

    // Get installed version via --version
    let installed_version = match std::process::Command::new(&bin_path)
        .arg("--version")
        .output()
    {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Parse "synoema-mcp X.Y.Z" → "X.Y.Z"
            s.strip_prefix("synoema-mcp ").unwrap_or(&s).to_string()
        }
        _ => {
            eprintln!("Cannot determine installed version. Reinstall: synoema mcp-install");
            std::process::exit(1);
        }
    };

    println!("Installed: v{}", installed_version);
    println!("Checking for updates...");

    // Query GitHub releases API
    let api_url = "https://api.github.com/repos/Delimitter/synoema/releases/latest";
    let latest_version = match fetch_latest_version(api_url) {
        Some(v) => v,
        None => {
            eprintln!("Cannot check for updates (GitHub API unavailable or rate limited).");
            eprintln!("Manual check: https://github.com/Delimitter/synoema/releases");
            std::process::exit(1);
        }
    };

    if latest_version == installed_version {
        println!("synoema-mcp is up to date (v{})", installed_version);
        return;
    }

    println!("New version available: v{} → v{}", installed_version, latest_version);

    let (platform, ext) = match detect_platform() {
        Some(p) => p,
        None => {
            eprintln!("Cannot auto-update on this platform. Use --from-source.");
            std::process::exit(1);
        }
    };

    let binary_name = format!("synoema-mcp-{}-{}{}", latest_version, platform, ext);
    let url = format!(
        "https://github.com/Delimitter/synoema/releases/download/v{}/{}",
        latest_version, binary_name
    );

    println!("Downloading {} ...", binary_name);
    let ok = download_file(&url, &bin_path);
    if !ok {
        eprintln!("Download failed. Manual: {}", url);
        std::process::exit(1);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755));
    }

    println!("Updated: {} → v{}", bin_path.display(), latest_version);
}

fn which_mcp() -> Option<std::path::PathBuf> {
    let name = if cfg!(windows) { "synoema-mcp.exe" } else { "synoema-mcp" };
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|p| p.join(name))
            .find(|p| p.exists())
    })
}

fn fetch_latest_version(api_url: &str) -> Option<String> {
    // Use curl to fetch JSON, parse tag_name
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "-H", "Accept: application/vnd.github+json", api_url])
        .output()
        .ok()?;

    if !output.status.success() { return None; }

    let body = String::from_utf8_lossy(&output.stdout);
    // Simple parse: find "tag_name": "vX.Y.Z"
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"tag_name\"") {
            // Extract version from "tag_name": "v0.1.0-alpha.1",
            let v = trimmed.split('"').nth(3)?;
            return Some(v.strip_prefix('v').unwrap_or(v).to_string());
        }
    }
    None
}

fn is_dir_non_empty(path: &std::path::Path) -> bool {
    path.exists() && path.read_dir()
        .map(|mut d| d.next().is_some())
        .unwrap_or(false)
}

fn write_dir(path: &std::path::Path) {
    if let Err(e) = std::fs::create_dir_all(path) {
        eprintln!("Error creating directory '{}': {}", path.display(), e);
        std::process::exit(1);
    }
}

fn write_file(path: &std::path::Path, content: &str) {
    if let Err(e) = std::fs::write(path, content) {
        eprintln!("Error writing '{}': {}", path.display(), e);
        std::process::exit(1);
    }
}

// ── Unit tests for init ───────────────────────────────────

#[cfg(test)]
mod init_tests {
    use super::*;

    fn tmp_dir(suffix: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!("synoema_init_test_{}", suffix));
        let _ = std::fs::remove_dir_all(&base);
        base
    }

    fn init_project_at(root: &std::path::Path, name: &str, force: bool, no_git: bool, ai_target: Option<&str>) {
        init_project_at_full(root, name, force, no_git, ai_target, false);
    }

    fn init_project_at_full(root: &std::path::Path, name: &str, force: bool, no_git: bool, ai_target: Option<&str>, mcp_binary: bool) {
        if !force && is_dir_non_empty(root) {
            panic!("directory non-empty and force=false");
        }
        let src_dir = root.join("src");
        let tests_dir = root.join("tests");
        write_dir(&src_dir);
        write_dir(&tests_dir);
        write_file(&src_dir.join("main.sno"), &apply_tmpl(TMPL_MAIN, name));
        write_file(&tests_dir.join("test.sno"), &apply_tmpl(TMPL_TEST, name));
        write_file(&root.join("project.sno"), &apply_tmpl(TMPL_PROJECT, name));
        if !no_git {
            write_file(&root.join(".gitignore"), TMPL_GITIGNORE);
        }
        // Always: AGENTS.md + docs
        write_file(&root.join("AGENTS.md"), &apply_tmpl(TMPL_AGENTS, name));
        let docs_llm = root.join("docs").join("llm");
        write_dir(&docs_llm);
        write_file(&docs_llm.join("synoema.md"), LLM_SYNOEMA_REF);
        write_file(&docs_llm.join("stdlib.md"), LLM_STDLIB_REF);

        let target = ai_target.unwrap_or("");
        let wants = |t: &str| target == t || target == "all";

        if wants("claude") || ai_target.is_none() {
            write_file(&root.join("CLAUDE.md"), &apply_tmpl(TMPL_CLAUDE, name));
        }
        if wants("claude") {
            let claude_dir = root.join(".claude");
            write_dir(&claude_dir);
            write_file(&claude_dir.join("settings.json"), &mcp_config_json("mcpServers", mcp_binary));
        }
        if wants("cursor") {
            write_file(&root.join(".cursorrules"), &apply_tmpl(TMPL_CURSORRULES, name));
            let cursor_dir = root.join(".cursor");
            write_dir(&cursor_dir);
            write_file(&cursor_dir.join("mcp.json"), &mcp_config_json("mcpServers", mcp_binary));
        }
        if wants("copilot") {
            let github_dir = root.join(".github");
            write_dir(&github_dir);
            write_file(&github_dir.join("copilot-instructions.md"), &apply_tmpl(TMPL_COPILOT_INSTRUCTIONS, name));
            let copilot_dir = github_dir.join("copilot");
            write_dir(&copilot_dir);
            write_file(&copilot_dir.join("mcp.json"), &mcp_config_json("servers", mcp_binary));
        }
        if wants("windsurf") {
            write_file(&root.join(".windsurfrules"), &apply_tmpl(TMPL_WINDSURFRULES, name));
        }
        if wants("cline") {
            write_file(&root.join(".clinerules"), &apply_tmpl(TMPL_CLINERULES, name));
            let vscode_dir = root.join(".vscode");
            write_dir(&vscode_dir);
            write_file(&vscode_dir.join("mcp.json"), &mcp_config_json("servers", mcp_binary));
        }
    }

    #[test]
    fn init_creates_structure() {
        let root = tmp_dir("creates");
        init_project_at(&root, "myapp", false, false, None);
        assert!(root.join("src/main.sno").exists(), "src/main.sno missing");
        assert!(root.join("tests/test.sno").exists(), "tests/test.sno missing");
        assert!(root.join("project.sno").exists(), "project.sno missing");
        assert!(root.join("AGENTS.md").exists(), "AGENTS.md missing");
        assert!(root.join("CLAUDE.md").exists(), "CLAUDE.md missing");
        assert!(root.join(".gitignore").exists(), ".gitignore missing");
        assert!(root.join("docs/llm/synoema.md").exists(), "docs/llm/synoema.md missing");
        assert!(root.join("docs/llm/stdlib.md").exists(), "docs/llm/stdlib.md missing");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_name_substitution() {
        let root = tmp_dir("name_sub");
        init_project_at(&root, "coolapp", false, false, None);
        let main_src = std::fs::read_to_string(root.join("src/main.sno")).unwrap();
        assert!(main_src.contains("coolapp"), "name not substituted in main.sno");
        let project_src = std::fs::read_to_string(root.join("project.sno")).unwrap();
        assert!(project_src.contains("coolapp"), "name not substituted in project.sno");
        let agents = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(agents.contains("coolapp"), "name not substituted in AGENTS.md");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_fails_nonempty_without_force() {
        let root = tmp_dir("nonempty");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("existing.txt"), "content").unwrap();
        assert!(is_dir_non_empty(&root));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_force_nonempty() {
        let root = tmp_dir("force");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("existing.txt"), "content").unwrap();
        init_project_at(&root, "forceapp", true, false, None);
        assert!(root.join("src/main.sno").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_no_git() {
        let root = tmp_dir("nogit");
        init_project_at(&root, "nogitapp", false, true, None);
        assert!(root.join("src/main.sno").exists());
        assert!(!root.join(".gitignore").exists(), ".gitignore should not exist with --no-git");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_ai_cursor() {
        let root = tmp_dir("ai_cursor");
        init_project_at(&root, "cursorapp", false, false, Some("cursor"));
        assert!(root.join("AGENTS.md").exists(), "AGENTS.md missing");
        assert!(root.join(".cursorrules").exists(), ".cursorrules missing");
        assert!(root.join(".cursor/mcp.json").exists(), ".cursor/mcp.json missing");
        // No CLAUDE.md when --ai cursor (not claude/all)
        assert!(!root.join("CLAUDE.md").exists(), "CLAUDE.md should not exist for --ai cursor");
        let mcp = std::fs::read_to_string(root.join(".cursor/mcp.json")).unwrap();
        assert!(mcp.contains("synoema-mcp"), "MCP config should reference synoema-mcp");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_ai_all() {
        let root = tmp_dir("ai_all");
        init_project_at(&root, "allapp", false, false, Some("all"));
        assert!(root.join("AGENTS.md").exists(), "AGENTS.md missing");
        assert!(root.join("CLAUDE.md").exists(), "CLAUDE.md missing");
        assert!(root.join(".claude/settings.json").exists(), ".claude/settings.json missing");
        assert!(root.join(".cursorrules").exists(), ".cursorrules missing");
        assert!(root.join(".cursor/mcp.json").exists(), ".cursor/mcp.json missing");
        assert!(root.join(".github/copilot-instructions.md").exists(), "copilot-instructions.md missing");
        assert!(root.join(".github/copilot/mcp.json").exists(), "copilot mcp.json missing");
        assert!(root.join(".windsurfrules").exists(), ".windsurfrules missing");
        assert!(root.join(".clinerules").exists(), ".clinerules missing");
        assert!(root.join(".vscode/mcp.json").exists(), ".vscode/mcp.json missing");
        assert!(root.join("docs/llm/synoema.md").exists(), "docs/llm/synoema.md missing");
        assert!(root.join("docs/llm/stdlib.md").exists(), "docs/llm/stdlib.md missing");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_ai_copilot() {
        let root = tmp_dir("ai_copilot");
        init_project_at(&root, "copilotapp", false, false, Some("copilot"));
        assert!(root.join("AGENTS.md").exists(), "AGENTS.md missing");
        assert!(root.join(".github/copilot-instructions.md").exists(), "copilot-instructions.md missing");
        assert!(root.join(".github/copilot/mcp.json").exists(), "copilot mcp.json missing");
        assert!(!root.join(".cursorrules").exists(), ".cursorrules should not exist for --ai copilot");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_mcp_version_pinning() {
        let root = tmp_dir("version_pin");
        init_project_at(&root, "pinapp", false, false, Some("claude"));
        let settings = std::fs::read_to_string(root.join(".claude/settings.json")).unwrap();
        let version = env!("CARGO_PKG_VERSION");
        let expected = format!("synoema-mcp@{}", version);
        assert!(settings.contains(&expected),
            "MCP config should contain pinned version '{}', got: {}", expected, settings);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn init_mcp_binary_mode() {
        let root = tmp_dir("mcp_binary");
        init_project_at_full(&root, "binapp", false, false, Some("claude"), true);
        let settings = std::fs::read_to_string(root.join(".claude/settings.json")).unwrap();
        assert!(settings.contains(".synoema/bin/synoema-mcp"),
            "MCP binary config should contain binary path, got: {}", settings);
        assert!(!settings.contains("npx"),
            "MCP binary config should NOT contain npx, got: {}", settings);
        let _ = std::fs::remove_dir_all(&root);
    }
}

fn run_file(path: &str, format: ErrorFormat, script_args: Vec<String>) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let base_dir = std::path::Path::new(path).parent().unwrap_or(std::path::Path::new("."));
    match synoema_eval::eval_main_with_args(&source, Some(base_dir), script_args) {
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

fn fmt_command(path: &str, check: bool, _format: ErrorFormat) {
    let p = std::path::Path::new(path);
    if p.is_dir() {
        match fmt::format_directory(p, check) {
            Ok((total, changed)) => {
                if check {
                    if changed > 0 {
                        eprintln!("{}/{} file(s) need formatting", changed, total);
                        std::process::exit(1);
                    } else {
                        println!("{} file(s) already formatted", total);
                    }
                } else {
                    println!("Formatted {}/{} file(s)", total - changed, total);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    } else {
        match fmt::format_file(p, check) {
            Ok(true) => {
                if check {
                    println!("Already formatted: {}", path);
                }
            }
            Ok(false) => {
                eprintln!("Needs formatting: {}", path);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

fn build_file(path: &str, output: Option<&str>, check_only: bool, _verbose: bool, format: ErrorFormat) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            std::process::exit(2);
        }
    };

    let base_dir = std::path::Path::new(path).parent().unwrap_or(std::path::Path::new("."));

    // Extract Core IR
    match synoema_codegen::extract_core_ir(&source, Some(base_dir)) {
        Ok(core_ir) => {
            if check_only {
                println!("Type check passed");
                return;
            }

            // Determine output path
            let default_output = format!("{}.bc", path);
            let output_path = output.unwrap_or(&default_output);

            // Write bytecode file with header
            let bytecode = format!(
                "SYNOEMA BYTECODE v1\nsource: {}\n\n[Core IR]\n{}\n",
                path, core_ir
            );

            match std::fs::write(output_path, bytecode) {
                Ok(_) => {
                    let size = std::fs::metadata(output_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    println!("Built: {} ({} bytes)", output_path, size);
                }
                Err(e) => {
                    eprintln!("Error writing '{}': {}", output_path, e);
                    std::process::exit(2);
                }
            }
        }
        Err(diag) => {
            print_diag(&diag, Some(&source), format);
            std::process::exit(1);
        }
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
