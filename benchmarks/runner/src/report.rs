// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use crate::phases::{llm, runtime, size, tokens};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AllResults {
    pub tokens: Option<tokens::TokenResults>,
    pub runtime: Option<runtime::RuntimeResults>,
    pub llm: Option<llm::LlmResults>,
    pub size: Option<size::SizeResults>,
}

pub fn write_results(bench_root: &Path, results: &AllResults, elapsed: Duration) -> String {
    let date = chrono_now();
    let run_dir = bench_root
        .join("results")
        .join(format!("{date}_run_001"));

    // Find next available run number
    let results_base = bench_root.join("results");
    let _ = std::fs::create_dir_all(&results_base);
    let mut run_num = 1;
    loop {
        let candidate = results_base.join(format!("{date}_run_{run_num:03}"));
        if !candidate.exists() {
            let _ = std::fs::create_dir_all(&candidate);
            return write_to_dir(&candidate, results, elapsed);
        }
        run_num += 1;
        if run_num > 999 {
            break;
        }
    }
    write_to_dir(&run_dir, results, elapsed)
}

fn write_to_dir(dir: &Path, results: &AllResults, elapsed: Duration) -> String {
    let _ = std::fs::create_dir_all(dir);

    // raw.json
    if let Ok(json) = serde_json::to_string_pretty(results) {
        let _ = std::fs::write(dir.join("raw.json"), json);
    }

    // summary.txt
    let summary = build_summary(results, elapsed);
    let _ = std::fs::write(dir.join("summary.txt"), &summary);

    // details.txt
    let details = build_details(results, elapsed);
    let _ = std::fs::write(dir.join("details.txt"), &details);

    // Print summary to stdout
    println!("{summary}");

    dir.to_string_lossy().to_string()
}

fn build_summary(results: &AllResults, elapsed: Duration) -> String {
    let mut s = String::new();
    let date = chrono_now();

    s.push_str(&format!("SYNOEMA BENCHMARK RESULTS\n"));
    s.push_str(&"=".repeat(60));
    s.push_str(&format!("\nDate: {date}\n"));
    s.push_str(&format!(
        "Duration: {}m {}s\n\n",
        elapsed.as_secs() / 60,
        elapsed.as_secs() % 60
    ));

    if let Some(ref tok) = results.tokens {
        s.push_str("A. TOKEN EFFICIENCY\n");
        s.push_str(&"-".repeat(50));
        s.push_str(&format!("\n{:12} {:>10} {:>12}\n", "Language", "Avg Tokens", "vs Synoema"));

        let sno = tok.averages.get("synoema").copied().unwrap_or(1.0);
        let mut sorted: Vec<_> = tok.averages.iter().collect();
        sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        for &(ref lang, avg) in &sorted {
            let vs = if lang.as_str() == "synoema" {
                "baseline".into()
            } else {
                format!("+{:.0}%", (avg - sno) / sno * 100.0)
            };
            s.push_str(&format!("{:12} {:>10.1} {:>12}\n", lang, avg, vs));
        }
        s.push('\n');
    }

    if let Some(ref rt) = results.runtime {
        s.push_str("B. RUNTIME PERFORMANCE (median of measured runs)\n");
        s.push_str(&"-".repeat(50));
        s.push_str(&format!("\n{:12} {:>10} {:>12}\n", "Language", "Avg ms", "vs Synoema"));

        let sno = rt.averages.get("synoema").copied().unwrap_or(1.0);
        let mut sorted: Vec<_> = rt.averages.iter().collect();
        sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        for &(ref lang, avg) in &sorted {
            let vs = if lang.as_str() == "synoema" {
                "baseline".into()
            } else if *avg < sno {
                format!("{:.1}x faster", sno / avg)
            } else {
                format!("{:.1}x slower", avg / sno)
            };
            s.push_str(&format!("{:12} {:>10.1} {:>12}\n", lang, avg, vs));
        }
        s.push('\n');
    }

    if let Some(ref llm) = results.llm {
        s.push_str("C. LLM CODE GENERATION (all models avg)\n");
        s.push_str(&"-".repeat(60));
        s.push_str(&format!(
            "\n{:12} {:>9} {:>9} {:>10} {:>10}\n",
            "Language", "Syntax%", "Correct%", "Avg Tok", "Cost($)"
        ));

        let mut sorted: Vec<_> = llm.language_averages.iter().collect();
        sorted.sort_by(|a, b| b.1.syntax_pct.partial_cmp(&a.1.syntax_pct).unwrap());
        for (lang, avg) in &sorted {
            s.push_str(&format!(
                "{:12} {:>8.1}% {:>8.1}% {:>10.0} {:>9.4}\n",
                lang, avg.syntax_pct, avg.correct_pct, avg.avg_tokens, avg.avg_cost
            ));
        }
        s.push('\n');
    }

    if let Some(ref sz) = results.size {
        s.push_str("D. MODEL SIZE REDUCTION\n");
        s.push_str(&"-".repeat(70));
        s.push_str(&format!(
            "\n{:24} {:>10} {:>9} {:>9} {:>9} {:>10}\n",
            "Model / Config", "Syntax%", "Type%", "Run%", "Avg Tok", "Tasks"
        ));

        for mr in &sz.models {
            for cr in &mr.configs {
                let label = format!("{} / {}", mr.model, cr.config);
                s.push_str(&format!(
                    "{:24} {:>9.1}% {:>8.1}% {:>8.1}% {:>9.0} {:>10}\n",
                    label,
                    cr.avg_syntax_pct,
                    cr.avg_type_pct,
                    cr.avg_run_pct,
                    cr.avg_tokens_out,
                    cr.tasks.len(),
                ));
            }
        }
        s.push('\n');
    }

    s
}

