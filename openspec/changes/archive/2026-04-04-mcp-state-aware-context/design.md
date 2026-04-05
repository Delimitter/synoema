# Design: MCP State-Aware Context

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  MCP Server                     │
│                                                 │
│  ┌───────────┐     ┌──────────────────────────┐ │
│  │  tools.rs │────▶│  state.rs (NEW)          │ │
│  │           │     │                          │ │
│  │ eval      │     │  AppState enum           │ │
│  │ typecheck │     │  StateTracker            │ │
│  │ run       │     │    .on_tool_result()     │ │
│  │ ...       │     │    .current_state()      │ │
│  │           │     │    .history()            │ │
│  │ get_state │◀────│    .baseline_context()   │ │
│  │ get_ctx   │◀────│                          │ │
│  └───────────┘     └──────────────────────────┘ │
│       │                     ▲                   │
│       │ call result         │ on_tool_result    │
│       ▼                     │                   │
│  ┌───────────┐              │                   │
│  │  main.rs  │──────────────┘                   │
│  │  (loop)   │  after each tool call            │
│  └───────────┘                                  │
└─────────────────────────────────────────────────┘
```

## Key Decision: Global Mutable State

MCP-сервер — однопоточный stdio loop. StateTracker хранится как `thread_local!` или `static Mutex`. Для однопоточного случая `RefCell` в thread-local достаточно.

**Выбор: `thread_local!(RefCell<StateTracker>)`** — простейший вариант, без Mutex overhead. Аналогично `index::global()` который уже использует `thread_local!`.

## Modules

### state.rs (NEW, ~150 LOC)

```rust
pub enum AppState {
    Create,  // Writing code, exploring API
    Check,   // Type errors, compilation issues
    Run,     // Execution, output analysis
    Debug,   // Runtime errors, debugging
}

pub struct StateTracker {
    current: AppState,
    history: Vec<(AppState, String)>,  // (state, trigger_tool)
    last_error: Option<String>,        // Last error JSON for context
}
```

Methods:
- `on_tool_result(tool: &str, is_error: bool)` — transition logic
- `current_state() -> &AppState`
- `history() -> &[(AppState, String)]` — last 5 entries
- `baseline_context() -> String` — state-dependent context
- `set_last_error(json: &str)` — store for Check/Debug context

### Transition Logic

```rust
fn on_tool_result(&mut self, tool: &str, is_error: bool) {
    let next = match (tool, is_error) {
        ("eval", false)             => AppState::Create,
        ("eval", true)              => AppState::Check,
        ("typecheck", false)        => AppState::Check, // still in verify mode
        ("typecheck", true)         => AppState::Check,
        ("run", false)              => AppState::Run,
        ("run", true)               => AppState::Debug,
        ("search_code", _)          => AppState::Create,
        ("get_context_for_edit", _) => AppState::Create,
        ("recipe", _)               => AppState::Create,
        _                           => return, // no transition for unknown tools
    };
    // ... push to history, update current
}
```

### Baseline Context Content

- **Create**: embed `LLM_REF` (already in resources.rs as const) + examples list + tool hints
- **Check**: last_error JSON + brief fix hints
- **Run**: minimal status line
- **Debug**: last_error + available debug tools

## Integration Points

### main.rs

`handle_tools_call` вызывает `state::on_tool_result()` ПОСЛЕ успешного выполнения tool call. Минимальное изменение: 3-5 строк.

### tools.rs

Добавить `get_context` и `get_state` в `list()` и `call()`. Делегировать в `state.rs`.

## Testing Strategy

- Unit tests в `state.rs`: transition table coverage (каждый переход из спеки)
- Unit tests: baseline_context возвращает непустой текст для каждого state
- Integration test: последовательность tool calls → правильные переходы

## Not In Scope

- Event-driven triggers (Phase 2 — будущее)
- Persistent state across restarts
- Explicit state setting by user (Phase 2)
