use crate::phases::{llm, runtime, tokens};
use std::time::Duration;

pub fn print_header(phases: &[String], tasks: &[String], languages: &[&str]) {
    eprintln!();
    eprintln!("  SYNOEMA BENCHMARK SUITE");
    eprintln!("  ═══════════════════════");
    eprintln!("  Phases:    {}", phases.join(", "));
    eprintln!("  Tasks:     {}", tasks.len());
    eprintln!("  Languages: {}", languages.join(", "));
    eprintln!();
}

pub fn print_phase_start(id: &str, name: &str, mode: &str) {
    eprintln!("  Phase {id}: {name} ({mode})");
    eprintln!("  {}", "─".repeat(50));
}

pub fn print_token_results(results: &tokens::TokenResults) {
    eprintln!();
    eprintln!("  Token Efficiency Summary");
    eprintln!("  {:12} {:>10} {:>12}", "Language", "Avg Tokens", "vs Synoema");
    eprintln!("  {}", "─".repeat(36));

    let sno_avg = results.averages.get("synoema").copied().unwrap_or(1.0);

    let mut sorted: Vec<_> = results.averages.iter().collect();
    sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    for (lang, avg) in &sorted {
        let vs = if *lang == "synoema" {
            "baseline".to_string()
        } else {
            format!("+{:.0}%", (*avg - sno_avg) / sno_avg * 100.0)
        };
        eprintln!("  {:12} {:>10.1} {:>12}", lang, avg, vs);
    }
    eprintln!();
}

pub fn print_runtime_results(results: &runtime::RuntimeResults) {
    eprintln!();
    eprintln!("  Runtime Performance Summary");
    eprintln!("  {:12} {:>10} {:>12}", "Language", "Avg ms", "vs Synoema");
    eprintln!("  {}", "─".repeat(36));

    let sno_avg = results.averages.get("synoema").copied().unwrap_or(1.0);

    let mut sorted: Vec<_> = results.averages.iter().collect();
    sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    for &(ref lang, avg) in &sorted {
        let vs = if lang.as_str() == "synoema" {
            "baseline".to_string()
        } else if *avg < sno_avg {
            format!("{:.1}x faster", sno_avg / avg)
        } else {
            format!("{:.1}x slower", avg / sno_avg)
        };
        eprintln!("  {:12} {:>10.1} {:>12}", lang, avg, vs);
    }
    eprintln!();
}

pub fn print_llm_results(results: &llm::LlmResults) {
    eprintln!();
    eprintln!("  LLM Code Generation Summary (all models avg)");
    eprintln!(
        "  {:12} {:>9} {:>9} {:>10} {:>10}",
        "Language", "Syntax%", "Correct%", "Avg Tok", "Avg Cost"
    );
    eprintln!("  {}", "─".repeat(54));

    let mut sorted: Vec<_> = results.language_averages.iter().collect();
    sorted.sort_by(|a, b| b.1.syntax_pct.partial_cmp(&a.1.syntax_pct).unwrap());

    for (lang, avg) in &sorted {
        eprintln!(
            "  {:12} {:>8.1}% {:>8.1}% {:>10.0} {:>9.4}$",
            lang, avg.syntax_pct, avg.correct_pct, avg.avg_tokens, avg.avg_cost
        );
    }
    eprintln!();
}

pub fn print_footer(results_dir: &str, elapsed: Duration) {
    let secs = elapsed.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    eprintln!("  Total time: {mins}m {secs}s");
    eprintln!("  Results: {results_dir}");
    eprintln!();
}
