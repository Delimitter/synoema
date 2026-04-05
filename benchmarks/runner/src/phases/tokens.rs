// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenResults {
    pub tasks: Vec<TaskTokens>,
    pub averages: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTokens {
    pub task: String,
    pub counts: BTreeMap<String, u64>,
}

pub fn run(
    bench_root: &Path,
    task_list: &[String],
    languages: &[&str],
    verbose: bool,
) -> Result<TokenResults, String> {
    let script = bench_root.join("scripts/token_count.py");
    if !script.exists() {
        return Err(format!("token_count.py not found at {}", script.display()));
    }

    let mut results = TokenResults::default();
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    let mut count = 0u64;

    for task in task_list {
        let task_dir = bench_root.join("tasks").join(task);
        if !task_dir.is_dir() {
            continue;
        }

        if verbose {
            eprintln!("    cmd: python3 {} {}", script.display(), task_dir.display());
        }

        let output = Command::new("python3")
            .arg(&script)
            .arg(&task_dir)
            .output()
            .map_err(|e| format!("Failed to run token_count.py: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("  Warning: token_count.py failed for {task}: {stderr}");
            continue;
        }

        if verbose {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                eprintln!("    stderr: {}", stderr.trim());
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let counts: BTreeMap<String, u64> =
            serde_json::from_str(&stdout).map_err(|e| format!("JSON parse error for {task}: {e}"))?;

        // Filter to requested languages only
        let filtered: BTreeMap<String, u64> = counts
            .into_iter()
            .filter(|(k, _)| languages.contains(&k.as_str()))
            .collect();

        for (lang, tok) in &filtered {
            *totals.entry(lang.clone()).or_insert(0) += tok;
        }
        count += 1;

        eprint!("  [A] {task}: ");
        for (lang, tok) in &filtered {
            eprint!("{lang}={tok} ");
        }
        eprintln!();

        results.tasks.push(TaskTokens {
            task: task.clone(),
            counts: filtered,
        });
    }

    if count > 0 {
        for (lang, total) in &totals {
            results
                .averages
                .insert(lang.clone(), *total as f64 / count as f64);
        }
    }

    Ok(results)
}
