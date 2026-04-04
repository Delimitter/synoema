# Tasks: Map Type

- [x] Add `Pair a b = MkPair a b` + `fst`/`snd` to prelude.sno
- [x] Add `Map k v = MkMap [Pair k v]` to prelude.sno
- [x] Add construction: `map_empty`, `map_singleton`, `from_pairs`
- [x] Add lookup: `map_lookup`, `map_get`, `has_key`
- [x] Add modification: `map_insert`, `map_delete`, `map_update`
- [x] Add traversal: `map_keys`, `map_values`, `entries`, `map_size`, `map_map_values` (`map_fold` removed — JIT verifier error, see archive/2026-04-04-fix-hm)
- [x] Add `map_merge`
- [x] Test: basic CRUD via `synoema eval`
- [x] `cargo test` — 0 failures
