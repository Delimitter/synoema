# Tasks: fix-init-templates

## Files to modify

- `lang/templates/main.sno.tmpl`
- `lang/templates/test.sno.tmpl`
- `lang/templates/AGENTS.md.tmpl`

## Checklist

- [x] T1: Update main.sno.tmpl — add license header, file-level doc-comment with doctest
- [x] T2: Update test.sno.tmpl — replace `--- test:`/`--- expect:` with real `test "name" = expr` syntax
- [x] T3: Update AGENTS.md.tmpl — add "File Conventions" section documenting: license headers, doc-comments, doctests, file structure order
- [x] T4: cargo build — verify templates compile into binary
- [x] T5: Smoke test — run `synoema init` in temp dir, verify generated files match conventions
