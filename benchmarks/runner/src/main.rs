// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use clap::Parser;
use std::path::{Path, PathBuf};
use std::time::Instant;

mod phases;
mod report;
mod telemetry;

#[derive(Parser)]
#[command(name = "synoema-bench", about = "Synoema comparative benchmark suite")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Run benchmark phases
    Run {
        /// Run all phases (token + runtime + llm)
        #[arg(long)]
        all: bool,

        /// Phases to run: token,runtime,llm
        #[arg(long, value_delimiter = ',')]
        phases: Option<Vec<String>>,

        /// OpenRouter API key (required for llm phase)
        #[arg(long)]
        openrouter_key: Option<String>,

        /// Filter models (comma-separated, include only matching)
        #[arg(long, value_delimiter = ',')]
        models: Option<Vec<String>>,

        /// Exclude models (comma-separated, skip matching)
        #[arg(long, value_delimiter = ',')]
        exclude_models: Option<Vec<String>>,

        /// Filter by model tier: frontier, mid, weak
        #[arg(long)]
        tier: Option<String>,

        /// Filter tasks (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tasks: Option<Vec<String>>,

        /// Number of repeats for runtime/llm (default: 5)
        #[arg(long, default_value = "5")]
        repeats: usize,

        /// Verbose output: show commands, individual run timings, script stderr
        #[arg(long, short = 'v')]
        verbose: bool,

        /// Parallel threads for Phase C models (default: 2, 0 = sequential)
        #[arg(long, default_value = "2")]
        parallel: usize,

        /// Use local ollama for Phase C instead of OpenRouter
        #[arg(long)]
        ollama: bool,

        /// Ollama model to use (default: qwen3:8b)
        #[arg(long, default_value = "qwen3:8b")]
        ollama_model: String,

        /// Models for Phase D size benchmark (comma-separated ollama model tags)
        #[arg(long, value_delimiter = ',')]
        size_models: Option<Vec<String>>,
    },
}

fn resolve_phases(all: bool, phases: &Option<Vec<String>>) -> Vec<String> {
    if all {
        return vec!["token".into(), "runtime".into(), "llm".into(), "size".into()];
    }
    if let Some(p) = phases {
        return p.clone();
    }
    vec!["token".into(), "runtime".into()]
}

fn discover_tasks(bench_root: &Path, filter: &Option<Vec<String>>) -> Vec<String> {
    let tasks_dir = bench_root.join("tasks");
    let mut tasks = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&tasks_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(filter) = filter {
                    if filter.contains(&name) {
                        tasks.push(name);
                    }
                } else {
                    tasks.push(name);
                }
            }
        }
    }
    tasks.sort();
    tasks
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            all,
            phases,
            openrouter_key,
            models,
            exclude_models,
            tier,
            tasks,
            repeats,
            verbose,
            parallel,
            ollama,
            ollama_model,
            size_models,
        } => {
            let bench_root = find_bench_root();
            let active_phases = resolve_phases(all, &phases);
            let task_list = discover_tasks(&bench_root, &tasks);

            if task_list.is_empty() {
                eprintln!("No benchmark tasks found in {}/tasks/", bench_root.display());
                std::process::exit(1);
            }

            let languages = vec!["synoema", "python", "javascript", "typescript", "cpp"];

            telemetry::print_header(&active_phases, &task_list, &languages);

            let start = Instant::now();
            let mut all_results = report::AllResults::default();

            // Phase A: Token efficiency
            if active_phases.contains(&"token".to_string()) {
                telemetry::print_phase_start("A", "TOKEN ANALYSIS", "parallel");
                match phases::tokens::run(&bench_root, &task_list, &languages, verbose) {
                    Ok(results) => {
                        telemetry::print_token_results(&results);
                        all_results.tokens = Some(results);
                    }
                    Err(e) => eprintln!("  Phase A error: {e}"),
                }
            }

            // Phase B: Runtime performance
            if active_phases.contains(&"runtime".to_string()) {
                telemetry::print_phase_start("B", "RUNTIME PERFORMANCE", "sequential");
                match phases::runtime::run(&bench_root, &task_list, &languages, repeats, verbose) {
                    Ok(results) => {
                        telemetry::print_runtime_results(&results);
                        all_results.runtime = Some(results);
                    }
                    Err(e) => eprintln!("  Phase B error: {e}"),
                }
            }

            // Phase C: LLM generation
            if active_phases.contains(&"llm".to_string()) {
                if ollama {
                    telemetry::print_phase_start("C", "LLM CODE GENERATION (ollama)", "sequential");
                    match phases::llm::run_ollama(
                        &bench_root,
                        &task_list,
                        &languages,
                        &ollama_model,
                        repeats,
                        verbose,
                    ) {
                        Ok(results) => {
                            telemetry::print_llm_results(&results);
                            all_results.llm = Some(results);
                        }
                        Err(e) => eprintln!("  Phase C error: {e}"),
                    }
                } else if let Some(ref key) = openrouter_key {
                    let mode = if parallel > 1 { "parallel" } else { "sequential" };
                    telemetry::print_phase_start("C", "LLM CODE GENERATION", mode);
                    let model_list =
                        phases::llm::resolve_models(&models, &exclude_models, &tier);
                    match phases::llm::run(
                        &bench_root,
                        &task_list,
                        &languages,
                        &model_list,
                        key,
                        repeats,
                        verbose,
                        parallel,
                    ) {
                        Ok(results) => {
                            telemetry::print_llm_results(&results);
                            all_results.llm = Some(results);
                        }
                        Err(e) => eprintln!("  Phase C error: {e}"),
                    }
                } else {
                    eprintln!();
                    eprintln!(
                        "  Warning: Phase C skipped — no --openrouter-key or --ollama provided."
                    );
                    eprintln!("  Token and runtime benchmarks will still run.");
                }
            }

            // Phase D: Model Size Reduction
            if active_phases.contains(&"size".to_string()) {
                telemetry::print_phase_start("D", "MODEL SIZE REDUCTION", "sequential");
                let sm = size_models.clone().unwrap_or_default();
                match phases::size::run(&bench_root, &task_list, &sm, repeats, verbose) {
                    Ok(results) => {
                        // Print summary
                        eprintln!("\n  Phase D complete:");
                        for mr in &results.models {
                            for cr in &mr.configs {
                                eprintln!(
                                    "    {} / {}: syntax={:.0}% type={:.0}% run={:.0}%",
                                    mr.model, cr.config,
                                    cr.avg_syntax_pct, cr.avg_type_pct, cr.avg_run_pct,
                                );
                            }
                        }
                        all_results.size = Some(results);
                    }
                    Err(e) => eprintln!("  Phase D error: {e}"),
                }
            }

            let elapsed = start.elapsed();
            let results_dir = report::write_results(&bench_root, &all_results, elapsed);
            telemetry::print_footer(&results_dir, elapsed);
        }
    }
}

fn find_bench_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    loop {
        if dir.join("tasks").is_dir() && dir.join("runner").is_dir() {
            return dir;
        }
        if dir.join("benchmarks").join("tasks").is_dir() {
            return dir.join("benchmarks");
        }
        if !dir.pop() {
            break;
        }
    }
    // Fallback: use CARGO_MANIFEST_DIR to find runner/, then go up
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let runner_dir = PathBuf::from(manifest);
        if let Some(bench) = runner_dir.parent() {
            if bench.join("tasks").is_dir() {
                return bench.to_path_buf();
            }
        }
    }
    PathBuf::from("..")
}
