#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2025-present Synoema Contributors
#
# Adds SPDX license headers to source files that don't already have them.
# Usage: ./scripts/add_headers.sh [--dry-run]

set -euo pipefail

DRY_RUN=false
CHANGED=0
SKIPPED=0

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "=== DRY RUN MODE ==="
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

add_header() {
    local file="$1"
    local header="$2"

    if grep -q "SPDX-License-Identifier" "$file" 2>/dev/null; then
        SKIPPED=$((SKIPPED + 1))
        return
    fi

    CHANGED=$((CHANGED + 1))
    echo "  + $file"

    if [[ "$DRY_RUN" == "true" ]]; then
        return
    fi

    local tmp
    tmp=$(mktemp)
    printf '%s\n\n' "$header" > "$tmp"
    cat "$file" >> "$tmp"
    mv "$tmp" "$file"
}

echo "Adding license headers..."
echo ""

# --- Rust files in lang/crates/ (EXCEPT synoema-codegen/) -> Apache-2.0 ---
echo "== lang/crates/ (Apache-2.0) =="
APACHE_HEADER="// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors"

while IFS= read -r -d '' file; do
    add_header "$file" "$APACHE_HEADER"
done < <(find "$PROJECT_ROOT/lang/crates" -name "*.rs" -not -path "*/synoema-codegen/*" -print0 2>/dev/null | sort -z)

# --- Rust files in lang/crates/synoema-codegen/ -> BUSL-1.1 ---
echo ""
echo "== lang/crates/synoema-codegen/ (BUSL-1.1) =="
BSL_RS_HEADER="// SPDX-License-Identifier: BUSL-1.1
// Copyright (c) 2025-present Andrey Bubnov"

while IFS= read -r -d '' file; do
    add_header "$file" "$BSL_RS_HEADER"
done < <(find "$PROJECT_ROOT/lang/crates/synoema-codegen" -name "*.rs" -print0 2>/dev/null | sort -z)

# --- Files in tools/ (EXCEPT LICENSE and node_modules) -> BUSL-1.1 ---
echo ""
echo "== tools/ (BUSL-1.1) =="
BSL_SCRIPT_HEADER="# SPDX-License-Identifier: BUSL-1.1
# Copyright (c) 2025-present Andrey Bubnov"

while IFS= read -r -d '' file; do
    add_header "$file" "$BSL_SCRIPT_HEADER"
done < <(find "$PROJECT_ROOT/tools" \( -name "*.py" -o -name "*.sh" -o -name "*.rs" \) -not -path "*/node_modules/*" -print0 2>/dev/null | sort -z)

# --- .sno files in lang/examples/ AND examples/ -> MIT-0 ---
echo ""
echo "== examples/ (MIT-0) =="
MIT0_HEADER="-- SPDX-License-Identifier: MIT-0"

for dir in "$PROJECT_ROOT/lang/examples" "$PROJECT_ROOT/examples"; do
    if [[ -d "$dir" ]]; then
        while IFS= read -r -d '' file; do
            add_header "$file" "$MIT0_HEADER"
        done < <(find "$dir" -name "*.sno" -print0 2>/dev/null | sort -z)
    fi
done

echo ""
echo "Done. Changed: $CHANGED, Skipped (already had header): $SKIPPED"

if [[ "$DRY_RUN" == "true" ]]; then
    echo "(Dry run — no files were modified)"
fi
