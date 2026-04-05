# Proposal: fix-init-templates

## Problem

`synoema init` creates scaffolding that does NOT follow the project's own documentation conventions:

1. **main.sno.tmpl** — no license header, no doc-comments, no doctests. Every example in `lang/examples/` has all three.
2. **test.sno.tmpl** — uses phantom syntax `--- test:` / `--- expect:` that doesn't exist in the language. Real syntax: `test "name" = expr` or `--- example: expr == val`.
3. **AGENTS.md.tmpl** — has language reference but no file conventions section. LLMs know HOW to write code but not HOW to document/organize files.

## Cascade Effect

Templates set the convention for every new project. Undocumented templates → undocumented projects. bench-viz is direct proof: no doc-comments anywhere.

## Scope

4 template files in `lang/templates/`:
- `main.sno.tmpl` — add license, doc-comment, doctest
- `test.sno.tmpl` — replace phantom syntax with real `test` declarations
- `AGENTS.md.tmpl` — add "File Conventions" section
- `CLAUDE.md.tmpl` — no changes needed (thin pointer is correct)

## Non-goals

- Not changing init logic in main.rs (that's a separate fix for the full-path name bug)
- Not retrofitting bench-viz (user will re-init)
