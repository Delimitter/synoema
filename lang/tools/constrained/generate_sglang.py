#!/usr/bin/env python3
# SPDX-License-Identifier: BUSL-1.1
# Copyright (c) Andrey Bubnov
"""
Synoema Constrained Generation via SGLang (OpenAI-compatible API)

Generates Synoema programs using SGLang with EBNF grammar constraint.
Supports both constrained and unconstrained modes for benchmarking.

Usage:
    python3 generate_sglang.py --server http://localhost:30000 --output-dir generated/
    python3 generate_sglang.py --server http://localhost:30000 --output-dir generated/ --count 20
    python3 generate_sglang.py --server http://localhost:30000 --output-dir unconstrained/ --no-grammar

Requirements:
    pip install openai
"""

import argparse
import json
import os
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
    """Load prompts from the prompt database."""
    with open(PROMPTS_FILE) as f:
        tasks = json.load(f)

    prompts = []
    for task in tasks:
        task_name = task["task"]
        for i, prompt_text in enumerate(task["prompts"]):
            prompts.append({
                "task": task_name,
                "index": i,
                "prompt": prompt_text,
                "expected_features": task.get("expected_features", []),
            })
            if len(prompts) >= count:
                return prompts
    return prompts


def generate_program(
    client: openai.OpenAI,
    prompt: str,
    grammar: str | None,
    model: str,
    max_tokens: int,
    temperature: float,
) -> tuple[str, float]:
    """Generate a single program. Returns (code, latency_ms)."""
    extra_body = {}
    if grammar:
        extra_body["ebnf"] = grammar

    t0 = time.monotonic()
    response = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": prompt},
        ],
        max_tokens=max_tokens,
        temperature=temperature,
        extra_body=extra_body if extra_body else openai.NOT_GIVEN,
    )
    latency_ms = (time.monotonic() - t0) * 1000

    code = response.choices[0].message.content or ""
    return code.strip(), latency_ms


def main():
    parser = argparse.ArgumentParser(description="Synoema SGLang Constrained Generation")
    parser.add_argument("--server", required=True, help="SGLang server URL (e.g., http://localhost:30000)")
    parser.add_argument("--output-dir", required=True, help="Output directory for .sno files")
    parser.add_argument("--count", type=int, default=100, help="Number of programs (default: 100)")
    parser.add_argument("--no-grammar", action="store_true", help="Disable grammar constraint")
    parser.add_argument("--model", default="default", help="Model name (default: 'default')")
    parser.add_argument("--max-tokens", type=int, default=256, help="Max tokens (default: 256)")
    parser.add_argument("--temperature", type=float, default=0.2, help="Temperature (default: 0.2)")
    parser.add_argument("--timing-output", help="Write per-program timing to JSON file")
    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load grammar
    grammar = None
    if not args.no_grammar:
        grammar = GRAMMAR_FILE.read_text()

    # Load prompts
    prompts = load_prompts(args.count)

    # Connect to SGLang
    base_url = args.server.rstrip("/")
    if not base_url.endswith("/v1"):
        base_url += "/v1"
    client = openai.OpenAI(base_url=base_url, api_key="none")

    print("=" * 55)
    print("  Synoema Constrained Generation (SGLang)")
    print("=" * 55)
    print(f"  Server:      {args.server}")
    print(f"  Grammar:     {'DISABLED' if args.no_grammar else 'ENABLED'}")
    print(f"  Prompts:     {len(prompts)}")
    print(f"  Output:      {output_dir}")
    print(f"  Temperature: {args.temperature}")
    print(f"  Max tokens:  {args.max_tokens}")
    print("─" * 55)

    timings = []
    for i, p in enumerate(prompts):
        task_name = f"{i+1:03d}_{p['task']}"
        output_file = output_dir / f"{task_name}.sno"

        try:
            code, latency_ms = generate_program(
                client, p["prompt"], grammar,
                args.model, args.max_tokens, args.temperature,
            )
            output_file.write_text(code + "\n")
            size = len(code)
            timings.append({"file": task_name, "latency_ms": latency_ms, "size": size})
            print(f"  [{i+1:3d}/{len(prompts):3d}] {task_name}.sno  ({size} bytes, {latency_ms:.0f}ms)")
        except Exception as e:
            print(f"  [{i+1:3d}/{len(prompts):3d}] {task_name}.sno  FAILED: {e}")
            output_file.write_text(f"-- generation failed: {e}\n")
            timings.append({"file": task_name, "latency_ms": 0, "size": 0, "error": str(e)})

    # Summary
    ok_timings = [t for t in timings if "error" not in t]
    total_ms = sum(t["latency_ms"] for t in ok_timings)
    avg_ms = total_ms / len(ok_timings) if ok_timings else 0

    print("─" * 55)
    print(f"  Generated: {len(ok_timings)}/{len(prompts)} programs")
    print(f"  Total time: {total_ms:.0f}ms")
    print(f"  Avg latency: {avg_ms:.0f}ms per program")

    if args.timing_output:
        Path(args.timing_output).write_text(json.dumps(timings, indent=2))
        print(f"  Timing: {args.timing_output}")

    print("=" * 55)
    print(f"  Validate: python3 validate_e2e.py --input-dir {output_dir} --report")
    print("=" * 55)


if __name__ == "__main__":
    main()
