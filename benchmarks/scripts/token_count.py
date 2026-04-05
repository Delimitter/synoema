#!/usr/bin/env python3
"""Count BPE tokens (cl100k_base) for all language files in a task directory."""

import json
import sys
from pathlib import Path

try:
    import tiktoken
except ImportError:
    print("Error: tiktoken not installed. Run: pip install tiktoken", file=sys.stderr)
    sys.exit(1)

LANG_EXTENSIONS = {
    "synoema": ".sno",
    "python": ".py",
    "javascript": ".js",
    "typescript": ".ts",
    "cpp": ".cpp",
}

def count_tokens(text: str, enc) -> int:
    return len(enc.encode(text))

def main():
    if len(sys.argv) < 2:
        print("Usage: token_count.py <task_directory>", file=sys.stderr)
        sys.exit(1)

    task_dir = Path(sys.argv[1])
    if not task_dir.is_dir():
        print(f"Error: {task_dir} is not a directory", file=sys.stderr)
        sys.exit(1)

    enc = tiktoken.get_encoding("cl100k_base")
    results = {}

    for lang, ext in LANG_EXTENSIONS.items():
        # Find file matching pattern: <task_name><ext>
        files = list(task_dir.glob(f"*{ext}"))
        if not files:
            continue
        # Use the first match
        code = files[0].read_text()
        # Strip SPDX headers and empty lines at top for fair comparison
        lines = code.split('\n')
        stripped = []
        past_header = False
        for line in lines:
            if not past_header and (line.startswith('-- SPDX') or line.startswith('// SPDX') or line.startswith('# SPDX') or line.strip() == ''):
                continue
            past_header = True
            stripped.append(line)
        clean_code = '\n'.join(stripped)
        results[lang] = count_tokens(clean_code, enc)

    print(json.dumps(results))

if __name__ == "__main__":
    main()
