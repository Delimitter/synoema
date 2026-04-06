# LLM Prompt Templates

Task-specific prompt templates for small model (4B-32B) Synoema code generation.

## Templates

| Template | Tokens | Best For |
|----------|--------|----------|
| `arithmetic.md` | ~540 | Recursion, math, conditionals |
| `lists.md` | ~730 | List ops, HOF, comprehensions |
| `adt-patterns.md` | ~620 | ADTs, pattern matching, Result type |
| `records-maps.md` | ~610 | Records, Maps, key-value data |
| `string-io.md` | ~600 | String processing, IO, readline |

## Usage

1. Select template matching your task category
2. Prepend template content to your generation prompt
3. Optionally inject additional gotchas from `gotcha-map.json`

```bash
# Example with llama.cpp
TEMPLATE=$(cat docs/llm/templates/lists.md)
PROMPT="$TEMPLATE\n\nWrite a function that removes duplicates from a list."
./main -m model.gguf --grammar-file synoema.gbnf -p "$PROMPT"
```

## Gotcha Injection

`gotcha-map.json` maps feature keywords to gotcha IDs from the gotcha table.
Use when your task spans multiple categories:

```python
import json
features = ["lists", "strings"]  # detected from task
gotchas = json.load(open("gotcha-map.json"))
ids = set()
for f in features:
    ids.update(gotchas.get(f, []))
# inject gotchas with these IDs into prompt
```

## Compact Reference

For tasks that don't fit a single category, use `docs/llm/synoema-compact.md` (~900 tokens) — a general-purpose condensed reference with all gotchas and 2 examples.
