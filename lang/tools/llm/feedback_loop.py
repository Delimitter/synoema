#!/usr/bin/env python3
# SPDX-License-Identifier: BUSL-1.1
# Copyright (c) 2025-present Andrey Bubnov
"""
Synoema LLM Feedback Loop — generate → check → enrich → retry.

Usage:
    python feedback_loop.py --prompt "Write a factorial function" [--model gpt-4o] [--retries 3]
    python feedback_loop.py --prompt-file prompt.txt --provider anthropic

Requires:
    pip install openai anthropic  (depending on provider)
"""

import argparse
import json
import subprocess
import sys
import tempfile
import os

# ── Config ──────────────────────────────────────────────

DEFAULT_RETRIES = 3
TEMPERATURE_SCHEDULE = [1.0, 0.5, 0.2]  # decay per retry
SYNOEMA_BIN = os.environ.get("SYNOEMA_BIN", "cargo")
SYNOEMA_ARGS = ["run", "-p", "synoema-repl", "--"]


def find_synoema_binary():
    """Locate the synoema binary or use cargo run."""
    return SYNOEMA_BIN, SYNOEMA_ARGS


# ── Synoema check ──────────────────────────────────────

def check_synoema(code: str) -> dict:
    """Run synoema --errors json on the given code.

    Returns:
        {"ok": True} on success
        {"ok": False, "error": <json-parsed-error>} on failure
    """
    with tempfile.NamedTemporaryFile(suffix=".sno", mode="w", delete=False) as f:
        f.write(code)
        f.flush()
        tmp_path = f.name

    try:
        binary, args = find_synoema_binary()
        result = subprocess.run(
            [binary] + args + ["--errors", "json", "run", tmp_path],
            capture_output=True, text=True, timeout=30,
        )
        if result.returncode == 0:
            return {"ok": True, "stdout": result.stdout.strip()}
        # Parse JSON error from stderr
        stderr = result.stderr.strip()
        try:
            error = json.loads(stderr)
        except json.JSONDecodeError:
            error = {"message": stderr, "code": "unknown"}
        return {"ok": False, "error": error}
    finally:
        os.unlink(tmp_path)


# ── Error formatting for LLM ──────────────────────────

def format_error_for_llm(code: str, error: dict) -> str:
    """Format a Synoema error as a prompt for the LLM to fix."""
    parts = [
        "The following Synoema program has an error:\n",
        f"```sno\n{code}\n```\n",
    ]

    # Error details
    err_code = error.get("code", "unknown")
    msg = error.get("message", "unknown error")
    span = error.get("span", {})
    line = span.get("line", "?")
    col = span.get("col", "?")

    parts.append(f"Error at line {line}, column {col}:")
    parts.append(f"  {err_code}: {msg}")

    if "llm_hint" in error:
        parts.append(f"  Hint: {error['llm_hint']}")
    if "fixability" in error:
        parts.append(f"  Fixability: {error['fixability']}")
    if "did_you_mean" in error:
        parts.append(f"  Did you mean: {error['did_you_mean']}")
    if "notes" in error:
        for note in error["notes"]:
            parts.append(f"  Note: {note}")

    parts.append("\nPlease fix the error and output ONLY the corrected Synoema program.")
    parts.append("Output the program inside ```sno ... ``` code fences.")

    return "\n".join(parts)


# ── LLM providers ─────────────────────────────────────

def call_openai(prompt: str, system: str, model: str, temperature: float) -> str:
    """Call OpenAI API."""
    from openai import OpenAI
    client = OpenAI()
    response = client.chat.completions.create(
        model=model,
        temperature=temperature,
        messages=[
            {"role": "system", "content": system},
            {"role": "user", "content": prompt},
        ],
    )
    return response.choices[0].message.content


