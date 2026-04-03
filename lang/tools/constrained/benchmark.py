#!/usr/bin/env python3
# SPDX-License-Identifier: BUSL-1.1
# Copyright (c) Andrey Bubnov
"""
Synoema Constrained Decoding Benchmark

Compares constrained vs unconstrained generation:
- Latency overhead from grammar
- Token count differences
- Correctness rates (syntax, type, runtime)

Usage:
    python3 benchmark.py --server http://localhost:30000 --output-dir bench_results/ --report
    python3 benchmark.py --server http://localhost:30000 --output-dir bench_results/ --count 20
"""

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

try:
    import openai
except ImportError:
    print("Error: pip install openai", file=sys.stderr)
    sys.exit(1)

SCRIPT_DIR = Path(__file__).parent
GRAMMAR_FILE = SCRIPT_DIR / "synoema.gbnf"
PROMPTS_FILE = SCRIPT_DIR / "prompts" / "prompts.json"

SYSTEM_PROMPT = (
    "You write code in the Synoema programming language. "
    "Output ONLY valid Synoema code. No comments, no explanations, no markdown."
)


def load_prompts(count: int) -> list[dict]:
    with open(PROMPTS_FILE) as f:
        tasks = json.load(f)
    prompts = []
    for task in tasks:
        for i, text in enumerate(task["prompts"]):
            prompts.append({"task": task["task"], "index": i, "prompt": text})
            if len(prompts) >= count:
                return prompts
    return prompts


def generate_one(client, prompt: str, grammar: str | None, model: str, max_tokens: int, temp: float):
    extra = {"ebnf": grammar} if grammar else {}
    t0 = time.monotonic()
    resp = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": prompt},
        ],
        max_tokens=max_tokens,
        temperature=temp,
        extra_body=extra if extra else openai.NOT_GIVEN,
    )
    latency = (time.monotonic() - t0) * 1000
    code = (resp.choices[0].message.content or "").strip()
    tokens = resp.usage.completion_tokens if resp.usage else 0
    return code, latency, tokens


def validate_sno(code: str, project_root: Path, timeout: int = 10) -> dict:
    """Quick validation: parse + typecheck + run."""
    import tempfile
    with tempfile.NamedTemporaryFile(suffix=".sno", mode="w", delete=False) as f:
        f.write(code)
        tmp = f.name

    try:
        proc = subprocess.run(
            ["cargo", "run", "-q", "-p", "synoema-repl", "--", "--errors", "json", "run", tmp],
            capture_output=True, text=True,
            cwd=str(project_root),
            timeout=timeout,
        )
        if proc.returncode == 0:
            return {"syntax": True, "types": True, "runs": True}

        stderr = proc.stderr.lower()
        if "parse" in stderr or "unexpected" in stderr or "syntax" in stderr:
            return {"syntax": False, "types": False, "runs": False}
        elif "type" in stderr or "mismatch" in stderr or "unbound" in stderr:
            return {"syntax": True, "types": False, "runs": False}
        else:
            return {"syntax": True, "types": True, "runs": False}
    except Exception:
        return {"syntax": False, "types": False, "runs": False}
    finally:
        Path(tmp).unlink(missing_ok=True)


def run_benchmark(client, prompts, grammar, model, max_tokens, temp, label, project_root):
    results = []
    for i, p in enumerate(prompts):
        try:
            code, latency, tokens = generate_one(client, p["prompt"], grammar, model, max_tokens, temp)
            valid = validate_sno(code, project_root)
            results.append({
                "task": p["task"],
                "latency_ms": latency,
                "tokens": tokens,
                "size": len(code),
                **valid,
            })
            icon = "✓" if valid["runs"] else ("⚠" if valid["syntax"] else "✗")
            print(f"  {icon} [{i+1:3d}/{len(prompts)}] {p['task']:20s} {latency:6.0f}ms {tokens:3d}tok")
        except Exception as e:
            results.append({"task": p["task"], "latency_ms": 0, "tokens": 0, "size": 0,
                            "syntax": False, "types": False, "runs": False, "error": str(e)})
            print(f"  ✗ [{i+1:3d}/{len(prompts)}] {p['task']:20s} ERROR: {e}")
    return results


