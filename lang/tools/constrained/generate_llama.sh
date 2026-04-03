#!/usr/bin/env bash
# SPDX-License-Identifier: BUSL-1.1
# Copyright (c) Andrey Bubnov
#
# Synoema Constrained Generation via llama.cpp
#
# Usage:
#   ./generate_llama.sh --model model.gguf --output-dir generated/
#   ./generate_llama.sh --model model.gguf --output-dir generated/ --count 20
#   ./generate_llama.sh --model model.gguf --output-dir generated/ --no-grammar
#
# Requirements:
#   - llama.cpp built (llama-cli binary in PATH or LLAMA_CLI env var)
#   - A GGUF model file
#   - jq (for JSON prompt parsing)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GRAMMAR_FILE="${SCRIPT_DIR}/synoema.gbnf"
PROMPTS_FILE="${SCRIPT_DIR}/prompts/prompts.json"
LLAMA_CLI="${LLAMA_CLI:-llama-cli}"
MODEL=""
OUTPUT_DIR=""
COUNT=100
NO_GRAMMAR=false
TEMPERATURE=0.2
MAX_TOKENS=256
SEED=42

usage() {
    echo "Usage: $0 --model MODEL.gguf --output-dir DIR [options]"
    echo ""
    echo "Options:"
    echo "  --model PATH       Path to GGUF model file (required)"
    echo "  --output-dir DIR   Output directory for .sno files (required)"
    echo "  --count N          Number of programs to generate (default: 100)"
    echo "  --no-grammar       Generate without grammar constraint (for comparison)"
    echo "  --temperature T    Sampling temperature (default: 0.2)"
    echo "  --max-tokens N     Max tokens per generation (default: 256)"
    echo "  --seed N           Random seed (default: 42)"
    echo "  --llama-cli PATH   Path to llama-cli binary"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --model) MODEL="$2"; shift 2 ;;
        --output-dir) OUTPUT_DIR="$2"; shift 2 ;;
        --count) COUNT="$2"; shift 2 ;;
        --no-grammar) NO_GRAMMAR=true; shift ;;
        --temperature) TEMPERATURE="$2"; shift 2 ;;
        --max-tokens) MAX_TOKENS="$2"; shift 2 ;;
        --seed) SEED="$2"; shift 2 ;;
        --llama-cli) LLAMA_CLI="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

[[ -z "$MODEL" ]] && { echo "Error: --model required"; usage; }
[[ -z "$OUTPUT_DIR" ]] && { echo "Error: --output-dir required"; usage; }
[[ -f "$MODEL" ]] || { echo "Error: Model file not found: $MODEL"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "Error: jq is required"; exit 1; }

mkdir -p "$OUTPUT_DIR"

# Extract prompts from JSON
PROMPTS=$(jq -r '.[].prompts[]' "$PROMPTS_FILE")
TOTAL=$(echo "$PROMPTS" | wc -l | tr -d ' ')

echo "═══════════════════════════════════════════════════════"
echo "  Synoema Constrained Generation (llama.cpp)"
echo "═══════════════════════════════════════════════════════"
echo "  Model:       $MODEL"
echo "  Grammar:     $([ "$NO_GRAMMAR" = true ] && echo "DISABLED" || echo "$GRAMMAR_FILE")"
echo "  Prompts:     $TOTAL available, generating $COUNT"
echo "  Output:      $OUTPUT_DIR"
echo "  Temperature: $TEMPERATURE"
echo "  Max tokens:  $MAX_TOKENS"
echo "───────────────────────────────────────────────────────"

GENERATED=0
INDEX=0
TIMING_FILE=$(mktemp)

echo "$PROMPTS" | head -n "$COUNT" | while IFS= read -r prompt; do
    INDEX=$((INDEX + 1))
    TASK_NAME=$(printf "%03d" "$INDEX")
    OUTPUT_FILE="${OUTPUT_DIR}/${TASK_NAME}.sno"

    GRAMMAR_ARGS=""
    if [ "$NO_GRAMMAR" = false ]; then
        GRAMMAR_ARGS="--grammar-file ${GRAMMAR_FILE}"
    fi

    START_MS=$(python3 -c "import time; print(int(time.monotonic() * 1000))")

    # System prompt for Synoema code generation
    SYSTEM="You write code in the Synoema programming language. Output ONLY valid Synoema code, no comments, no explanations."

    "$LLAMA_CLI" \
        -m "$MODEL" \
        $GRAMMAR_ARGS \
        -p "System: ${SYSTEM}\nUser: ${prompt}\nAssistant:\n" \
        -n "$MAX_TOKENS" \
        --temp "$TEMPERATURE" \
        --seed "$SEED" \
        --log-disable \
        2>/dev/null \
        | sed '/^$/d' \
        > "$OUTPUT_FILE" || true

    END_MS=$(python3 -c "import time; print(int(time.monotonic() * 1000))")
    ELAPSED=$((END_MS - START_MS))
    echo "$ELAPSED" >> "$TIMING_FILE"

    SIZE=$(wc -c < "$OUTPUT_FILE" | tr -d ' ')
    printf "  [%3d/%3d] %s  (%d bytes, %dms)\n" "$INDEX" "$COUNT" "$TASK_NAME.sno" "$SIZE" "$ELAPSED"
done

# Summary
if [ -f "$TIMING_FILE" ]; then
    TOTAL_TIME=$(awk '{s+=$1} END {print s}' "$TIMING_FILE")
    AVG_TIME=$(awk '{s+=$1; n++} END {printf "%.0f", s/n}' "$TIMING_FILE")
    echo "───────────────────────────────────────────────────────"
    echo "  Generated: $(ls "$OUTPUT_DIR"/*.sno 2>/dev/null | wc -l | tr -d ' ') programs"
    echo "  Total time: ${TOTAL_TIME}ms"
    echo "  Avg time:   ${AVG_TIME}ms per program"
    rm -f "$TIMING_FILE"
fi

echo "═══════════════════════════════════════════════════════"
echo "  Run validation: python3 validate_e2e.py --input-dir $OUTPUT_DIR --report"
echo "═══════════════════════════════════════════════════════"
