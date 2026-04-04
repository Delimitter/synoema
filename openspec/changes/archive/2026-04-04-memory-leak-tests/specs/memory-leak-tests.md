# Delta Spec: Memory Leak Tests

## Capability
Тесты на проверку утечек памяти и корректности высвобождения в JIT runtime.

## New Public API
```rust
// runtime.rs — новая функция для тестов
pub fn arena_offset() -> usize;
pub fn arena_overflow_count() -> usize;
pub fn arena_region_depth() -> usize;
```

## Test Categories

### 1. Arena Leak Detection (M-1)
- `arena_offset()` == 0 после `arena_reset()`
- overflow_allocs.len() == 0 после `arena_reset()`
- region_depth == 0 после `arena_reset()`

### 2. Region Balance (M-2)
- region_enter + region_exit → offset возвращается к исходному
- Вложенные регионы: depth растёт и падает корректно
- Несбалансированные region_exit → depth не уходит ниже 0

### 3. Leak Audit Existing Tests (M-3)
- Каждый JIT-тест: arena_reset → arena_offset == 0
- Overflow allocs: очищаются после каждого цикла
- Обнаруженные утечки → исправить

### 4. Stress: Repeated Alloc-Reset Cycles (M-4)
- 1000 циклов alloc-reset → arena_offset стабильно 0
- Нет роста overflow_allocs между циклами

## Files Changed
- `lang/crates/synoema-codegen/src/runtime.rs` — новые pub fn для introspection
- `lang/crates/synoema-codegen/tests/stress.rs` — новые тесты M-1..M-4