def call_anthropic(prompt: str, system: str, model: str, temperature: float) -> str:
    """Call Anthropic API."""
    import anthropic
    client = anthropic.Anthropic()
    response = client.messages.create(
        model=model,
        max_tokens=4096,
        temperature=temperature,
        system=system,
        messages=[{"role": "user", "content": prompt}],
    )
    return response.content[0].text


PROVIDERS = {
    "openai": {"fn": call_openai, "default_model": "gpt-4o"},
    "anthropic": {"fn": call_anthropic, "default_model": "claude-sonnet-4-20250514"},
}


# ── Extract code from LLM response ────────────────────

def extract_code(response: str) -> str:
    """Extract code from markdown fences."""
    # Try ```sno ... ``` first
    for lang in ["sno", "synoema", ""]:
        marker = f"```{lang}"
        if marker in response:
            start = response.index(marker) + len(marker)
            end = response.index("```", start)
            return response[start:end].strip()
    # Fallback: return the whole response
    return response.strip()


# ── Main loop ─────────────────────────────────────────

SYSTEM_PROMPT = """You are a Synoema programming language expert.
Key syntax rules:
- No if/then/else: use ternary ? cond -> x : y
- Lists are space-separated: [1 2 3] not [1, 2, 3]
- No return keyword: last expression is the result
- Lambda: \\x -> body
- Pattern matching via multi-equation definitions
- Offside rule: indent body deeper than the definition name
- Operators: + - * / % ++ == /= < > <= >= && || : ?
"""


def run_loop(prompt: str, provider: str, model: str, max_retries: int, verbose: bool):
    """Run the generate → check → fix loop."""
    provider_info = PROVIDERS[provider]
    call_fn = provider_info["fn"]
    if model is None:
        model = provider_info["default_model"]

    current_prompt = prompt
    for attempt in range(max_retries + 1):
        temp = TEMPERATURE_SCHEDULE[min(attempt, len(TEMPERATURE_SCHEDULE) - 1)]

        if verbose:
            print(f"\n--- Attempt {attempt + 1}/{max_retries + 1} (temp={temp}) ---", file=sys.stderr)

        # Generate
        response = call_fn(current_prompt, SYSTEM_PROMPT, model, temp)
        code = extract_code(response)

        if verbose:
            print(f"Generated:\n{code}\n", file=sys.stderr)

        # Check
        result = check_synoema(code)
        if result["ok"]:
            print(code)
            if result.get("stdout"):
                print(f"\n--- Output ---\n{result['stdout']}", file=sys.stderr)
            if verbose:
                print(f"\nSuccess on attempt {attempt + 1}", file=sys.stderr)
            return True

        # Format error for retry
        error = result["error"]
        if verbose:
            print(f"Error: {json.dumps(error, indent=2)}", file=sys.stderr)

        current_prompt = format_error_for_llm(code, error)

    print(f"Failed after {max_retries + 1} attempts.", file=sys.stderr)
    print(code)  # output last attempt
    return False


# ── CLI ───────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Synoema LLM Feedback Loop")
    parser.add_argument("--prompt", type=str, help="The generation prompt")
    parser.add_argument("--prompt-file", type=str, help="Read prompt from file")
    parser.add_argument("--provider", choices=list(PROVIDERS.keys()), default="openai")
    parser.add_argument("--model", type=str, default=None)
    parser.add_argument("--retries", type=int, default=DEFAULT_RETRIES)
    parser.add_argument("--verbose", "-v", action="store_true")
    args = parser.parse_args()

    if args.prompt_file:
        with open(args.prompt_file) as f:
            prompt = f.read().strip()
    elif args.prompt:
        prompt = args.prompt
    else:
        parser.error("Either --prompt or --prompt-file is required")
        return

    full_prompt = (
        f"Write a Synoema program that does the following:\n{prompt}\n\n"
        f"Output ONLY the program inside ```sno ... ``` code fences."
    )

    ok = run_loop(full_prompt, args.provider, args.model, args.retries, args.verbose)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
