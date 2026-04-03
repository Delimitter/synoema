---
id: design
type: design
status: done
---

# Design: Region Inference

## Компонент 1: Runtime — Region Stack

### Текущее состояние

Одна глобальная arena с bump pointer. `arena_save()`/`arena_restore()` — ручной checkpoint.

### Решение

Заменяем save/restore семантику на стек регионов. Каждый `region_enter` создаёт checkpoint (запоминает offset), каждый `region_exit` восстанавливает offset. Это по сути автоматический save/restore, но управляемый компилятором.

```rust
// runtime.rs — добавить к Arena

const MAX_REGION_DEPTH: usize = 64;

struct Arena {
    ptr: *mut u8,
    offset: usize,
    overflow_allocs: Vec<(*mut u8, Layout)>,
    overflow_warned: bool,
    // NEW: region stack
    region_stack: [usize; MAX_REGION_DEPTH],  // saved offsets
    region_depth: usize,
}

impl Arena {
    fn region_enter(&mut self) {
        if self.region_depth >= MAX_REGION_DEPTH {
            eprintln!("[synoema] region depth limit reached, skipping");
            return;
        }
        self.region_stack[self.region_depth] = self.offset;
        self.region_depth += 1;
    }

    fn region_exit(&mut self) {
        if self.region_depth == 0 { return; }
        self.region_depth -= 1;
        self.offset = self.region_stack[self.region_depth];
        // Note: overflow_allocs between enter/exit are NOT freed here
        // (they belong to the global overflow pool, freed at arena_reset)
    }
}

// FFI exports
pub extern "C" fn synoema_region_enter() -> i64 {
    ARENA.with(|a| { a.borrow_mut().region_enter(); 0 })
}

pub extern "C" fn synoema_region_exit() -> i64 {
    ARENA.with(|a| { a.borrow_mut().region_exit(); 0 })
}
```

**Обратная совместимость:** `arena_save()` / `arena_restore()` остаются для ручного использования. Region stack — дополнительный механизм.

## Компонент 2: TCO Auto-Regions

### Текущее состояние

TCO в compiler.rs: `TcoContext` имеет `loop_block` и `params`. Tail calls компилируются как jump к `loop_block`.

### Решение

В `compile_top_level_function`, после создания loop header и перед первой инструкцией тела:
1. Emit `call synoema_region_enter()`
2. Перед каждым jump к loop_block (tail call): emit `call synoema_region_exit()`

```
  loop_header:
    region_enter()           ← NEW
    ... body evaluation ...
    region_exit()            ← NEW (before back-edge)
    jump loop_header
  exit:
    region_exit()            ← NEW (before normal return)
    return result
```

Все аллокации в теле цикла попадают в child region. При tail call → region_exit возвращает offset, все аллокации итерации стираются. Результат возвращается через параметры (unboxed i64), не через heap.

**Важно:** return value из последней итерации (не tail call) должен быть аллоцирован ПОСЛЕ region_exit. Для этого: при компиляции return-выражения в TCO-функции, emit region_exit() перед вычислением return value.

Нет — это неправильно, потому что return value может использовать значения из текущей итерации. Правильный подход:

**Для TCO-циклов return value не существует в обычном смысле** — если функция tail-recursive, каждый не-tail-call branch возвращает значение напрямую. Эти return values — чаще всего base-case литералы или аккумуляторы (параметры). Они уже в параметрах loop, не в heap текущей итерации.

Поэтому для tail-recursive functions: `region_exit()` перед КАЖДЫМ jump к loop_block безопасен. Для non-tail branches (return): не делаем region_exit, пусть arena_reset в конце программы очистит.

Уточнённая схема:
```
  loop_header:
    region_enter()
    ... body ...
    // tail-call path:
    region_exit()
    set_params(new_args)
    jump loop_header
    // base-case path (return):
    // NO region_exit — allocations survive for return value
    return result
```

## Компонент 3: Escape Analysis на Core IR

### Определение

