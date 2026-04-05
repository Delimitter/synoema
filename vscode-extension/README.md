# Synoema VSCode Extension

A VSCode extension for the [Synoema programming language](https://github.com/Delimitter/synoema) — a language optimized for code generation by LLMs.

## Features

- **Syntax Highlighting** — Full support for Synoema syntax including keywords, operators, strings with interpolation, comments, and builtins
- **Run Command** — Execute Synoema programs directly from the editor
- **JIT Compilation** — Compile programs to native code using Cranelift JIT
- **Expression Evaluation** — Quickly evaluate Synoema expressions
- **Integrated Output** — View results and errors in a dedicated output panel
- **Language Support** — Language configuration for brackets, comments, and indentation

## Installation

### From VSCode Marketplace

1. Open VSCode
2. Go to Extensions (Ctrl+Shift+X / Cmd+Shift+X)
3. Search for "Synoema"
4. Click Install

### From Source

```bash
git clone https://github.com/Delimitter/synoema.git
cd vscode-extension
npm install
npm run build
vsce package
# Then install the .vsix file manually
```

## Requirements

- VSCode 1.75.0 or later
- Synoema CLI installed and accessible in your PATH

### Install Synoema CLI

```bash
npm install -g synoema
# or
brew install synoema  # macOS (if available)
```

If `synoema` is not in your PATH, configure the extension to point to the correct location (see Configuration below).

## Usage

### Run File

**Keybinding:** `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (macOS)

**Command Palette:** `Synoema: Run File`

Executes the current `.sno` file. Output and any errors are displayed in the Synoema output panel. The file is automatically saved before execution.

Example:
```sno
main = [1 2 3] |> map (\x -> x * 2) |> show
```

Press `Ctrl+Shift+R` to see: `[2, 4, 6]`

### JIT Compile

**Keybinding:** `Ctrl+Shift+J` (Windows/Linux) or `Cmd+Shift+J` (macOS)

**Command Palette:** `Synoema: JIT File`

Compiles the current file to native machine code using the Cranelift JIT. Execution time is shown in the output panel.

### Evaluate Expression

**Command Palette:** `Synoema: Eval Selection`

Select an expression in the editor and run this command to evaluate it without needing a full program. The result is displayed in the output panel.

Example:
1. Select `6 * 7` in your editor
2. Run `Synoema: Eval Selection`
3. Output shows: `42`

## Configuration

Open VSCode settings and search for `synoema` to configure:

| Setting | Default | Description |
|---------|---------|-------------|
| `synoema.path` | `"synoema"` | Path to the Synoema CLI executable. Set to full path if not in PATH. |
| `synoema.showOutput` | `true` | Automatically show the output panel when running commands. |
| `synoema.timeout` | `30000` | Timeout for Synoema commands in milliseconds. |

### Example Configuration

If Synoema is installed at `/usr/local/bin/synoema`:

```json
{
  "synoema.path": "/usr/local/bin/synoema"
}
```

## Troubleshooting

### "Synoema CLI not found"

**Problem:** You see a warning that Synoema CLI is not found.

**Solution:**
1. Ensure Synoema is installed: `which synoema`
2. If installed but not in PATH, configure `synoema.path` in VSCode settings
3. Restart VSCode after installation

### Command Times Out

**Problem:** Execution takes longer than the timeout duration.

**Solution:**
Increase `synoema.timeout` in settings (value in milliseconds):

```json
{
  "synoema.timeout": 60000
}
```

### Syntax Highlighting Not Working

**Problem:** `.sno` files are not highlighted.

**Solution:**
1. Verify the file extension is `.sno`
2. Check that the language is recognized: click the language selector (bottom right) and choose "Synoema"
3. Reload VSCode window (Cmd+R / Ctrl+K Ctrl+R)

### File Not Saved Warning

**Problem:** Changes aren't reflected when running commands.

**Solution:**
The extension automatically saves the file before execution. If you see an error, ensure the file was saved successfully (no dot indicator on the file tab).

## Examples

Synoema comes with example programs in the `examples/` directory:

- `quicksort.sno` — Quicksort implementation with pattern matching
- `factorial.sno` — Recursive factorial
- `fibonacci.sno` — Fibonacci sequence
- And more!

Open any example file and press `Ctrl+Shift+R` to run it.

## Performance

- **Run Mode** — Interpreted, suitable for development
- **JIT Mode** — Compiled to native code with Cranelift, 4.4× faster than interpreted execution on benchmarks

## Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|---------------|-------|
| Run File | `Ctrl+Shift+R` | `Cmd+Shift+R` |
| JIT File | `Ctrl+Shift+J` | `Cmd+Shift+J` |
| Eval Selection | — | — |

## Language Reference

For syntax, builtins, and language features, see:
- [Language Reference](https://github.com/Delimitter/synoema/blob/main/docs/specs/language_reference.md)
- [LLM Quick Reference](https://github.com/Delimitter/synoema/blob/main/docs/llm/synoema.md)

## Contributing

Found a bug or have a feature request? Open an issue on [GitHub](https://github.com/Delimitter/synoema/issues).

## License

MIT — See [LICENSE](https://github.com/Delimitter/synoema/blob/main/LICENSE)

## About Synoema

Synoema is a programming language optimized for code generation by large language models (LLMs). Key benefits:

- **-46% tokens** compared to Python
- **4.4× faster** than Python (with JIT)
- **100% syntactically correct** output via GBNF constrained decoding

Learn more: [GitHub](https://github.com/Delimitter/synoema)
