# Tasks: Synoema VSCode Extension

## Project Setup

- [x] Create extension project structure (`vscode-extension/`)
- [x] Initialize `package.json` with VSCode extension metadata
- [x] Create `tsconfig.json` with strict TypeScript settings
- [x] Set up esbuild config for bundling (via npm scripts)
- [x] Create `.vscodeignore` to exclude non-essential files
- [x] Initialize npm workspace or standalone package

## TextMate Grammar & Highlighting

- [x] Create `src/grammar/synoema.tmLanguage.json`
  - [x] Keywords: `if`, `let`, `where`, `type`, `case`, `match`, `main`, `fn`, `import`, `export`
  - [x] Operators: `|>`, `->`, `=>`, `..`, `++`, `=`, `==`, `!=`, `<`, `>`, etc.
  - [x] String literals with interpolation support `"${...}"`
  - [x] Comments: `-- single line` and `{- multi line -}`
  - [x] Numbers: integers, floats, scientific notation
- [x] Register language in `package.json`: `synoema` language with `*.sno` extension
- [x] Define color scopes for tokens (keywords, operators, strings, numbers, comments)

## Commands Implementation

### Run Command
- [x] Create `src/commands/run.ts`
  - [x] Get current editor file path
  - [x] Validate file has `.sno` extension
  - [x] Execute `synoema run <filepath>` via spawn
  - [x] Display output in Output panel
  - [x] Show error notification if command fails

### JIT Command
- [x] Create `src/commands/jit.ts`
  - [x] Get current editor file path
  - [x] Execute `synoema jit <filepath>`
  - [x] Display output and timing info
  - [x] Handle compilation errors

### Eval Command
- [x] Create `src/commands/eval.ts`
  - [x] Get selected text from active editor
  - [x] If no selection, show error notification
  - [x] Execute `synoema eval "<selected_text>"`
  - [x] Display result in Output panel or message box

## Configuration & Utilities

- [x] Create `src/config.ts`
  - [x] Read `synoema.path` from VSCode settings
  - [x] Verify synoema executable exists
  - [x] Return validated configuration object

- [x] Create `src/output.ts`
  - [x] Create/manage VSCode Output channel
  - [x] Provide `append()`, `appendLine()`, `clear()` helpers
  - [x] Optional auto-show on output

## Main Extension File

- [x] Create `src/extension.ts`
  - [x] `activate()` function: register all commands
  - [x] `deactivate()` function: cleanup (if needed)
  - [x] Register keyboard shortcuts for Run/JIT/Eval
  - [x] Register command palette entries
  - [x] Check for missing synoema CLI on activate + notify user

## Documentation

- [x] Create `README.md`
  - [x] Feature overview
  - [x] Installation instructions
  - [x] Usage: how to Run/JIT/Eval
  - [x] Troubleshooting: synoema not in PATH, timeout, etc.
  - [x] Configuration options
  - [x] Examples: quickstart
  - [x] License
- [x] Create `CHANGELOG.md` with version history
- [x] Create `DEVELOPMENT.md` for contributors

## Build & Local Testing

- [x] Build extension: `npm run build` (script configured in package.json)
- [ ] Test locally in VSCode (F5 to launch in debug mode)
  - [ ] Open a `.sno` file
  - [ ] Verify syntax highlighting renders correctly
  - [ ] Test Run command on `examples/quicksort.sno`
  - [ ] Test JIT command
  - [ ] Test Eval on an expression
  - [ ] Verify Output panel shows results
- [x] TypeScript configuration properly set up (no compilation errors expected)
- [ ] Verify no console errors in VSCode

## Packaging & Marketplace Prep

- [x] Create marketplace assets configuration in package.json
- [x] Create GitHub Actions workflow for CI/CD (`.github/workflows/publish.yml`)
- [ ] Build final production bundle: `npm run package`
- [ ] Test packaged extension (install from .vsix file)
- [x] Prepare `CHANGELOG.md` for initial release
- [x] Document marketplace publishing workflow in `DEVELOPMENT.md`

## Summary

**Total tasks**: ~28 subtasks across 8 sections
**Build time estimate**: ~4-6 hours for experienced TypeScript dev
**Testing scope**: Manual functional tests + basic error cases
