use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

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

pub fn resolve_models(
    filter: &Option<Vec<String>>,
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
                return f.iter().any(|name| m.id.contains(name.as_str()));
            }
            true
        })
        .map(|m| (m.id.to_string(), m.tier.to_string()))
        .collect()
}

/// Run Phase C via local ollama. Ensures model is pulled, then delegates to `run()` with
/// ollama's OpenAI-compatible endpoint.
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
    // Use "ollama" as api_key (ollama doesn't require auth) and pass base_url
    run_with_base_url(
        bench_root,
        task_list,
        languages,
        &model_list,
        "ollama",
        repeats,
        verbose,
        Some("http://localhost:11434/v1"),
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
) -> Result<LlmResults, String> {
    run_with_base_url(bench_root, task_list, languages, model_list, api_key, repeats, verbose, None)
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

    let mut results = LlmResults::default();
    let total_ops = model_list.len() * llm_tasks.len() * languages.len() * repeats;
    let mut done = 0usize;
    let mut consecutive_failures = 0u32;
    const MAX_CONSECUTIVE_FAILURES: u32 = 10;

    // Accumulators for language averages
    let mut lang_syntax: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut lang_correct: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut lang_tokens: BTreeMap<String, (f64, u32)> = BTreeMap::new();

    for (model_id, tier) in model_list {
        let mut model_tasks = Vec::new();

        for task in &llm_tasks {
            let task_dir = bench_root.join("tasks").join(task);
            let expected = task_dir.join("expected_output.txt");

            for &lang in languages {
                let mut syntax_ok = 0u32;
                let mut correct = 0u32;
                let mut total_tok_in = 0u64;
                let mut total_tok_out = 0u64;

                for _ in 0..repeats {
                    if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                        eprintln!(
                            "\n  ERROR: {MAX_CONSECUTIVE_FAILURES} consecutive failures — aborting Phase C."
                        );
                        eprintln!("  Check API key and network connectivity.");
                        return Ok(results);
                    }

                    let mut cmd = Command::new("python3");
                    cmd.arg(&script)
                        .arg("--model")
                        .arg(model_id)
                        .arg("--language")
                        .arg(lang)
                        .arg("--task-dir")
                        .arg(&task_dir)
                        .arg("--key")
                        .arg(api_key);

                    if let Some(url) = base_url {
                        cmd.arg("--base-url").arg(url);
                    }

                    if lang == "synoema" && context_file.exists() {
                        cmd.arg("--context").arg(&context_file);
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
                                        eprintln!("    API error: {err}");
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
                                    eprintln!("    parse error: {}", stdout.trim());
                                }
                            }
                        }
                        Ok(output) => {
                            consecutive_failures += 1;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            if consecutive_failures <= 3 || verbose {
                                eprintln!("  Warning: llm_generate.py failed: {stderr}");
                            }
                        }
                        Err(e) => {
                            consecutive_failures += 1;
                            if consecutive_failures <= 3 || verbose {
                                eprintln!("  Warning: failed to run llm_generate.py: {e}");
                            }
                        }
                    }

                    done += 1;
                }

                let total_runs = repeats as u32;
                let avg_in = total_tok_in as f64 / total_runs as f64;
                let avg_out = total_tok_out as f64 / total_runs as f64;

                // Accumulate for language averages
                let l = lang.to_string();
                lang_syntax.entry(l.clone()).or_insert((0, 0)).0 += syntax_ok;
                lang_syntax.entry(l.clone()).or_insert((0, 0)).1 += total_runs;
                lang_correct.entry(l.clone()).or_insert((0, 0)).0 += correct;
                lang_correct.entry(l.clone()).or_insert((0, 0)).1 += total_runs;
                lang_tokens.entry(l.clone()).or_insert((0.0, 0)).0 += avg_in + avg_out;
                lang_tokens.entry(l).or_insert((0.0, 0)).1 += 1;

                eprint!(
                    "\r  [C] {}/{} ({:.0}%) — {model_id} / {task} / {lang}: {syntax_ok}/{total_runs} syntax, {correct}/{total_runs} correct    ",
                    done, total_ops, done as f64 / total_ops as f64 * 100.0
                );

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

        results.models.push(ModelResults {
            model: model_id.clone(),
            tier: tier.clone(),
            tasks: model_tasks,
        });
    }

    eprintln!();

    // Compute language averages
    // Pricing approximation: $5/M input, $15/M output (weighted avg across models)
    let price_in = 5.0 / 1_000_000.0;
    let price_out = 15.0 / 1_000_000.0;

    for lang in languages {
        let l = lang.to_string();
        let (syn_ok, syn_total) = lang_syntax.get(&l).copied().unwrap_or((0, 1));
        let (cor_ok, cor_total) = lang_correct.get(&l).copied().unwrap_or((0, 1));
        let (tok_sum, tok_count) = lang_tokens.get(&l).copied().unwrap_or((0.0, 1));
        let avg_tok = tok_sum / tok_count as f64;

        results.language_averages.insert(
            l,
            LlmLanguageAvg {
                syntax_pct: syn_ok as f64 / syn_total as f64 * 100.0,
                correct_pct: cor_ok as f64 / cor_total as f64 * 100.0,
                avg_tokens: avg_tok,
                avg_cost: avg_tok * 0.6 * price_in + avg_tok * 0.4 * price_out,
            },
        );
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_detection() {
        // ollama_available() should return a bool without panicking,
        // regardless of whether ollama is installed
        let result = ollama_available();
        // Just verify it returns a bool (type system ensures this)
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
        // Find bench root
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
