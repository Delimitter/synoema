# Development Guide

## Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Delimitter/synoema.git
   cd synoema/vscode-extension
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Ensure Synoema CLI is available:**
   ```bash
   synoema --version
   ```

   If not in PATH, install via npm:
   ```bash
   npm install -g synoema
   ```

## Building

```bash
# Build once
npm run build

# Watch for changes during development
npm run watch
```

Output is in `dist/extension.js`.

## Testing

### Local Testing (F5 Debug)

1. Open this directory in VSCode
2. Press `F5` to launch a new VSCode window with the extension
3. Open a `.sno` file
4. Test commands:
   - `Ctrl+Shift+R` — Run File
   - `Ctrl+Shift+J` — JIT File
   - `Ctrl+Shift+X` and select expression, then run Eval Selection
5. Check output in the Synoema output panel

### Testing with Examples

The Synoema repository includes example programs. Copy them to a test directory:

```bash
cp -r ../examples ./test-examples
```

Then test with files like `test-examples/quicksort.sno`.

## Project Structure

```
vscode-extension/
├── src/
│   ├── extension.ts           # Entry point, command registration
│   ├── commands/
│   │   ├── run.ts            # Run command implementation
│   │   ├── jit.ts            # JIT command implementation
│   │   └── eval.ts           # Eval command implementation
│   ├── config.ts             # Configuration utilities
│   ├── output.ts             # Output panel management
│   └── grammar/
│       └── synoema.tmLanguage.json  # Syntax highlighting grammar
├── package.json              # Extension metadata
├── tsconfig.json             # TypeScript configuration
├── language-configuration.json # Language rules
├── .vscodeignore             # Files to exclude from package
└── README.md                 # User documentation
```

## Making Changes

### Adding a New Command

1. Create `src/commands/new-command.ts`:
   ```typescript
   import * as vscode from 'vscode';
   import { getConfig } from '../config';
   import * as output from '../output';

   export async function newCommand(): Promise<void> {
     // Implementation
   }
   ```

2. Register in `src/extension.ts`:
   ```typescript
   import { newCommand } from './commands/new-command';

   const cmd = vscode.commands.registerCommand('synoema.newCommand', newCommand);
   context.subscriptions.push(cmd);
   ```

3. Add to `package.json` contributes:
   ```json
   "commands": [
     {
       "command": "synoema.newCommand",
       "title": "Synoema: New Command"
     }
   ]
   ```

### Updating Syntax Highlighting

Edit `src/grammar/synoema.tmLanguage.json`:
- Add patterns for new token types
- Update scope names to match VSCode color themes
- Reference: [TextMate Grammar](https://macromates.com/manual/en/language_grammars)

### Configuration Options

Add to `package.json` under `contributes.configuration`:

```json
"synoema.newOption": {
  "type": "string",
  "default": "value",
  "description": "Description of the option"
}
```

Then read in `src/config.ts`:

```typescript
const newOption = config.get<string>('newOption') || 'default';
```

## Publishing

### Prerequisites

1. Create a VSCode Marketplace account at https://marketplace.visualstudio.com
2. Create a Personal Access Token (PAT) with "Manage" scope
3. Add the PAT as a GitHub secret `VSCODE_MARKETPLACE_TOKEN`

### Manual Publishing

```bash
npm run package          # Creates .vsix file
vsce publish --pat <TOKEN>  # Publish to marketplace
```

### Automatic Publishing (GitHub Actions)

Tag a release:
```bash
git tag v0.0.2
git push --tags
```

GitHub Actions will automatically build and publish.

## Troubleshooting

### TypeScript Errors

Run the build to check for errors:
```bash
npm run build
```

### Extension Not Loading

1. Check VSCode's output panel for errors
2. Verify TypeScript compilation succeeded
3. Reload VSCode window (Cmd+K Cmd+R)

### Synoema CLI Not Found in Tests

Ensure Synoema is in PATH:
```bash
which synoema
npm run build
```

## Code Quality

- **TypeScript**: Strict mode enabled in `tsconfig.json`
- **No linter**: Project uses natural code style matching Synoema conventions
- **Bundle size**: Target <500KB with esbuild compression

## Performance

- **Build time**: ~2-5 seconds
- **Extension activation**: <100ms
- **Command execution**: Depends on synoema CLI (typically <1s for small programs)

## Future Enhancements

- [ ] LSP (Language Server Protocol) support for real-time diagnostics
- [ ] Debugging support (breakpoints, stepping)
- [ ] Syntax tree visualization
- [ ] Interactive REPL in editor
- [ ] Package manager integration
- [ ] Custom themes for Synoema syntax
