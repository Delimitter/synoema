#!/usr/bin/env python3
"""Phase D: Model Size Reduction Benchmark — local llama.cpp inference with GBNF.

Runs code-generation tasks against a local llama-server (OpenAI-compatible API)
across languages (synoema, python, haskell) and modes (zero-shot, few-shot,
constrained). Validates syntax and correctness, outputs per-generation JSON.

Requires: requests, tiktoken.
Does NOT manage llama-server — start it manually before running.

Usage:
    python3 size_benchmark.py --tasks-dir ../tasks --output-dir ./results
    python3 size_benchmark.py --base-url http://localhost:8090 --languages synoema python --modes zero-shot few-shot
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

try:
    import requests
except ImportError:
    print("Error: requests not installed. Run: pip install requests", file=sys.stderr)
    sys.exit(1)

try:
    import tiktoken
except ImportError:
    print("Error: tiktoken not installed. Run: pip install tiktoken", file=sys.stderr)
    sys.exit(1)

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

LANGUAGES = ["synoema", "python", "haskell"]
MODES = ["zero-shot", "few-shot", "constrained"]

LANG_EXTENSIONS = {
    "synoema": ".sno",
    "python": ".py",
    "haskell": ".hs",
}

LANG_DISPLAY = {
    "synoema": "Synoema",
    "python": "Python",
    "haskell": "Haskell",
}

# Context file names inside --context-dir
LANG_CONTEXT_FILES = {
    "synoema": "synoema.md",
    "python": "python.md",
    "haskell": "haskell.md",
}

# Example task used for few-shot demonstrations
FEWSHOT_EXAMPLE_TASK = "factorial"

ENC = tiktoken.get_encoding("cl100k_base")

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def log(msg: str) -> None:
    """Print progress to stderr."""
    print(msg, file=sys.stderr, flush=True)


def count_tokens(text: str) -> int:
    return len(ENC.encode(text))


def strip_fences(code: str) -> str:
    """Remove markdown code fences if the model wrapped the output."""
    if not code.startswith("```"):
        return code
    lines = code.split("\n")
    if lines[-1].strip() == "```":
        lines = lines[1:-1]
    else:
        lines = lines[1:]
    return "\n".join(lines)


def get_task_prompt(task_dir: Path) -> str:
    prompt_file = task_dir / "prompt.txt"
    if prompt_file.exists():
        return prompt_file.read_text().strip()
    name = task_dir.name.replace("_", " ")
    return (
        f"Write a complete, runnable program that implements {name}. "
        f"Include a main entry point that demonstrates the solution with "
        f"example input and prints the result."
    )


def get_expected_output(task_dir: Path) -> str:
    f = task_dir / "expected_output.txt"
    return f.read_text().strip() if f.exists() else ""


# ---------------------------------------------------------------------------
# LLM interaction (raw HTTP to llama-server)
# ---------------------------------------------------------------------------


def call_llm(
    base_url: str,
    system_msg: str,
    user_msg: str,
    grammar: str | None = None,
    timeout: int = 120,
) -> tuple[str, float]:
    """Send a chat completion request. Returns (response_text, elapsed_ms)."""
    url = f"{base_url}/v1/chat/completions"
    body: dict = {
        "messages": [
            {"role": "system", "content": system_msg},
            {"role": "user", "content": user_msg},
        ],
        "temperature": 0.2,
        "max_tokens": 2048,
        "stream": False,
    }
    if grammar:
        body["grammar"] = grammar

    t0 = time.monotonic()
    resp = requests.post(url, json=body, timeout=timeout)
    elapsed = (time.monotonic() - t0) * 1000
    resp.raise_for_status()

    data = resp.json()
    text = data["choices"][0]["message"]["content"].strip()
    return text, elapsed


# ---------------------------------------------------------------------------
# Prompt construction
# ---------------------------------------------------------------------------


def build_system_prompt(language: str, context_dir: Path | None) -> str:
    lang_name = LANG_DISPLAY[language]
    prompt = (
        f"You are a code generator. Output ONLY valid {lang_name} code. "
        f"No explanations, no markdown."
    )
    if context_dir:
        ctx_file = context_dir / LANG_CONTEXT_FILES[language]
        if ctx_file.exists():
            prompt += f"\n\nLanguage reference:\n{ctx_file.read_text()}"
    return prompt


def build_fewshot_suffix(language: str, tasks_dir: Path) -> str:
    """Build few-shot examples from the factorial task directory."""
    example_dir = tasks_dir / FEWSHOT_EXAMPLE_TASK
    if not example_dir.is_dir():
        return ""

    ext = LANG_EXTENSIONS[language]
    examples: list[str] = []

    # Collect up to 3 example files from different tasks that have this language
    example_tasks = ["factorial", "fibonacci", "fizzbuzz"]
    for task_name in example_tasks:
        task_dir = tasks_dir / task_name
        if not task_dir.is_dir():
            continue
        files = list(task_dir.glob(f"*{ext}"))
        if not files:
            continue
        code = files[0].read_text().strip()
        prompt = get_task_prompt(task_dir)
        expected = get_expected_output(task_dir)
        example = f"### Task: {prompt}\n```\n{code}\n```"
        if expected:
            example += f"\nOutput: {expected}"
        examples.append(example)
        if len(examples) >= 3:
            break

    if not examples:
        return ""

    return "\n\n## Examples\n\n" + "\n\n".join(examples)


# ---------------------------------------------------------------------------
# Validation
# ---------------------------------------------------------------------------


def find_lang_dir() -> Path:
    """Locate the lang/ workspace root (for Synoema validation)."""
    # Walk up from this script: benchmarks/scripts/ -> benchmarks/ -> project root
    here = Path(__file__).resolve().parent
    candidate = here.parent.parent / "lang"
    if (candidate / "Cargo.toml").exists():
        return candidate
    # Fallback: try relative to cwd
    candidate = Path.cwd() / "lang"
    if (candidate / "Cargo.toml").exists():
        return candidate
    return Path("lang")  # best-effort


def validate_syntax(code: str, language: str) -> tuple[bool, str]:
    """Check if code parses. Returns (ok, error_msg)."""
    ext = LANG_EXTENSIONS[language]
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=ext, delete=False
        ) as f:
            f.write(code)
            path = f.name

        if language == "synoema":
            lang_dir = find_lang_dir()
            result = subprocess.run(
                [
                    "cargo", "run", "--quiet",
                    "--manifest-path", str(lang_dir / "Cargo.toml"),
                    "-p", "synoema-repl", "--", "run", path,
                ],
                capture_output=True, text=True, timeout=30,
            )
            if result.returncode != 0:
                return False, result.stderr.strip()
            return True, ""

        elif language == "python":
            result = subprocess.run(
                [
                    "python3", "-c",
                    f"compile(open('{path}').read(), '{path}', 'exec')",
                ],
                capture_output=True, text=True, timeout=10,
            )
            if result.returncode != 0:
                return False, result.stderr.strip()
            return True, ""

        elif language == "haskell":
            # runghc combines parse + typecheck + run
            result = subprocess.run(
                ["runghc", path],
                capture_output=True, text=True, timeout=30,
            )
            if result.returncode != 0:
                return False, result.stderr.strip()
            return True, ""

    except subprocess.TimeoutExpired:
        return False, "timeout"
    except FileNotFoundError as e:
        return False, f"command not found: {e}"
    finally:
        Path(path).unlink(missing_ok=True)

    return False, "unknown language"


def validate_correctness(code: str, language: str, expected: str) -> tuple[bool, str]:
    """Run code and compare stdout to expected output. Returns (ok, actual_output)."""
    if not expected:
        return False, ""

    ext = LANG_EXTENSIONS[language]
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=ext, delete=False
        ) as f:
            f.write(code)
            path = f.name

        if language == "synoema":
            lang_dir = find_lang_dir()
            result = subprocess.run(
                [
                    "cargo", "run", "--quiet",
                    "--manifest-path", str(lang_dir / "Cargo.toml"),
                    "-p", "synoema-repl", "--", "jit", path,
                ],
                capture_output=True, text=True, timeout=30,
            )
        elif language == "python":
            result = subprocess.run(
                ["python3", path],
                capture_output=True, text=True, timeout=15,
            )
        elif language == "haskell":
            result = subprocess.run(
                ["runghc", path],
                capture_output=True, text=True, timeout=30,
            )
        else:
            return False, ""

        actual = result.stdout.strip()
        return actual == expected, actual

    except subprocess.TimeoutExpired:
        return False, "timeout"
    except FileNotFoundError as e:
        return False, f"command not found: {e}"
    finally:
        Path(path).unlink(missing_ok=True)


# ---------------------------------------------------------------------------
# Single generation
# ---------------------------------------------------------------------------


def run_one(
    task_dir: Path,
    language: str,
    mode: str,
    base_url: str,
    context_dir: Path | None,
    grammar_text: str | None,
    tasks_dir: Path,
    model_name: str,
) -> dict:
    """Run a single generation + validation. Returns result dict."""
    task_name = task_dir.name
    system_msg = build_system_prompt(language, context_dir)

    # Few-shot and constrained: append examples
    if mode in ("few-shot", "constrained"):
        system_msg += build_fewshot_suffix(language, tasks_dir)

    user_msg = f"{get_task_prompt(task_dir)}"
    tokens_in = count_tokens(system_msg + user_msg)

    # Grammar: only for synoema in constrained mode
    grammar = None
    if mode == "constrained" and language == "synoema" and grammar_text:
        grammar = grammar_text

    try:
        raw_code, elapsed_ms = call_llm(
            base_url, system_msg, user_msg, grammar=grammar,
        )
    except requests.exceptions.ConnectionError:
        return _error_result(
            task_name, language, mode, model_name, tokens_in,
            "connection refused — is llama-server running?",
        )
    except requests.exceptions.Timeout:
        return _error_result(
            task_name, language, mode, model_name, tokens_in,
            "request timeout",
        )
    except requests.exceptions.HTTPError as e:
        return _error_result(
            task_name, language, mode, model_name, tokens_in,
            f"HTTP {e.response.status_code}: {e.response.text[:200]}",
        )
    except Exception as e:
        return _error_result(
            task_name, language, mode, model_name, tokens_in, str(e),
        )

    code = strip_fences(raw_code)
    tokens_out = count_tokens(code)

    syntax_ok, syntax_err = validate_syntax(code, language)

    expected = get_expected_output(task_dir)
    if syntax_ok and expected:
        correct, actual = validate_correctness(code, language, expected)
    else:
        correct = False
        actual = ""

    # For haskell, successful runghc means it also type-checks
    type_ok = syntax_ok if language == "haskell" else syntax_ok

    error_msg = ""
    if not syntax_ok:
        error_msg = syntax_err
    elif not correct and expected:
        error_msg = f"wrong output: got {actual!r}, expected {expected!r}"

    return {
        "task": task_name,
        "language": language,
        "mode": mode,
        "model": model_name,
        "syntax_ok": syntax_ok,
        "type_ok": type_ok,
        "correct": correct,
        "tokens_out": tokens_out,
        "time_ms": round(elapsed_ms, 1),
        "code": code,
        "error": error_msg,
    }


def _error_result(
    task: str, language: str, mode: str, model: str, tokens_in: int, error: str,
) -> dict:
    return {
        "task": task,
        "language": language,
        "mode": mode,
        "model": model,
        "syntax_ok": False,
        "type_ok": False,
        "correct": False,
        "tokens_out": 0,
        "time_ms": 0,
        "code": "",
        "error": error,
    }


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------


def discover_tasks(tasks_dir: Path) -> list[Path]:
    """Find all task directories (must contain prompt.txt)."""
    tasks = sorted(
        d for d in tasks_dir.iterdir()
        if d.is_dir() and (d / "prompt.txt").exists()
    )
    return tasks


def detect_model(base_url: str) -> str:
    """Try to detect the loaded model name from llama-server."""
    try:
        resp = requests.get(f"{base_url}/v1/models", timeout=5)
        if resp.ok:
            data = resp.json()
            models = data.get("data", [])
            if models:
                return models[0].get("id", "unknown")
    except Exception:
        pass
    return "unknown"


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Phase D: small-model-proof benchmark (llama.cpp + GBNF)"
    )
    parser.add_argument(
        "--base-url", default="http://localhost:8090",
        help="llama-server base URL (default: http://localhost:8090)",
    )
    parser.add_argument(
        "--tasks-dir", type=Path, required=True,
        help="Directory containing task subdirectories",
    )
    parser.add_argument(
        "--languages", nargs="+", default=LANGUAGES,
        choices=LANGUAGES, help="Languages to benchmark (default: all)",
    )
    parser.add_argument(
        "--modes", nargs="+", default=MODES,
        choices=MODES, help="Generation modes (default: all)",
    )
    parser.add_argument(
        "--repeats", type=int, default=5,
        help="Repetitions per (task, language, mode) triple (default: 5)",
    )
    parser.add_argument(
        "--output-dir", type=Path, default=None,
        help="Write per-run JSON files here (default: stdout only)",
    )
    parser.add_argument(
        "--grammar", type=Path, default=None,
        help="Path to .gbnf grammar file for constrained decoding",
    )
    parser.add_argument(
        "--context-dir", type=Path, default=None,
        help="Dir with language reference files (synoema.md, python.md, haskell.md)",
    )
    args = parser.parse_args()

    # Resolve paths
    tasks_dir = args.tasks_dir.resolve()
    if not tasks_dir.is_dir():
        print(f"Error: tasks directory not found: {tasks_dir}", file=sys.stderr)
        sys.exit(1)

    tasks = discover_tasks(tasks_dir)
    if not tasks:
        print(f"Error: no tasks found in {tasks_dir}", file=sys.stderr)
        sys.exit(1)

    # Load grammar once
    grammar_text = None
    if args.grammar and args.grammar.exists():
        grammar_text = args.grammar.read_text()
        log(f"Loaded grammar: {args.grammar} ({len(grammar_text)} chars)")

    # Detect model
    model_name = detect_model(args.base_url)
    log(f"Model: {model_name}")
    log(f"Tasks: {len(tasks)}, Languages: {args.languages}, Modes: {args.modes}, Repeats: {args.repeats}")

    total = len(tasks) * len(args.languages) * len(args.modes) * args.repeats
    log(f"Total generations: {total}")

    # Prepare output dir
    if args.output_dir:
        args.output_dir.mkdir(parents=True, exist_ok=True)

    results: list[dict] = []
    done = 0

    for task_dir in tasks:
        for language in args.languages:
            for mode in args.modes:
                for repeat in range(args.repeats):
                    done += 1
                    tag = f"[{done}/{total}]"
                    log(f"{tag} {task_dir.name} / {language} / {mode} (rep {repeat + 1})")

                    result = run_one(
                        task_dir=task_dir,
                        language=language,
                        mode=mode,
                        base_url=args.base_url,
                        context_dir=args.context_dir,
                        grammar_text=grammar_text,
                        tasks_dir=tasks_dir,
                        model_name=model_name,
                    )

                    status = "OK" if result["correct"] else (
                        "SYNTAX" if not result["syntax_ok"] else "WRONG"
                    )
                    log(f"  -> {status} ({result['time_ms']:.0f}ms, {result['tokens_out']} tok)")

                    # Output JSON line to stdout
                    print(json.dumps(result))
                    sys.stdout.flush()
                    results.append(result)

    # Write collected results to output dir
    if args.output_dir:
        out_file = args.output_dir / "results.jsonl"
        with open(out_file, "w") as f:
            for r in results:
                f.write(json.dumps(r) + "\n")
        log(f"Results written to {out_file}")

        # Write summary
        _write_summary(results, args.output_dir / "summary.json")

    log("Done.")


def _write_summary(results: list[dict], path: Path) -> None:
    """Compute and write aggregate statistics."""
    from collections import defaultdict

    groups: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for r in results:
        key = (r["language"], r["mode"])
        groups[key].append(r)

    summary = {}
    for (lang, mode), runs in sorted(groups.items()):
        n = len(runs)
        syntax_rate = sum(1 for r in runs if r["syntax_ok"]) / n if n else 0
        correct_rate = sum(1 for r in runs if r["correct"]) / n if n else 0
        avg_tokens = sum(r["tokens_out"] for r in runs) / n if n else 0
        avg_time = sum(r["time_ms"] for r in runs) / n if n else 0

        summary[f"{lang}/{mode}"] = {
            "runs": n,
            "syntax_rate": round(syntax_rate, 3),
            "correct_rate": round(correct_rate, 3),
            "avg_tokens_out": round(avg_tokens, 1),
            "avg_time_ms": round(avg_time, 1),
        }

    with open(path, "w") as f:
        json.dump(summary, f, indent=2)

    log(f"Summary written to {path}")


if __name__ == "__main__":
    main()
