---
id: tasks
type: tasks
status: done
---

# Tasks: Task-Specific RAG Templates

- [x] **T1: Create template directory structure**
  - `docs/llm/templates/` directory
  - `docs/llm/templates/README.md` with usage guide (378 tokens)

- [x] **T2: Create gotcha injection map**
  - `docs/llm/templates/gotcha-map.json`
  - 11 feature categories mapped to gotcha IDs

- [x] **T3: Create `arithmetic.md` template** (541 tokens)

- [x] **T4: Create `lists.md` template** (726 tokens)
  - Fixed pipeline example (multiline pipe indentation issue)

- [x] **T5: Create `adt-patterns.md` template** (612 tokens)
  - Used Int-based ADT (Float * in multi-equation has type checker limitation)

- [x] **T6: Create `records-maps.md` template** (610 tokens)

- [x] **T7: Create `string-io.md` template** (605 tokens)
  - Replaced interactive IO example with string builder (bind `<-` limited)

- [x] **T8: Verify token counts**
  - arithmetic: 541, lists: 726, adt: 612, records: 610, string-io: 605
  - All within target range (500-750 tokens)

- [x] **T9: Validate all examples compile**
  - All 10 examples: parse ✓ typecheck ✓ run ✓