Значение `x` **утекает** (escapes) из scope `let x = e1 in e2`, если:
1. `x` появляется в return-позиции `e2` (body оценивается в `x` или содержит `x` в структуре)
2. `x` захвачен closure в `e2`, которая утекает
3. `x` передан функции, чей результат утекает, И функция может сохранить аргумент

### Упрощения для Synoema

- Нет мутации → значение либо escapes при создании, либо никогда
- Builtin functions имеют known semantics: `length xs` потребляет xs, не сохраняет; `show x` потребляет, возвращает новую строку
- Конструкторы (`Cons`, `Just`, record literal) всегда сохраняют аргументы → escape

### Алгоритм: Free Variable + Return Position Analysis

```rust
// synoema-core/src/escape.rs (новый файл? нет — расширяем optimize.rs)
// Нет, правило: не плодить файлы. Добавляем в optimize.rs.

/// Determine if a let-bound variable escapes its body.
fn escapes(var: &str, body: &CoreExpr) -> bool {
    match body {
        // x is returned directly → escapes
        CoreExpr::Var(n) => n == var,

        // x in a data structure → escapes
        CoreExpr::MkList(elems) => elems.iter().any(|e| mentions(var, e)),
        CoreExpr::Record(fields) => fields.iter().any(|(_, e)| mentions(var, e)),
        CoreExpr::App(CoreExpr::Con(_), arg) => mentions(var, arg),

        // let y = e1 in e2: x escapes if it escapes from e2,
        // OR if it escapes from e1 AND y escapes from e2
        CoreExpr::Let(y, val, inner) => {
            escapes(var, inner) || (mentions(var, val) && escapes(y, inner))
        }

        // Case: x escapes if it escapes any branch
        CoreExpr::Case(scrut, alts) => {
            mentions(var, scrut) || alts.iter().any(|a| escapes(var, &a.body))
        }

        // Lambda: x captured in closure → escapes (conservative)
        CoreExpr::Lam(_, body) => mentions(var, body),

        // App(f, arg): if x is the arg of a known-consuming builtin → doesn't escape
        // Otherwise: conservative → escapes if mentioned
        CoreExpr::App(func, arg) => {
            if is_consuming_builtin(func) && mentions(var, arg) && !mentions(var, func) {
                false  // builtin consumes argument, doesn't retain
            } else {
                mentions(var, func) || mentions(var, arg)
            }
        }

        // Literals, PrimOp → no escape
        CoreExpr::Lit(_) | CoreExpr::PrimOp(_) | CoreExpr::Con(_) => false,

        _ => mentions(var, body),  // conservative
    }
}

/// Check if var is free in expr (syntactic mention)
fn mentions(var: &str, expr: &CoreExpr) -> bool { ... }

/// Known builtins that consume their argument without retaining
fn is_consuming_builtin(expr: &CoreExpr) -> bool {
    matches!(expr,
        CoreExpr::App(box CoreExpr::PrimOp(
            PrimOp::Show | PrimOp::Print | PrimOp::Seq
        ), _)
        | CoreExpr::Var(n) if matches!(n.as_str(),
            "length" | "sum" | "str_len" | "str_find" | "str_trim"
        )
    )
}
```

### Консервативность

Escape analysis MUST be conservative: если сомневаемся — считаем что escapes. False positive (считаем escaping когда не escapes) = пропущенная оптимизация. False negative (считаем non-escaping когда escapes) = USE-AFTER-FREE.

## Компонент 4: Region Annotation Pass

Новый проход после оптимизации, перед codegen. Добавляет `RegionEnter`/`RegionExit` в Core IR.

### Core IR расширение

```rust
// core_ir.rs — два новых варианта

pub enum CoreExpr {
    // ... existing variants ...

    /// Enter a new memory region (push arena checkpoint).
    /// Body is evaluated in the new region. Result may escape.
    /// region_exit is implicit at the end of body evaluation.
    Region(Box<CoreExpr>),
}
```

