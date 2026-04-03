#!/usr/bin/env python3
"""
Synoema BPE Alignment Verification

Verifies that all Synoema operators encode to exactly 1 BPE token
across major LLM tokenizers. This is a regression test — if any
operator breaks alignment, the language syntax must be reconsidered.

Usage:
    python3 verify_bpe.py
"""

import tiktoken
import json
import sys

# ── Synoema Operators ──────────────────────────────────────────

OPERATORS = {
    # Two-char operators
    "--":  "comment",
    "---": "doc_comment",
    "->":  "arrow",
    "<-":  "bind",
    "|>":  "pipe",
    "++":  "concat",
    ">>":  "compose",
    "==":  "eq",
    "!=":  "neq",
    "<=":  "lte",
    ">=":  "gte",
    "&&":  "and",
    "||":  "or",
    "..":  "range",
    # Single-char operators
    "?":   "cond",
    ":":   "type/cons",
    ".":   "field",
    "=":   "assign",
    "@":   "directive",
    "|":   "alt",
    "\\":  "lambda",
    "_":   "wildcard",
    ",":   "comma",
    "+":   "add",
    "-":   "sub",
    "*":   "mul",
    "/":   "div",
    "%":   "mod",
    "<":   "lt",
    ">":   "gt",
    # Delimiters
    "(":   "lparen",
    ")":   "rparen",
    "[":   "lbracket",
    "]":   "rbracket",
}

KEYWORDS = {
    "mod":      "module",
    "use":      "import",
    "trait":    "typeclass",
    "impl":     "instance",
    "true":     "bool_true",
    "false":    "bool_false",
    "lazy":     "lazy_eval",
    "derive":   "derive_clause",
}

# ── Tokenizer configs ────────────────────────────────────────

TOKENIZERS = {
    "cl100k_base (GPT-4)": "cl100k_base",
    "o200k_base (GPT-4o)": "o200k_base",
}

def verify_alignment():
    """Check every Synoema operator against each tokenizer."""
    results = {}
    all_pass = True

    for tok_name, tok_id in TOKENIZERS.items():
        enc = tiktoken.get_encoding(tok_id)
        results[tok_name] = {}

        print(f"\n{'='*60}")
        print(f"  Tokenizer: {tok_name}")
        print(f"{'='*60}")
        print(f"  {'Symbol':<8} {'Name':<12} {'Tokens':<8} {'IDs':<20} {'Status'}")
        print(f"  {'-'*58}")

        # Check operators
        for symbol, name in OPERATORS.items():
            tokens = enc.encode(symbol)
            count = len(tokens)
            status = "✓" if count == 1 else f"✗ ({count} tokens!)"
            if count != 1:
                all_pass = False

            # Display
            sym_display = repr(symbol) if symbol == "\\" else symbol
            print(f"  {sym_display:<8} {name:<12} {count:<8} {str(tokens):<20} {status}")

            results[tok_name][symbol] = {
                "name": name,
                "token_count": count,
                "token_ids": tokens,
                "aligned": count == 1,
            }

        print()
        print(f"  Keywords:")
        print(f"  {'-'*58}")

        for word, name in KEYWORDS.items():
            tokens = enc.encode(word)
            count = len(tokens)
            status = "✓" if count == 1 else f"✗ ({count} tokens!)"
            if count != 1:
                # Keywords > 1 token is acceptable but worth noting
                pass
            print(f"  {word:<8} {name:<12} {count:<8} {str(tokens):<20} {status}")

    return results, all_pass


def context_test():
    """Test operators in realistic code context (not isolated)."""
    enc = tiktoken.get_encoding("cl100k_base")

    test_cases = [
        # (description, code_snippet)
        ("arrow in lambda",     "\\x -> x + 1"),
        ("bind in effect",      "x <- readFile path"),
        ("pipe chain",          "xs |> filter even |> sum"),
        ("conditional",         "? x > 0 -> x : 0"),
        ("list comprehension",  "[x | x <- xs , x > 0]"),
        ("concat",              "xs ++ ys ++ zs"),
        ("compose",             "f >> g >> h"),
        ("pattern match",       "(x:xs)"),
        ("type sig",            "add : Int Int -> Int"),
        ("comparison chain",    "x >= 0 && x <= 100"),
    ]

    print(f"\n{'='*60}")
    print(f"  Context Test (cl100k_base)")
    print(f"{'='*60}")
    print(f"  {'Snippet':<35} {'Tokens':<8}")
    print(f"  {'-'*43}")

    for desc, code in test_cases:
        tokens = enc.encode(code)
        print(f"  {code:<35} {len(tokens):<8}")

    return True


