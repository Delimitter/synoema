#!/usr/bin/env python3
"""Phase D: Analyze model size benchmark results and generate comparison tables/charts."""

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path

def load_results(results_dir: Path):
    """Load all JSON result files from a directory."""
    results = []
    for f in sorted(results_dir.glob("*.jsonl")):
        with open(f) as fh:
            for line in fh:
                line = line.strip()
                if line:
                    results.append(json.loads(line))
    # Also try single results.json
    single = results_dir / "results.json"
    if single.exists():
        with open(single) as fh:
            data = json.load(fh)
            if isinstance(data, list):
                results.extend(data)
    return results


def aggregate(results):
    """Aggregate results by (model, language, mode)."""
    groups = defaultdict(lambda: {"syntax_ok": 0, "type_ok": 0, "correct": 0, "total": 0, "tokens_out": []})
    for r in results:
        key = (r.get("model", "unknown"), r.get("language", "unknown"), r.get("mode", "unknown"))
        g = groups[key]
        g["total"] += 1
        if r.get("syntax_ok"):
            g["syntax_ok"] += 1
        if r.get("type_ok"):
            g["type_ok"] += 1
        if r.get("correct"):
            g["correct"] += 1
        if r.get("tokens_out"):
            g["tokens_out"].append(r["tokens_out"])
    return groups


def print_summary_table(groups):
    """Print correctness table: model x (language+mode)."""
    # Extract unique models and (language, mode) pairs
    models = sorted(set(k[0] for k in groups.keys()))
    lang_modes = sorted(set((k[1], k[2]) for k in groups.keys()))

    # Header
    header = f"{'Model':>30}"
    for lang, mode in lang_modes:
        label = f"{lang}"
        if mode != "zero-shot":
            label += f"+{mode}"
        header += f"  {label:>14}"
    print(header)
    print("-" * len(header))

    # Rows
    for model in models:
        row = f"{model:>30}"
        for lang, mode in lang_modes:
            key = (model, lang, mode)
            if key in groups:
                g = groups[key]
                rate = g["correct"] / g["total"] * 100 if g["total"] > 0 else 0
                row += f"  {rate:>13.1f}%"
            else:
                row += f"  {'—':>14}"
        print(row)


def print_syntax_table(groups):
    """Print syntax rate table."""
    models = sorted(set(k[0] for k in groups.keys()))
    lang_modes = sorted(set((k[1], k[2]) for k in groups.keys()))

    print("\n--- Syntax Rate ---")
    header = f"{'Model':>30}"
    for lang, mode in lang_modes:
        label = f"{lang}"
        if mode != "zero-shot":
            label += f"+{mode}"
        header += f"  {label:>14}"
    print(header)
    print("-" * len(header))

    for model in models:
        row = f"{model:>30}"
        for lang, mode in lang_modes:
            key = (model, lang, mode)
            if key in groups:
                g = groups[key]
                rate = g["syntax_ok"] / g["total"] * 100 if g["total"] > 0 else 0
                row += f"  {rate:>13.1f}%"
            else:
                row += f"  {'—':>14}"
        print(row)


def compute_min_model(groups, threshold=70.0):
    """Find minimum model achieving threshold% correctness per language+mode."""
    lang_modes = sorted(set((k[1], k[2]) for k in groups.keys()))
    models_sorted = sorted(set(k[0] for k in groups.keys()))

    print(f"\n--- Minimum Model for {threshold}% Correctness ---")
    for lang, mode in lang_modes:
        label = f"{lang}"
        if mode != "zero-shot":
            label += f"+{mode}"
        best = None
        for model in models_sorted:
            key = (model, lang, mode)
            if key in groups:
                g = groups[key]
                rate = g["correct"] / g["total"] * 100 if g["total"] > 0 else 0
                if rate >= threshold:
                    best = model
                    break
        if best:
            print(f"  {label:>20}: {best}")
        else:
            print(f"  {label:>20}: none (max < {threshold}%)")


def export_markdown(groups, output_path: Path):
    """Export results as markdown tables."""
    models = sorted(set(k[0] for k in groups.keys()))
    lang_modes = sorted(set((k[1], k[2]) for k in groups.keys()))

    lines = ["# Phase D: Model Size Reduction Results\n"]

    # Correctness table
    lines.append("## Correctness Rate (%)\n")
    header = "| Model |"
    sep = "|-------|"
    for lang, mode in lang_modes:
        label = f"{lang}"
        if mode != "zero-shot":
            label += f"+{mode}"
        header += f" {label} |"
        sep += "--------|"
    lines.append(header)
    lines.append(sep)

    for model in models:
        row = f"| {model} |"
        for lang, mode in lang_modes:
            key = (model, lang, mode)
            if key in groups:
                g = groups[key]
                rate = g["correct"] / g["total"] * 100 if g["total"] > 0 else 0
                row += f" {rate:.1f}% |"
            else:
                row += " — |"
        lines.append(row)

    lines.append("")
    output_path.write_text("\n".join(lines))
    print(f"\nMarkdown exported to {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Analyze Phase D benchmark results")
    parser.add_argument("results_dir", type=Path, help="Directory with result files")
    parser.add_argument("--threshold", type=float, default=70.0, help="Correctness threshold (default: 70%%)")
    parser.add_argument("--export-md", type=Path, help="Export markdown to file")
    args = parser.parse_args()

    if not args.results_dir.exists():
        print(f"Error: {args.results_dir} not found", file=sys.stderr)
        sys.exit(1)

    results = load_results(args.results_dir)
    if not results:
        print("No results found", file=sys.stderr)
        sys.exit(1)

    print(f"Loaded {len(results)} results\n")

    groups = aggregate(results)

    print("--- Correctness Rate ---")
    print_summary_table(groups)
    print_syntax_table(groups)
    compute_min_model(groups, args.threshold)

    if args.export_md:
        export_markdown(groups, args.export_md)


if __name__ == "__main__":
    main()
