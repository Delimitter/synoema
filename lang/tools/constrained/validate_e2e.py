#!/usr/bin/env python3
# SPDX-License-Identifier: BUSL-1.1
# Copyright (c) Andrey Bubnov
"""
Synoema E2E Constrained Decoding Validator

Validates generated .sno files by:
1. Parsing each file with synoema-repl (--errors json)
2. Typechecking (run mode catches type errors)
3. Collecting metrics: syntax rate, type rate, error distribution

Usage:
    python3 validate_e2e.py --input-dir generated/ --output results.json
    python3 validate_e2e.py --input-dir generated/ --report
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from dataclasses import dataclass, field, asdict
from typing import Optional

SCRIPT_DIR = Path(__file__).parent
PROJECT_ROOT = SCRIPT_DIR.parent.parent  # lang/


@dataclass
class ProgramResult:
    file: str
    syntax_ok: bool
    types_ok: bool
    runs_ok: bool
    parse_errors: list = field(default_factory=list)
    type_errors: list = field(default_factory=list)
    runtime_errors: list = field(default_factory=list)
    parse_time_ms: float = 0.0
    output: str = ""


@dataclass
class E2EReport:
    total_generated: int = 0
    syntax_correct: int = 0
    type_correct: int = 0
    run_correct: int = 0
    syntax_rate: float = 0.0
    type_rate: float = 0.0
    run_rate: float = 0.0
    avg_parse_time_ms: float = 0.0
    errors_by_code: dict = field(default_factory=dict)
    programs: list = field(default_factory=list)


def validate_program(sno_path: Path, timeout: int = 10) -> ProgramResult:
    """Validate a single .sno file through parse → typecheck → run."""
    result = ProgramResult(file=str(sno_path), syntax_ok=False, types_ok=False, runs_ok=False)
    source = sno_path.read_text()

    # Step 1: Parse check (--errors json eval "0" just to trigger parse)
    t0 = time.monotonic()
    try:
        proc = subprocess.run(
            ["cargo", "run", "-q", "-p", "synoema-repl", "--", "--errors", "json", "run", str(sno_path)],
            capture_output=True, text=True,
            cwd=str(PROJECT_ROOT),
            timeout=timeout,
        )
        result.parse_time_ms = (time.monotonic() - t0) * 1000

        if proc.returncode == 0:
            result.syntax_ok = True
            result.types_ok = True
            result.runs_ok = True
            result.output = proc.stdout.strip()
        else:
            stderr = proc.stderr.strip()
            # Try to parse JSON errors
            for line in stderr.split("\n"):
                line = line.strip()
                if not line:
                    continue
                try:
                    err = json.loads(line)
                    code = err.get("code", "unknown")
                    msg = err.get("message", stderr[:200])
                    if "parse" in code.lower() or "syntax" in code.lower() or "unexpected" in msg.lower():
                        result.parse_errors.append({"code": code, "message": msg})
                    elif "type" in code.lower() or "mismatch" in code.lower() or "unbound" in code.lower():
                        result.syntax_ok = True  # parsed but type error
                        result.type_errors.append({"code": code, "message": msg})
                    else:
                        result.syntax_ok = True  # parsed but runtime or other error
                        result.runtime_errors.append({"code": code, "message": msg})

                    result.errors_by_code = result.errors_by_code if hasattr(result, 'errors_by_code') else {}
                except json.JSONDecodeError:
                    # Non-JSON error output — classify by content
                    lower = stderr.lower()
                    if "parse" in lower or "unexpected" in lower or "syntax" in lower:
                        result.parse_errors.append({"code": "parse_error", "message": stderr[:300]})
                    elif "type" in lower or "mismatch" in lower or "unbound" in lower:
                        result.syntax_ok = True
                        result.type_errors.append({"code": "type_error", "message": stderr[:300]})
                    else:
                        result.syntax_ok = True
                        result.runtime_errors.append({"code": "runtime_error", "message": stderr[:300]})
                    break  # Don't re-parse non-JSON

    except subprocess.TimeoutExpired:
        result.runtime_errors.append({"code": "timeout", "message": f"Timed out after {timeout}s"})
    except Exception as e:
        result.runtime_errors.append({"code": "internal", "message": str(e)})

    return result


def validate_directory(input_dir: Path, timeout: int = 10) -> E2EReport:
    """Validate all .sno files in a directory."""
    report = E2EReport()
    sno_files = sorted(input_dir.glob("*.sno"))

    if not sno_files:
        print(f"  No .sno files found in {input_dir}", file=sys.stderr)
        return report

    print(f"  Validating {len(sno_files)} programs from {input_dir}")
    print(f"  {'─' * 55}")

    parse_times = []
    for sno in sno_files:
        result = validate_program(sno, timeout=timeout)
        report.total_generated += 1
        report.programs.append(asdict(result))

        if result.syntax_ok:
            report.syntax_correct += 1
        if result.types_ok:
            report.type_correct += 1
        if result.runs_ok:
            report.run_correct += 1

        parse_times.append(result.parse_time_ms)

        # Collect error codes
        for err_list in [result.parse_errors, result.type_errors, result.runtime_errors]:
            for err in err_list:
                code = err.get("code", "unknown")
                report.errors_by_code[code] = report.errors_by_code.get(code, 0) + 1

        status = "OK" if result.runs_ok else ("TYPE" if result.syntax_ok else "PARSE")
        icon = "✓" if result.runs_ok else ("⚠" if result.syntax_ok else "✗")
        print(f"  {icon} {sno.name:30s} {status}")

    # Compute rates
    n = report.total_generated
    if n > 0:
        report.syntax_rate = report.syntax_correct / n
        report.type_rate = report.type_correct / n
        report.run_rate = report.run_correct / n
        report.avg_parse_time_ms = sum(parse_times) / len(parse_times)

    return report


def print_report(report: E2EReport):
    """Print human-readable report."""
    print()
    print("=" * 55)
    print("  Synoema E2E Validation Report")
    print("=" * 55)
    print(f"  Total programs:     {report.total_generated}")
    print(f"  Syntax correct:     {report.syntax_correct}/{report.total_generated} ({report.syntax_rate:.1%})")
    print(f"  Type correct:       {report.type_correct}/{report.total_generated} ({report.type_rate:.1%})")
    print(f"  Fully correct:      {report.run_correct}/{report.total_generated} ({report.run_rate:.1%})")
    print(f"  Avg parse time:     {report.avg_parse_time_ms:.1f} ms")

    if report.errors_by_code:
        print(f"\n  Errors by code:")
        for code, count in sorted(report.errors_by_code.items(), key=lambda x: -x[1]):
            print(f"    {code:25s} {count}")

    print("=" * 55)


def main():
    parser = argparse.ArgumentParser(description="Synoema E2E Constrained Decoding Validator")
    parser.add_argument("--input-dir", required=True, help="Directory with generated .sno files")
    parser.add_argument("--output", help="Output JSON report path")
    parser.add_argument("--report", action="store_true", help="Print human-readable report")
    parser.add_argument("--timeout", type=int, default=10, help="Timeout per program (seconds)")
    args = parser.parse_args()

    input_dir = Path(args.input_dir)
    if not input_dir.is_dir():
        print(f"Error: {input_dir} is not a directory", file=sys.stderr)
        sys.exit(1)

    report = validate_directory(input_dir, timeout=args.timeout)

    if args.report or not args.output:
        print_report(report)

    if args.output:
        output_path = Path(args.output)
        # Remove per-program details for summary JSON (keep it compact)
        summary = asdict(report)
        del summary["programs"]
        with open(output_path, "w") as f:
            json.dump(summary, f, indent=2)
        print(f"\n  JSON report written to {output_path}")

        # Also write detailed report
        detail_path = output_path.with_suffix(".detailed.json")
        with open(detail_path, "w") as f:
            json.dump(asdict(report), f, indent=2)
        print(f"  Detailed report written to {detail_path}")


if __name__ == "__main__":
    main()
