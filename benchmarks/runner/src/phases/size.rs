// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Phase D: Model Size Reduction — local ollama inference with multiple
//! small models, multi-config (baseline/compact/multipass), and extended
//! validation (syntax + type + run correctness).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::phases::llm;

// ── Default models ──────────────────────────────────────────────────

const DEFAULT_SIZE_MODELS: &[&str] = &[
    "qwen2.5-coder:0.5b",
    "qwen2.5-coder:1.5b",
    "qwen2.5-coder:3b",
    "qwen2.5-coder:7b",
];

/// LLM-eligible tasks (same as Phase C).
const LLM_TASKS: &[&str] = &[
    "factorial",
    "fibonacci",
    "quicksort",
    "fizzbuzz",
    "filter_map",
    "binary_search",
    "error_handling",
    "pattern_match",
    "type_definition",
];

// ── Config definitions ──────────────────────────────────────────────

#[derive(Debug, Clone)]
struct SizeConfig {
    name: String,
    reference_path: PathBuf,
    multi_pass: bool,
    max_retries: u32,
}

// ── Result types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizeResults {
    pub models: Vec<SizeModelResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeModelResults {
    pub model: String,
    pub configs: Vec<SizeConfigResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeConfigResults {
    pub config: String,
    pub tasks: Vec<SizeTaskResult>,
    pub avg_syntax_pct: f64,
    pub avg_type_pct: f64,
    pub avg_run_pct: f64,
    pub avg_tokens_out: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeTaskResult {
    pub task: String,
    pub syntax_ok: u32,
    pub type_ok: u32,
    pub run_ok: u32,
    pub total_runs: u32,
    pub avg_tokens_out: f64,
}

// ── Validation ──────────────────────────────────────────────────────

#[derive(Debug)]
struct ValidationResult {
    syntax_ok: bool,
    type_ok: bool,
    run_ok: bool,
    error_json: Option<String>,
}

/// Validate a .sno file: parse → typecheck → run → compare output.
fn validate_sno(
    sno_path: &Path,
    expected_path: &Path,
    synoema_bin: &Path,
) -> ValidationResult {
    // Run with --errors json to get structured errors
    let output = Command::new(synoema_bin)
        .arg("--errors")
        .arg("json")
        .arg("run")
        .arg(sno_path)
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();

            if !o.status.success() {
                // Parse error JSON from stderr for multi-pass feedback
                let error_json = if stderr.contains('{') {
                    Some(stderr.clone())
                } else {
                    None
                };

                // Check if it's a syntax error or type error
                let is_syntax_err = stderr.contains("unexpected_token")
                    || stderr.contains("indentation")
                    || stderr.contains("unterminated");
                let is_type_err = stderr.contains("type_mismatch")
                    || stderr.contains("arity_mismatch")
                    || stderr.contains("unbound_variable")
                    || stderr.contains("infinite_type");

                if is_syntax_err {
                    return ValidationResult {
                        syntax_ok: false,
                        type_ok: false,
                        run_ok: false,
                        error_json,
                    };
                }
                if is_type_err {
                    return ValidationResult {
                        syntax_ok: true,
                        type_ok: false,
                        run_ok: false,
                        error_json,
                    };
                }
                // Runtime error (division by zero, pattern match failure, etc.)
                return ValidationResult {
                    syntax_ok: true,
                    type_ok: true,
                    run_ok: false,
                    error_json,
                };
            }

            // Success: check output correctness
            let run_ok = if expected_path.exists() {
                let expected = std::fs::read_to_string(expected_path).unwrap_or_default();
                stdout.trim() == expected.trim()
            } else {
                true // no expected output = pass if no error
            };

            ValidationResult {
                syntax_ok: true,
                type_ok: true,
                run_ok,
                error_json: None,
            }
        }
        Err(_) => ValidationResult {
            syntax_ok: false,
            type_ok: false,
            run_ok: false,
            error_json: None,
        },
    }
}

/// Extract llm_hint from error JSON for multi-pass feedback.
fn extract_hint(error_json: &str) -> String {
    // Simple extraction: find "llm_hint" field or "message" field
    for line in error_json.lines() {
        let line = line.trim();
        if line.contains("llm_hint") || line.contains("hint:") {
            // Clean up and return
            let hint = line
                .trim_start_matches(|c: char| !c.is_alphabetic())
                .trim_end_matches(|c: char| c == '"' || c == ',');
            if !hint.is_empty() {
                return hint.to_string();
            }
        }
    }
    // Fallback: return first error line
    error_json.lines().next().unwrap_or("Unknown error").to_string()
}

// ── Generation ──────────────────────────────────────────────────────