def summarize(results: list, label: str) -> dict:
    n = len(results)
    ok = [r for r in results if "error" not in r]
    return {
        "label": label,
        "total": n,
        "syntax_correct": sum(1 for r in ok if r["syntax"]),
        "type_correct": sum(1 for r in ok if r["types"]),
        "run_correct": sum(1 for r in ok if r["runs"]),
        "syntax_rate": sum(1 for r in ok if r["syntax"]) / n if n else 0,
        "type_rate": sum(1 for r in ok if r["types"]) / n if n else 0,
        "run_rate": sum(1 for r in ok if r["runs"]) / n if n else 0,
        "avg_latency_ms": sum(r["latency_ms"] for r in ok) / len(ok) if ok else 0,
        "avg_tokens": sum(r["tokens"] for r in ok) / len(ok) if ok else 0,
        "errors": sum(1 for r in results if "error" in r),
    }


def print_comparison(constrained: dict, unconstrained: dict):
    print()
    print("=" * 70)
    print("  Benchmark Comparison: Constrained vs Unconstrained")
    print("=" * 70)
    print(f"  {'Metric':<25s} {'Constrained':>15s} {'Unconstrained':>15s} {'Delta':>10s}")
    print(f"  {'─' * 65}")

    def row(label, c_val, u_val, fmt=".1f", pct=False):
        delta = c_val - u_val
        d_str = f"{delta:+{fmt}}" + ("%" if pct else "")
        c_str = f"{c_val:{fmt}}" + ("%" if pct else "")
        u_str = f"{u_val:{fmt}}" + ("%" if pct else "")
        print(f"  {label:<25s} {c_str:>15s} {u_str:>15s} {d_str:>10s}")

    row("Syntax rate", constrained["syntax_rate"] * 100, unconstrained["syntax_rate"] * 100, ".1f", True)
    row("Type rate", constrained["type_rate"] * 100, unconstrained["type_rate"] * 100, ".1f", True)
    row("Run rate", constrained["run_rate"] * 100, unconstrained["run_rate"] * 100, ".1f", True)
    row("Avg latency (ms)", constrained["avg_latency_ms"], unconstrained["avg_latency_ms"], ".0f")
    row("Avg tokens", constrained["avg_tokens"], unconstrained["avg_tokens"], ".1f")

    # Overhead calculation
    if unconstrained["avg_latency_ms"] > 0:
        overhead = (constrained["avg_latency_ms"] - unconstrained["avg_latency_ms"]) / unconstrained["avg_latency_ms"] * 100
        print(f"\n  Grammar overhead: {overhead:+.1f}%")

    print("=" * 70)


def main():
    parser = argparse.ArgumentParser(description="Synoema Constrained Decoding Benchmark")
    parser.add_argument("--server", required=True, help="SGLang server URL")
    parser.add_argument("--output-dir", required=True, help="Output directory for results")
    parser.add_argument("--count", type=int, default=100, help="Programs per mode (default: 100)")
    parser.add_argument("--model", default="default", help="Model name")
    parser.add_argument("--max-tokens", type=int, default=256, help="Max tokens")
    parser.add_argument("--temperature", type=float, default=0.2, help="Temperature")
    parser.add_argument("--report", action="store_true", help="Print comparison report")
    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    project_root = SCRIPT_DIR.parent.parent

    base_url = args.server.rstrip("/")
    if not base_url.endswith("/v1"):
        base_url += "/v1"
    client = openai.OpenAI(base_url=base_url, api_key="none")

    grammar = GRAMMAR_FILE.read_text()
    prompts = load_prompts(args.count)

    # Run constrained
    print("\n  ── Constrained (with GBNF grammar) ──")
    c_results = run_benchmark(client, prompts, grammar, args.model, args.max_tokens, args.temperature,
                              "constrained", project_root)

    # Run unconstrained
    print("\n  ── Unconstrained (no grammar) ──")
    u_results = run_benchmark(client, prompts, None, args.model, args.max_tokens, args.temperature,
                              "unconstrained", project_root)

    c_summary = summarize(c_results, "constrained")
    u_summary = summarize(u_results, "unconstrained")

    # Write results
    report = {"constrained": c_summary, "unconstrained": u_summary, "programs_per_mode": len(prompts)}
    report_path = output_dir / "benchmark.json"
    report_path.write_text(json.dumps(report, indent=2))
    print(f"\n  Results: {report_path}")

    if args.report:
        print_comparison(c_summary, u_summary)


if __name__ == "__main__":
    main()
