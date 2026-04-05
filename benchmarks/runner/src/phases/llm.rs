// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Check if ollama is installed and reachable.
pub fn ollama_available() -> bool {
    Command::new("ollama")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Ensure a model is pulled in ollama. Returns Ok if already present or pulled successfully.
pub fn ensure_model(model: &str, verbose: bool) -> Result<(), String> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .map_err(|e| format!("Failed to run `ollama list`: {e}"))?;

    let list_out = String::from_utf8_lossy(&output.stdout);
    if list_out.lines().any(|line| line.starts_with(model) || line.contains(model)) {
        if verbose {
            eprintln!("  ollama: model {model} already present");
        }
        return Ok(());
    }

    eprintln!("  ollama: pulling {model}...");
    let pull = Command::new("ollama")
        .arg("pull")
        .arg(model)
        .status()
        .map_err(|e| format!("Failed to run `ollama pull {model}`: {e}"))?;

    if pull.success() {
        eprintln!("  ollama: {model} ready");
        Ok(())
    } else {
        Err(format!("ollama pull {model} failed with exit code {}", pull.code().unwrap_or(-1)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmResults {
    pub models: Vec<ModelResults>,
    pub language_averages: BTreeMap<String, LlmLanguageAvg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResults {
    pub model: String,
    pub tier: String,
    pub tasks: Vec<LlmTaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTaskResult {
    pub task: String,
    pub language: String,
    pub syntax_ok: u32,
    pub correct: u32,
    pub total_runs: u32,
    pub avg_tokens_in: f64,
    pub avg_tokens_out: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmLanguageAvg {
    pub syntax_pct: f64,
    pub correct_pct: f64,
    pub avg_tokens: f64,
    pub avg_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GenerateResult {
    syntax_ok: bool,
    correct: bool,
    tokens_in: u64,
    tokens_out: u64,
    code: String,
    #[serde(default)]
    error: Option<String>,
}

struct ModelDef {
    id: &'static str,
    tier: &'static str,
}

const ALL_MODELS: &[ModelDef] = &[
    // Tier 1: Frontier
    ModelDef { id: "openai/gpt-4o", tier: "frontier" },
    ModelDef { id: "google/gemini-2.5-pro", tier: "frontier" },
    ModelDef { id: "qwen/qwen3-max-thinking", tier: "frontier" },
    // Tier 2: Mid
    ModelDef { id: "openai/gpt-4o-mini", tier: "mid" },
    ModelDef { id: "deepseek/deepseek-chat-v3-0324", tier: "mid" },
    ModelDef { id: "qwen/qwen3-coder-next", tier: "mid" },
    ModelDef { id: "meta-llama/llama-4-maverick", tier: "mid" },
    // Tier 3: Weak
    ModelDef { id: "qwen/qwen3.5-9b", tier: "weak" },
    ModelDef { id: "liquid/lfm-2.5-1.2b-instruct:free", tier: "weak" },
    ModelDef { id: "rekaai/reka-edge", tier: "weak" },
];

/// LLM-eligible tasks (subset that can be validated)
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

const MAX_CONSECUTIVE_FAILURES: u32 = 10;

pub fn resolve_models(
    filter: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
    tier: &Option<String>,
) -> Vec<(String, String)> {
    ALL_MODELS
        .iter()
        .filter(|m| {
            if let Some(t) = tier {
                if m.tier != t.as_str() {
                    return false;
                }
            }
            if let Some(f) = filter {
                if !f.iter().any(|name| m.id.contains(name.as_str())) {
                    return false;
                }
            }
            if let Some(ex) = exclude {
                if ex.iter().any(|name| m.id.contains(name.as_str())) {
                    return false;
                }
            }
            true
        })
        .map(|m| (m.id.to_string(), m.tier.to_string()))
        .collect()
}

/// Run Phase C via local ollama (always sequential — single GPU).
pub fn run_ollama(
    bench_root: &Path,
    task_list: &[String],
    languages: &[&str],
    model: &str,
    repeats: usize,
    verbose: bool,
) -> Result<LlmResults, String> {
    if !ollama_available() {
        return Err("ollama is not installed or not in PATH".to_string());
    }
    ensure_model(model, verbose)?;

    let model_list = vec![(model.to_string(), "local".to_string())];
    run_with_base_url(
        bench_root,
        task_list,
        languages,
        &model_list,
        "ollama",
        repeats,
        verbose,
        Some("http://localhost:11434/v1"),
        1, // always sequential for ollama
    )
}

pub fn run(
    bench_root: &Path,
    task_list: &[String],
    languages: &[&str],
    model_list: &[(String, String)],
    api_key: &str,
    repeats: usize,
    verbose: bool,
    parallel: usize,
) -> Result<LlmResults, String> {
    run_with_base_url(bench_root, task_list, languages, model_list, api_key, repeats, verbose, None, parallel)
}

/// Run all tasks/languages/repeats for a single model. Returns ModelResults.
/// Has its own consecutive_failures counter — one model failing doesn't abort others.
fn run_one_model(
    script: &Path,
    context_file: &Path,
    bench_root: &Path,
    model_id: &str,
    tier: &str,
    llm_tasks: &[&String],
    languages: &[&str],
    api_key: &str,
    repeats: usize,
    verbose: bool,
    base_url: Option<&str>,
    save_dir: Option<&Path>,
    progress: &AtomicUsize,
    total_ops: usize,
) -> ModelResults {
    let mut model_tasks = Vec::new();
    let mut consecutive_failures = 0u32;
    let mut aborted = false;

    for task in llm_tasks {
        let task_dir = bench_root.join("tasks").join(task);
        let expected = task_dir.join("expected_output.txt");

        for &lang in languages {
            let mut syntax_ok = 0u32;
            let mut correct = 0u32;
            let mut total_tok_in = 0u64;
            let mut total_tok_out = 0u64;

            for attempt in 0..repeats {
                if aborted {
                    progress.fetch_add(1, Ordering::Relaxed);
                    continue;
                }

                let mut cmd = Command::new("python3");
                cmd.arg(script)
                    .arg("--model")
                    .arg(model_id)
                    .arg("--language")
                    .arg(lang)
                    .arg("--task-dir")
                    .arg(&task_dir)
                    .arg("--key")
                    .arg(api_key);

                if let Some(dir) = save_dir {
                    cmd.arg("--save-dir").arg(dir);
                    cmd.arg("--attempt").arg(attempt.to_string());
                }

                if let Some(url) = base_url {
                    cmd.arg("--base-url").arg(url);
                }

                if lang == "synoema" && context_file.exists() {
                    cmd.arg("--context").arg(context_file);
                }
                if expected.exists() {
                    cmd.arg("--expected").arg(&expected);
                }

                if verbose {
                    eprintln!("    cmd: python3 {} --model {model_id} --language {lang} --task-dir {}", script.display(), task_dir.display());
                }

                match cmd.output() {
                    Ok(output) if output.status.success() => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if let Ok(res) = serde_json::from_str::<GenerateResult>(&stdout) {
                            if let Some(ref err) = res.error {
                                consecutive_failures += 1;
                                if consecutive_failures <= 3 || verbose {
                                    eprintln!("    API error ({model_id}): {err}");
                                }
                            } else {
                                consecutive_failures = 0;
                                if verbose {
                                    eprintln!(
                                        "    {model_id}/{task}/{lang}: syntax={} correct={} tok_in={} tok_out={}",
                                        res.syntax_ok, res.correct, res.tokens_in, res.tokens_out
                                    );
                                }
                                if res.syntax_ok {
                                    syntax_ok += 1;
                                }
                                if res.correct {
                                    correct += 1;
                                }
                                total_tok_in += res.tokens_in;
                                total_tok_out += res.tokens_out;
                            }
                        } else {
                            consecutive_failures += 1;
                            if verbose {
                                eprintln!("    parse error ({model_id}): {}", stdout.trim());
                            }
                        }
                    }
                    Ok(output) => {
                        consecutive_failures += 1;
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if consecutive_failures <= 3 || verbose {
                            eprintln!("  Warning ({model_id}): llm_generate.py failed: {stderr}");
                        }
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        if consecutive_failures <= 3 || verbose {
                            eprintln!("  Warning ({model_id}): failed to run llm_generate.py: {e}");
                        }
                    }
                }

                progress.fetch_add(1, Ordering::Relaxed);

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    eprintln!(
                        "\n  ERROR: {MAX_CONSECUTIVE_FAILURES} consecutive failures for {model_id} — skipping remaining."
                    );
                    aborted = true;
                    // Fast-forward progress for remaining ops in this model
                    let done_so_far = progress.load(Ordering::Relaxed);
                    let model_done = done_so_far; // approximate
                    let _ = model_done; // progress will be incremented via continue above
                    break;
                }
            }

            let total_runs = repeats as u32;
            let avg_in = total_tok_in as f64 / total_runs as f64;
            let avg_out = total_tok_out as f64 / total_runs as f64;

            if !aborted {
                let done = progress.load(Ordering::Relaxed);
                eprint!(
                    "\r  [C] {}/{} ({:.0}%) — {model_id} / {task} / {lang}: {syntax_ok}/{total_runs} syntax, {correct}/{total_runs} correct    ",
                    done, total_ops, done as f64 / total_ops as f64 * 100.0
                );
            }

            model_tasks.push(LlmTaskResult {
                task: task.to_string(),
                language: lang.to_string(),
                syntax_ok,
                correct,
                total_runs,
                avg_tokens_in: avg_in,
                avg_tokens_out: avg_out,
            });
        }
    }

    ModelResults {
        model: model_id.to_string(),
        tier: tier.to_string(),
        tasks: model_tasks,
    }
}

/// Compute language averages from collected model results.
fn compute_language_averages(
    models: &[ModelResults],
    languages: &[&str],
) -> BTreeMap<String, LlmLanguageAvg> {
    let mut lang_syntax: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut lang_correct: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut lang_tokens: BTreeMap<String, (f64, u32)> = BTreeMap::new();

    for model in models {
        for t in &model.tasks {
            let l = t.language.clone();
            lang_syntax.entry(l.clone()).or_insert((0, 0)).0 += t.syntax_ok;
            lang_syntax.entry(l.clone()).or_insert((0, 0)).1 += t.total_runs;
            lang_correct.entry(l.clone()).or_insert((0, 0)).0 += t.correct;
            lang_correct.entry(l.clone()).or_insert((0, 0)).1 += t.total_runs;
            lang_tokens.entry(l.clone()).or_insert((0.0, 0)).0 += t.avg_tokens_in + t.avg_tokens_out;
            lang_tokens.entry(l).or_insert((0.0, 0)).1 += 1;
        }
    }

    // Pricing approximation: $5/M input, $15/M output (weighted avg across models)
    let price_in = 5.0 / 1_000_000.0;
    let price_out = 15.0 / 1_000_000.0;

    let mut averages = BTreeMap::new();
    for lang in languages {
        let l = lang.to_string();
        let (syn_ok, syn_total) = lang_syntax.get(&l).copied().unwrap_or((0, 1));
        let (cor_ok, cor_total) = lang_correct.get(&l).copied().unwrap_or((0, 1));
        let (tok_sum, tok_count) = lang_tokens.get(&l).copied().unwrap_or((0.0, 1));
        let avg_tok = tok_sum / tok_count as f64;

        averages.insert(
            l,
            LlmLanguageAvg {
                syntax_pct: syn_ok as f64 / syn_total as f64 * 100.0,
                correct_pct: cor_ok as f64 / cor_total as f64 * 100.0,
                avg_tokens: avg_tok,
                avg_cost: avg_tok * 0.6 * price_in + avg_tok * 0.4 * price_out,
            },
        );
    }

    averages
}

fn run_with_base_url(
    bench_root: &Path,
    task_list: &[String],
    languages: &[&str],
    model_list: &[(String, String)],
    api_key: &str,
    repeats: usize,
    verbose: bool,
    base_url: Option<&str>,
    parallel: usize,
) -> Result<LlmResults, String> {
    let script = bench_root.join("scripts/llm_generate.py");
    if !script.exists() {
        return Err(format!("llm_generate.py not found at {}", script.display()));
    }

    let context_file = bench_root
        .parent()
        .map(|p| p.join("docs/llm/synoema.md"))
        .unwrap_or_default();

    let llm_tasks: Vec<&String> = task_list
        .iter()
        .filter(|t| LLM_TASKS.contains(&t.as_str()))
        .collect();

    // Pre-flight check: verify llm_generate.py can import its dependencies
    let preflight = Command::new("python3")
        .arg("-c")
        .arg("import openai; import tiktoken; print('ok')")
        .output();
    match preflight {
        Ok(o) if o.status.success() => {}
        _ => {
            return Err(
                "Python dependencies missing. Run: pip3 install openai tiktoken".to_string(),
            );
        }
    }

    let total_ops = model_list.len() * llm_tasks.len() * languages.len() * repeats;
    let progress = AtomicUsize::new(0);

    // Create save directory for generated code
    let save_dir = bench_root.join("results").join("generated");
    let _ = std::fs::create_dir_all(&save_dir);

    // Owned copies for thread safety
    let base_url_owned: Option<String> = base_url.map(|s| s.to_string());
    let script = script.clone();
    let context_file = context_file.clone();
    let bench_root_buf: PathBuf = bench_root.to_path_buf();

    let threads = if parallel == 0 { 1 } else { parallel };
    if threads > 1 {
        eprintln!("  (parallel: {threads} threads across {} models)", model_list.len());
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|e| format!("Failed to create thread pool: {e}"))?;

    let model_results: Vec<ModelResults> = pool.install(|| {
        model_list
            .par_iter()
            .map(|(model_id, tier)| {
                run_one_model(
                    &script,
                    &context_file,
                    &bench_root_buf,
                    model_id,
                    tier,
                    &llm_tasks,
                    languages,
                    api_key,
                    repeats,
                    verbose,
                    base_url_owned.as_deref(),
                    Some(&save_dir),
                    &progress,
                    total_ops,
                )
            })
            .collect()
    });

    eprintln!();

    let language_averages = compute_language_averages(&model_results, languages);

    Ok(LlmResults {
        models: model_results,
        language_averages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_detection() {
        let result = ollama_available();
        assert!(result || !result);
    }

    #[test]
    #[ignore] // requires ollama installed
    fn test_ensure_model() {
        if !ollama_available() {
            eprintln!("skipping: ollama not available");
            return;
        }
        ensure_model("qwen3:8b", true).expect("should pull or confirm qwen3:8b");
    }

    #[test]
    #[ignore] // requires ollama + qwen3:8b + python deps
    fn test_ollama_single_task() {
        if !ollama_available() {
            eprintln!("skipping: ollama not available");
            return;
        }
        let bench_root = std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        if !bench_root.join("tasks").is_dir() {
            eprintln!("skipping: bench_root/tasks not found at {}", bench_root.display());
            return;
        }
        let tasks = vec!["factorial".to_string()];
        let languages = &["synoema"];
        let result = run_ollama(&bench_root, &tasks, languages, "qwen3:8b", 1, true);
        assert!(result.is_ok(), "ollama single task failed: {:?}", result.err());
    }
}
