// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeResults {
    pub tasks: Vec<TaskRuntime>,
    pub averages: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRuntime {
    pub task: String,
    pub measurements: BTreeMap<String, RuntimeMeasurement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMeasurement {
    pub median_ms: f64,
    pub p5_ms: f64,
    pub p95_ms: f64,
    pub runs: Vec<f64>,
}

const WARMUP_RUNS: usize = 3;

/// Map of language name to (file extension, run command builder)
fn lang_ext(lang: &str) -> Option<&'static str> {
    match lang {
        "synoema" => Some("sno"),
        "python" => Some("py"),
        "javascript" => Some("js"),
        "typescript" => Some("ts"),
        "cpp" => Some("cpp"),
        _ => None,
    }
}

fn build_run_command(lang: &str, file: &Path, bench_root: &Path) -> Option<Vec<String>> {
    let f = file.to_string_lossy().to_string();
    match lang {
        "synoema" => {
            // Use pre-built binary to avoid cargo overhead in timing
            let lang_dir = bench_root.parent().map(|p| p.join("lang")).unwrap_or_default();
            // Binary may be named "synoema" or "synoema-repl" depending on build config
            let release_bin = if lang_dir.join("target/release/synoema").exists() {
                lang_dir.join("target/release/synoema")
            } else {
                lang_dir.join("target/release/synoema-repl")
            };
            let debug_bin = if lang_dir.join("target/debug/synoema").exists() {
                lang_dir.join("target/debug/synoema")
            } else {
                lang_dir.join("target/debug/synoema-repl")
            };
            let bin = if release_bin.exists() {
                release_bin.to_string_lossy().to_string()
            } else if debug_bin.exists() {
                debug_bin.to_string_lossy().to_string()
            } else {
                // Fallback: build release first, then use binary
                eprintln!("  Building synoema-repl (release)...");
                let build = Command::new("cargo")
                    .args(["build", "--release", "--manifest-path",
                           &lang_dir.join("Cargo.toml").to_string_lossy(),
                           "-p", "synoema-repl"])
                    .output();
                match build {
                    Ok(b) if b.status.success() => release_bin.to_string_lossy().to_string(),
                    _ => {
                        eprintln!("  Warning: could not build synoema-repl");
                        return None;
                    }
                }
            };
            Some(vec![bin, "jit".into(), f])
        }
        "python" => Some(vec!["python3".into(), f]),
        "javascript" => Some(vec!["node".into(), f]),
        "typescript" => Some(vec!["npx".into(), "tsx".into(), f]),
        "cpp" => {
            // Compile first, then run
            let out = format!("/tmp/bench_{}", file.file_stem().unwrap().to_string_lossy());
            let compile = Command::new("g++")
                .args(["-std=c++17", "-O2", "-o", &out, &f])
                .output();
            match compile {
                Ok(c) if c.status.success() => Some(vec![out]),
                Ok(c) => {
                    let stderr = String::from_utf8_lossy(&c.stderr);
                    eprintln!("  Warning: C++ compile failed: {stderr}");
                    None
                }
                Err(e) => {
                    eprintln!("  Warning: g++ not found: {e}");
                    None
                }
            }
        }
        _ => None,
    }
}

fn measure_once(cmd: &[String]) -> Option<f64> {
    let start = Instant::now();
    let output = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    match output {
        Ok(s) if s.success() => Some(elapsed),
        Ok(_) => None,
        Err(e) => {
            eprintln!("  Warning: command error: {e}");
            None
        }
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn run(
    bench_root: &Path,
    task_list: &[String],
    languages: &[&str],
    repeats: usize,
    verbose: bool,
) -> Result<RuntimeResults, String> {
    let mut results = RuntimeResults::default();
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut count = 0u64;

    // Runtime-eligible tasks: those with actual executable programs
    let runtime_tasks: Vec<&String> = task_list
        .iter()
        .filter(|t| {
            // Skip token-only tasks
            !matches!(t.as_str(), "json_build" | "error_handling" | "pattern_match" | "type_definition")
        })
        .collect();

    for task in &runtime_tasks {
        let task_dir = bench_root.join("tasks").join(task);
        let mut measurements = BTreeMap::new();

        eprint!("  [B] {task}: ");

        for &lang in languages {
            let ext = match lang_ext(lang) {
                Some(e) => e,
                None => continue,
            };

            // Find the source file
            let file = task_dir.join(format!("{task}.{ext}"));
            if !file.exists() {
                continue;
            }

            let cmd = match build_run_command(lang, &file, bench_root) {
                Some(c) => c,
                None => continue,
            };

            if verbose {
                eprintln!("    cmd: {}", cmd.join(" "));
            }

            // Warm-up
            for i in 0..WARMUP_RUNS {
                let t = measure_once(&cmd);
                if verbose {
                    if let Some(ms) = t {
                        eprintln!("    {lang} warmup {}/{WARMUP_RUNS}: {ms:.1}ms", i + 1);
                    } else {
                        eprintln!("    {lang} warmup {}/{WARMUP_RUNS}: failed", i + 1);
                    }
                }
            }

            // Measured runs
            let mut times: Vec<f64> = Vec::new();
            for i in 0..repeats {
                if let Some(t) = measure_once(&cmd) {
                    if verbose {
                        eprintln!("    {lang} run {}/{repeats}: {t:.1}ms", i + 1);
                    }
                    times.push(t);
                } else if verbose {
                    eprintln!("    {lang} run {}/{repeats}: failed", i + 1);
                }
            }

            if times.is_empty() {
                eprint!("{lang}=FAIL ");
                continue;
            }

            times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = percentile(&times, 50.0);
            let p5 = percentile(&times, 5.0);
            let p95 = percentile(&times, 95.0);

            eprint!("{lang}={median:.0}ms ");

            *totals.entry(lang.to_string()).or_insert(0.0) += median;

            measurements.insert(
                lang.to_string(),
                RuntimeMeasurement {
                    median_ms: median,
                    p5_ms: p5,
                    p95_ms: p95,
                    runs: times,
                },
            );
        }

        eprintln!();
        count += 1;

        results.tasks.push(TaskRuntime {
            task: task.to_string(),
            measurements,
        });
    }

    if count > 0 {
        for (lang, total) in &totals {
            results
                .averages
                .insert(lang.clone(), *total / count as f64);
        }
    }

    Ok(results)
}
