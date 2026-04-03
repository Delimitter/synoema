# Tasks: IO/Effects System

## Checklist

- [x] T1: Add `Type::io()` helper to types.rs
- [x] T2: Retype IO builtins in infer.rs (print, readline, file_read, tcp_*, fd_*)
- [x] T3: IO coercion in unify.rs (IO a ~ a auto-unwrap)
- [x] T4: Verify IO type parsing (IO (), IO Int, String -> IO ())
- [x] T5: Tests — IO types + coercion (15 новых тестов)
- [x] T6: All existing tests pass (705 total, 0 failures, 0 warnings)
- [x] T7: Update docs/llm/synoema.md (IO section + stdlib table)
- [x] T8: Update docs/specs/language_reference.md (IO в implemented)
- [x] T9: Update context/PROJECT_STATE.md, context/PHASES.md, CLAUDE.md
