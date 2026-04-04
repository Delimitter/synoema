# Design: Synoema VSCode Extension

## Architecture

```
vscode-extension/
├── src/
│   ├── extension.ts           # Main entry point, command registration
│   ├── commands/
│   │   ├── run.ts            # Execute synoema run
│   │   ├── jit.ts            # Execute synoema jit
│   │   └── eval.ts           # Evaluate selected expression
│   ├── config.ts             # Configuration (synoema path, etc.)
│   ├── output.ts             # Output channel management
│   └── grammar/
│       └── synoema.tmLanguage.json  # TextMate grammar for highlighting
├── package.json              # Extension manifest
├── tsconfig.json             # TypeScript config
├── webpack.config.js         # Bundling config
├── .vscodeignore             # Exclude from package
└── README.md                 # User documentation
```

## Technical Decisions

### 1. Language & Tooling
- **Language**: TypeScript (industry standard for VSCode extensions)
- **Build**: esbuild via webpack (fast, minimal bundle)
- **Runtime**: Node.js 16+ (VSCode requirement)
- **API**: VSCode Extension API (stable, documented)

### 2. Syntax Highlighting
- **Format**: TextMate grammar (`.tmLanguage.json`)
- **Approach**: Scoped tokens matching Synoema lexer
  - `keyword.control`: if, let, where, pattern
  - `keyword.operator`: |>, ->, =>, λ
  - `punctuation.definition`: { } [ ] ( )
  - `string`, `number`, `comment`
- **No semantic highlighting initially** — grammar-only for Phase 1

### 3. Command Execution
- **Method**: `child_process.exec()` to shell `synoema` CLI
- **Configuration**: Read `synoema.path` from VSCode settings
  - Default: assume `synoema` in PATH
  - User can override: `/path/to/synoema` if not in PATH
- **Working directory**: Document root (if multi-root, use file's root)
- **Timeout**: 30 seconds per command

### 4. Output Display
- **Channel**: VSCode Output Panel (`vscode.window.createOutputChannel("Synoema")`)
- **Format**:
  ```
  $ synoema run examples/quicksort.sno
  [12, 5, 3, 15, ...]
  ✓ Duration: 45ms
  ```
- **Errors**: Show in Output + optional error dialog

### 5. Commands Registration
- **Run**: `synoema.run` — execute current file
  - Keybind: `Ctrl+Shift+R` (Windows/Linux), `Cmd+Shift+R` (macOS)
  - Palette: "Synoema: Run File"
- **JIT**: `synoema.jit` — JIT-compile current file
  - Keybind: `Ctrl+Shift+J` (Windows/Linux), `Cmd+Shift+J` (macOS)
  - Palette: "Synoema: JIT File"
- **Eval**: `synoema.eval` — evaluate selected text
  - No default keybind, Palette: "Synoema: Eval Selection"

### 6. Extension Manifest (`package.json`)
- **Publisher**: Will use placeholder `synoema` (publish step configures real account)
- **Name**: `synoema` (lowercase, marketplace convention)
- **Version**: `0.0.1` (incremented on Marketplace publish)
- **Categories**: Programming Languages
- **Keywords**: synoema, language, code-generation, llm
- **Activation**: `onLanguage:synoema` + `onCommand:*`

### 7. Error Handling
- **Missing synoema CLI**: Show notification with instructions
- **Execution timeout**: Notify user + show last output
- **File not saved**: Suggest save or warn about stale code
- **Parse errors**: Display in output, highlight file

## Dependencies

- **@types/vscode**: ^1.75.0 (VSCode API types)
- **@types/node**: ^18.0.0 (Node types)
- No external runtime dependencies (keep extension lightweight)

## Testing Strategy

- **Manual**: Test each command on example files
- **Automation**: Minimal unit tests for config parsing + command building
- **VSCode test suite**: Run in VSCode's test environment (deferred to Phase 2)

## Marketplace Publishing

- **Account**: Created under synoema.dev or personal GitHub account
- **CI/CD**: GitHub Actions to build + publish on tag
- **Icon**: Use project logo from docs/
- **Extension size**: Target <500KB bundled + gzipped

## Configuration (VSCode settings)

```json
{
  "synoema.path": "synoema",           // Path to synoema CLI
  "synoema.showOutput": true,          // Auto-open output on run
  "synoema.timeout": 30000,            // Timeout in ms
  "synoema.workspaceRoot": null        // Override workspace root detection
}
```
