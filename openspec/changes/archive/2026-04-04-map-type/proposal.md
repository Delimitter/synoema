# Proposal: Map Type

## Problem
No key-value data structure. Needed for JSON objects, config, etc.

## Scope
- `Pair a b` ADT + `fst`/`snd` in prelude
- `Map k v` as sorted association list in prelude
- Full API: CRUD, traversal, merge
- All pure functions, no runtime FFI