/// Call ollama to generate code.
fn generate_ollama(
    model: &str,
    prompt: &str,
    temperature: f64,
    _verbose: bool,
) -> Result<(String, u64), String> {
    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": temperature,
            "num_predict": 512,
        }
    });

    let output = Command::new("curl")
        .arg("-s")
        .arg("http://localhost:11434/api/generate")
        .arg("-d")
        .arg(body.to_string())
        .output()
        .map_err(|e| format!("curl failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "ollama API error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let resp: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("JSON parse error: {e}"))?;

    let code = resp["response"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let tokens = resp["eval_count"].as_u64().unwrap_or(0);

    Ok((code, tokens))
}

/// Extract clean Synoema code from LLM response.
fn extract_sno_code(response: &str) -> String {
    // Check for ```sno or ``` code blocks
    if let Some(start) = response.find("```sno") {
        let code_start = start + 6;
        if let Some(end) = response[code_start..].find("```") {
            return response[code_start..code_start + end].trim().to_string();
        }
    }
    if let Some(start) = response.find("```") {
        let code_start = start + 3;
        // Skip language tag if present
        let code_start = if let Some(nl) = response[code_start..].find('\n') {
            code_start + nl + 1
        } else {
            code_start
        };
        if let Some(end) = response[code_start..].find("```") {
            return response[code_start..code_start + end].trim().to_string();
        }
    }
    // No code block: use entire response, strip non-code lines
    response
        .lines()
        .filter(|l| {
            !l.starts_with("Here") && !l.starts_with("This") && !l.starts_with("The ")
                && !l.starts_with("Note") && !l.is_empty()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Main runner ─────────────────────────────────────────────────────

pub fn run(
    bench_root: &Path,
    task_list: &[String],
    size_models: &[String],
    repeats: usize,
    verbose: bool,
) -> Result<SizeResults, String> {
    if !llm::ollama_available() {
        return Err("ollama is not installed or not in PATH".into());
    }

    // Resolve models
    let models: Vec<String> = if size_models.is_empty() {
        DEFAULT_SIZE_MODELS.iter().map(|s| s.to_string()).collect()
    } else {
        size_models.to_vec()
    };

    // Find synoema binary
    let synoema_bin = find_synoema_bin(bench_root)?;

    // Resolve reference paths
    let project_root = bench_root.parent().unwrap_or(bench_root);
    let baseline_ref = project_root.join("docs/llm/synoema.md");
    let compact_ref = project_root.join("docs/llm/synoema-compact.md");

    let configs = vec![
        SizeConfig {
            name: "baseline".into(),
            reference_path: baseline_ref.clone(),
            multi_pass: false,
            max_retries: 0,
        },
        SizeConfig {
            name: "compact".into(),
            reference_path: compact_ref.clone(),
            multi_pass: false,
            max_retries: 0,
        },
        SizeConfig {
            name: "multipass".into(),
            reference_path: compact_ref,
            multi_pass: true,
            max_retries: 2,
        },
    ];

    // Filter to LLM-eligible tasks
    let llm_tasks: Vec<&String> = task_list
        .iter()
        .filter(|t| LLM_TASKS.contains(&t.as_str()))
        .collect();

    let total_ops = models.len() * configs.len() * llm_tasks.len() * repeats;
    let mut done = 0usize;

    let mut all_model_results = Vec::new();

    for model in &models {
        // Ensure model is pulled
        llm::ensure_model(model, verbose)?;

        let mut config_results = Vec::new();

        for config in &configs {
            let reference = if config.reference_path.exists() {
                std::fs::read_to_string(&config.reference_path).unwrap_or_default()
            } else {
                eprintln!(
                    "  Warning: reference not found at {}, using empty",
                    config.reference_path.display()
                );
                String::new()
            };

            let mut task_results = Vec::new();
            let mut total_syntax = 0u32;
            let mut total_type = 0u32;
            let mut total_run = 0u32;
            let mut total_count = 0u32;
            let mut total_tokens = 0u64;

            for task in &llm_tasks {
                let task_dir = bench_root.join("tasks").join(task);
                let prompt_file = task_dir.join("prompt.txt");
                let expected_file = task_dir.join("expected_output.txt");

                let task_prompt = std::fs::read_to_string(&prompt_file).unwrap_or_default();

                let mut syntax_ok = 0u32;
                let mut type_ok = 0u32;
                let mut run_ok = 0u32;
                let mut tokens_out = 0u64;

                for _ in 0..repeats {
                    let full_prompt = format!(
                        "{reference}\n\n{task_prompt}\n\nWrite the complete Synoema program. Use `main = <expr>` as entry point."
                    );

                    let temperatures: Vec<f64> = if config.multi_pass {
                        [0.7, 0.4, 0.2].iter().take(1 + config.max_retries as usize).copied().collect()
                    } else {
                        vec![0.2]
                    };

                    let mut best_validation = ValidationResult {
                        syntax_ok: false,
                        type_ok: false,
                        run_ok: false,
                        error_json: None,
                    };
                    let mut attempt_prompt = full_prompt.clone();
                    let mut attempt_tokens = 0u64;

                    for (attempt, &temp) in temperatures.iter().enumerate() {
                        match generate_ollama(model, &attempt_prompt, temp, verbose) {
                            Ok((response, toks)) => {
                                attempt_tokens += toks;
                                let code = extract_sno_code(&response);

                                if code.is_empty() {
                                    if verbose {
                                        eprintln!("    empty response from {model}");
                                    }
                                    continue;
                                }

                                // Write to temp file and validate
                                let tmp_path = std::env::temp_dir().join(format!(
                                    "synoema_bench_{task}_{}.sno",
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                ));
                                let _ = std::fs::write(&tmp_path, &code);
                                best_validation =
                                    validate_sno(&tmp_path, &expected_file, &synoema_bin);
                                let _ = std::fs::remove_file(&tmp_path);

                                if verbose {
                                    eprintln!(
                                        "    {model}/{}/{task} attempt {}: syn={} type={} run={}",
                                        config.name,
                                        attempt + 1,
                                        best_validation.syntax_ok,
                                        best_validation.type_ok,
                                        best_validation.run_ok,
                                    );
                                }

                                // If passed or no more retries, stop
                                if best_validation.run_ok || !config.multi_pass {
                                    break;
                                }

                                // Multi-pass: build retry prompt with error feedback
                                if let Some(ref err_json) = best_validation.error_json {
                                    let hint = extract_hint(err_json);
                                    attempt_prompt = format!(
                                        "{reference}\n\nYour previous code had an error:\n{hint}\n\nFix the code:\n```sno\n{code}\n```\n\nWrite the corrected complete Synoema program."
                                    );
                                } else {
                                    break;
                                }
                            }
                            Err(e) => {
                                if verbose {
                                    eprintln!("    {model} generation error: {e}");
                                }
                                break;
                            }
                        }
                    }

                    if best_validation.syntax_ok {
                        syntax_ok += 1;
                    }
                    if best_validation.type_ok {
                        type_ok += 1;
                    }
                    if best_validation.run_ok {
                        run_ok += 1;
                    }
                    tokens_out += attempt_tokens;

                    done += 1;
                    eprint!(
                        "\r  [D] {done}/{total_ops} ({:.0}%) — {model} / {} / {task}    ",
                        done as f64 / total_ops as f64 * 100.0,
                        config.name,
                    );
                }

                let runs = repeats as u32;
                total_syntax += syntax_ok;
                total_type += type_ok;
                total_run += run_ok;
                total_count += runs;
                total_tokens += tokens_out;

                task_results.push(SizeTaskResult {
                    task: task.to_string(),
                    syntax_ok,
                    type_ok,
                    run_ok,
                    total_runs: runs,
                    avg_tokens_out: tokens_out as f64 / runs as f64,
                });
            }

            let count = if total_count > 0 { total_count } else { 1 };
            config_results.push(SizeConfigResults {
                config: config.name.clone(),
                tasks: task_results,
                avg_syntax_pct: total_syntax as f64 / count as f64 * 100.0,
                avg_type_pct: total_type as f64 / count as f64 * 100.0,
                avg_run_pct: total_run as f64 / count as f64 * 100.0,
                avg_tokens_out: total_tokens as f64 / count as f64,
            });
        }

        all_model_results.push(SizeModelResults {
            model: model.clone(),
            configs: config_results,
        });
    }

    eprintln!();
    Ok(SizeResults {
        models: all_model_results,
    })
}

/// Find the synoema binary (release preferred, then debug).
fn find_synoema_bin(bench_root: &Path) -> Result<PathBuf, String> {
    let project_root = bench_root.parent().unwrap_or(bench_root);
    let release = project_root.join("lang/target/release/synoema");
    if release.exists() {
        return Ok(release);
    }
    let debug = project_root.join("lang/target/debug/synoema");
    if debug.exists() {
        return Ok(debug);
    }
    // Try PATH
    if Command::new("synoema").arg("--version").output().is_ok() {
        return Ok(PathBuf::from("synoema"));
    }
    Err("synoema binary not found. Run: cd lang && cargo build --release -p synoema-repl".into())
}