fn build_details(results: &AllResults, elapsed: Duration) -> String {
    let mut s = String::new();
    let date = chrono_now();

    s.push_str(&"=".repeat(70));
    s.push_str(&format!("\n SYNOEMA BENCHMARK — DETAILED RESULTS\n"));
    s.push_str(&format!(" Run date: {date}\n"));
    s.push_str(&"=".repeat(70));
    s.push_str("\n\n");

    // Section A details
    if let Some(ref tok) = results.tokens {
        s.push_str("SECTION A: TOKEN EFFICIENCY\n");
        s.push_str(&"=".repeat(40));
        s.push_str("\n\n");

        for task_tok in &tok.tasks {
            s.push_str(&format!("A. {}\n", task_tok.task));
            s.push_str(&"-".repeat(40));
            s.push_str(&format!(
                "\n{:>14} {:>8} {:>12}\n",
                "Language", "Tokens", "vs Synoema"
            ));

            let sno = task_tok.counts.get("synoema").copied().unwrap_or(1);
            let mut sorted: Vec<_> = task_tok.counts.iter().collect();
            sorted.sort_by_key(|(_, v)| *v);

            for (lang, count) in &sorted {
                let vs = if *lang == "synoema" {
                    "baseline".into()
                } else {
                    format!("+{:.0}%", (**count as f64 - sno as f64) / sno as f64 * 100.0)
                };
                s.push_str(&format!("{:>14} {:>8} {:>12}\n", lang, count, vs));
            }
            s.push('\n');
        }
    }

    // Section B details
    if let Some(ref rt) = results.runtime {
        s.push_str("\nSECTION B: RUNTIME PERFORMANCE\n");
        s.push_str(&"=".repeat(40));
        s.push_str("\n\n");

        for task_rt in &rt.tasks {
            s.push_str(&format!("B. {}\n", task_rt.task));
            s.push_str(&"-".repeat(60));
            s.push_str(&format!(
                "\n{:>14} {:>10} {:>10} {:>10} {:>12}\n",
                "Language", "median", "p5", "p95", "vs Synoema"
            ));

            let sno_median = task_rt
                .measurements
                .get("synoema")
                .map(|m| m.median_ms)
                .unwrap_or(1.0);

            let mut sorted: Vec<_> = task_rt.measurements.iter().collect();
            sorted.sort_by(|a, b| a.1.median_ms.partial_cmp(&b.1.median_ms).unwrap());

            for (lang, m) in &sorted {
                let vs = if *lang == "synoema" {
                    "baseline".into()
                } else if m.median_ms < sno_median {
                    format!("{:.1}x faster", sno_median / m.median_ms)
                } else {
                    format!("{:.1}x slower", m.median_ms / sno_median)
                };
                s.push_str(&format!(
                    "{:>14} {:>9.1}ms {:>9.1}ms {:>9.1}ms {:>12}\n",
                    lang, m.median_ms, m.p5_ms, m.p95_ms, vs
                ));
            }
            s.push('\n');
        }

        s.push_str("Methodology: 3 warm-up discarded, 5 measured runs, median reported.\n");
        s.push_str("C++ compiled with g++ -O2. TypeScript via tsx.\n\n");
    }

    // Section C details
    if let Some(ref llm) = results.llm {
        s.push_str("\nSECTION C: LLM CODE GENERATION\n");
        s.push_str(&"=".repeat(40));
        s.push_str("\n\n");

        for model_res in &llm.models {
            s.push_str(&format!(
                "Model: {} (tier: {})\n",
                model_res.model, model_res.tier
            ));
            s.push_str(&"-".repeat(60));
            s.push_str(&format!(
                "\n{:>14} {:>12} {:>8} {:>9} {:>9} {:>9}\n",
                "Task", "Language", "Syntax", "Correct", "Tok In", "Tok Out"
            ));

            for task in &model_res.tasks {
                s.push_str(&format!(
                    "{:>14} {:>12} {:>5}/{:<3} {:>5}/{:<3} {:>9.0} {:>9.0}\n",
                    task.task,
                    task.language,
                    task.syntax_ok,
                    task.total_runs,
                    task.correct,
                    task.total_runs,
                    task.avg_tokens_in,
                    task.avg_tokens_out
                ));
            }
            s.push('\n');
        }
    }

    // Section D: Environment
    s.push_str("\nSECTION D: ENVIRONMENT\n");
    s.push_str(&"=".repeat(40));
    s.push_str("\n");
    s.push_str(&format!(
        "Duration: {}m {}s\n",
        elapsed.as_secs() / 60,
        elapsed.as_secs() % 60
    ));

    s
}

fn chrono_now() -> String {
    // Simple date without chrono dependency
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok();
    output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}