Один вариант `Region(body)` вместо пары Enter/Exit — проще для codegen (structured scope). Codegen: `region_enter()` перед body, `region_exit()` после body.

### Pass

```rust
// optimize.rs — add annotate_regions pass

pub fn annotate_regions(program: CoreProgram) -> CoreProgram {
    let defs = program.defs.into_iter().map(|def| CoreDef {
        name: def.name,
        body: annotate_expr(def.body),
    }).collect();
    CoreProgram { defs, ctor_tags: program.ctor_tags }
}

fn annotate_expr(expr: CoreExpr) -> CoreExpr {
    match expr {
        CoreExpr::Let(name, val, body) => {
            let val = annotate_expr(*val);
            let body = annotate_expr(*body);

            // Check: does val allocate heap? And does name NOT escape body?
            if allocates_heap(&val) && !escapes(&name, &body) {
                // Wrap the entire let in a region
                CoreExpr::Region(Box::new(
                    CoreExpr::Let(name, Box::new(val), Box::new(body))
                ))
            } else {
                CoreExpr::Let(name, Box::new(val), Box::new(body))
            }
        }
        // Recurse into other forms
        CoreExpr::Lam(p, b) => CoreExpr::Lam(p, Box::new(annotate_expr(*b))),
        CoreExpr::Case(s, alts) => CoreExpr::Case(
            Box::new(annotate_expr(*s)),
            alts.into_iter().map(|a| Alt {
                pat: a.pat,
                body: annotate_expr(a.body),
            }).collect()
        ),
        other => other, // Lit, Var, etc — no change
    }
}

/// Does this expression allocate heap objects?
fn allocates_heap(expr: &CoreExpr) -> bool {
    matches!(expr,
        CoreExpr::MkList(_)
        | CoreExpr::Record(_)
        | CoreExpr::App(_, _)  // function calls may allocate
    )
}
```

## Компонент 5: Codegen для Region

```rust
// compiler.rs — compile_expr

CoreExpr::Region(body) => {
    // Enter region
    let enter_fn = self.functions["synoema_region_enter"];
    let enter_ref = self.module.declare_func_in_func(enter_fn, builder.func);
    builder.ins().call(enter_ref, &[]);

    // Compile body
    let result = self.compile_expr(builder, body, vars, tco)?;

    // Exit region
    let exit_fn = self.functions["synoema_region_exit"];
    let exit_ref = self.module.declare_func_in_func(exit_fn, builder.func);
    builder.ins().call(exit_ref, &[]);

    Ok(result)
}
```

## Взаимодействие компонентов

```
Source (.sno) → Lexer → Parser → Types → Core IR → Optimizer
                                                       │
                                              ┌────────┴────────┐
                                              ▼                  ▼
                                         annotate_regions    (unchanged
                                              │               for interp)
                                              ▼
                                    Core IR with Region nodes
                                              │
                                              ▼
                                         Compiler (JIT)
                                              │
                                    ┌─────────┼─────────┐
                                    ▼         ▼         ▼
                              TCO auto-   Region     normal
                              regions     nodes      codegen
                                    │         │
                                    ▼         ▼
                              region_enter / region_exit FFI calls
                                          │
                                          ▼
                                    Runtime (arena region stack)
```

## Безопасность

1. **Region stack overflow**: MAX_REGION_DEPTH=64, с warning при превышении
2. **Use-after-free prevention**: conservative escape analysis — if in doubt, don't create region
3. **TCO regions**: только для tail-call back-edge, не для return path
4. **Nested regions**: корректны — inner region_exit не затрагивает outer region
5. **arena_save/restore совместимость**: region_enter/exit используют тот же offset, но через стек вместо ручных значений

## Known Limitations

1. **App(...) считается allocating** — conservative, может пропустить оптимизации для non-allocating functions
2. **Closures conservative** — если переменная упоминается в lambda, считаем escaped
3. **Нет cross-function analysis** — каждая функция анализируется изолированно
4. **TCO regions не для mutual recursion** — только self-recursive TCO