def comparison_test():
    """Compare token counts of equivalent constructs: Synoema vs Python."""
    enc = tiktoken.get_encoding("cl100k_base")

    comparisons = [
        ("if/else",
         "? x > 0 -> x : 0",           # Synoema
         "x if x > 0 else 0"),          # Python

        ("function def",
         "fac n = n * fac (n - 1)",     # Synoema
         "def fac(n): return n * fac(n - 1)"),  # Python

        ("lambda",
         "\\x -> x * 2",                # Synoema
         "lambda x: x * 2"),            # Python

        ("list (no commas)",
         "[1 2 3 4 5]",                 # Synoema
         "[1, 2, 3, 4, 5]"),            # Python

        ("list comprehension",
         "[x | x <- xs , x > 0]",       # Synoema
         "[x for x in xs if x > 0]"),   # Python

        ("pipe vs nested",
         "xs |> filter even |> sum",    # Synoema
         "sum(filter(even, xs))"),       # Python

        ("pattern match",
         "fac 0 = 1",                   # Synoema
         "if n == 0: return 1"),         # Python

        ("concat",
         "xs ++ ys",                    # Synoema
         "xs + ys"),                    # Python

        ("comment",
         "-- this is a comment",        # Synoema
         "# this is a comment"),        # Python
    ]

    print(f"\n{'='*70}")
    print(f"  Synoema vs Python — Construct-Level Token Comparison (cl100k_base)")
    print(f"{'='*70}")
    print(f"  {'Construct':<20} {'Synoema':>8} {'Python':>8} {'Saving':>8}")
    print(f"  {'-'*48}")

    total_synoema = 0
    total_python = 0

    for name, sno_code, python_code in comparisons:
        a_tok = len(enc.encode(sno_code))
        p_tok = len(enc.encode(python_code))
        saving = ((p_tok - a_tok) / p_tok * 100) if p_tok > 0 else 0
        total_synoema += a_tok
        total_python += p_tok
        sign = "+" if saving >= 0 else ""
        print(f"  {name:<20} {a_tok:>8} {p_tok:>8} {sign}{saving:>6.1f}%")

    overall = ((total_python - total_sno) / total_python * 100)
    print(f"  {'-'*48}")
    print(f"  {'TOTAL':<20} {total_sno:>8} {total_python:>8} +{overall:>6.1f}%")

    return overall


if __name__ == "__main__":
    print("Synoema BPE Alignment Verification")
    print("=" * 60)

    # Test 1: Operator alignment
    results, all_pass = verify_alignment()

    # Test 2: Context test
    context_test()

    # Test 3: Construct comparison
    saving = comparison_test()

    # Summary
    print(f"\n{'='*60}")
    print(f"  SUMMARY")
    print(f"{'='*60}")

    op_count = len(OPERATORS)
    aligned = sum(1 for tok_results in results.values()
                  for v in tok_results.values() if v["aligned"])
    total_checks = op_count * len(TOKENIZERS)

    print(f"  Operators checked:    {op_count}")
    print(f"  Tokenizers tested:    {len(TOKENIZERS)}")
    print(f"  Alignment checks:     {aligned}/{total_checks} passed")
    print(f"  Construct savings:    {saving:.1f}% vs Python")
    print()

    if all_pass:
        print("  ✓ ALL OPERATORS BPE-ALIGNED")
    else:
        print("  ✗ SOME OPERATORS NOT ALIGNED — review syntax!")
        sys.exit(1)

    if saving >= 35:
        print(f"  ✓ TOKEN SAVINGS TARGET MET (≥35%, actual: {saving:.1f}%)")
    else:
        print(f"  ⚠ TOKEN SAVINGS BELOW TARGET (target: 35%, actual: {saving:.1f}%)")

    # Save results as JSON
    output = {
        "operators": {sym: {
            tok_name: results[tok_name][sym]
            for tok_name in results
        } for sym in OPERATORS},
        "construct_saving_pct": round(saving, 1),
        "all_aligned": all_pass,
    }

    json_path = "bpe_results.json"
    with open(json_path, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\n  Results saved to {json_path}")
