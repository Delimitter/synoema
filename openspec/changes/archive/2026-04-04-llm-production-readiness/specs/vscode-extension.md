# Spec: VS Code Extension (Restore)

## Scope

Восстановить TextMate grammar для .sno файлов. Расширение находится в `vscode-extension/`.

## Содержимое

```
vscode-extension/
├── package.json           # extension manifest
├── syntaxes/
│   └── synoema.tmLanguage.json   # TextMate grammar
├── language-configuration.json    # bracket matching, comments
└── README.md
```

## TextMate Grammar (ключевые scopes)

| Pattern | Scope |
|---------|-------|
| `--` comments | `comment.line.double-dash` |
| `---` doc comments | `comment.block.documentation` |
| Keywords: `mod`, `use`, `import`, `type`, `trait`, `impl`, `test`, `prop`, `derive` | `keyword.other` |
| `?`, `->`, `:` (ternary) | `keyword.control` |
| Operators: `+`, `-`, `*`, `/`, `==`, etc. | `keyword.operator` |
| String literals | `string.quoted.double` |
| String interpolation `${...}` | `meta.embedded` |
| Number literals | `constant.numeric` |
| Bool literals `true`/`false` | `constant.language` |
| Function definition `name args =` | `entity.name.function` |
| Type names (capitalized) | `entity.name.type` |
| Constructor names (capitalized) | `entity.name.tag` |

## Language Configuration

- Comments: `--` (line), no block comments
- Brackets: `()`, `[]`, `{}`
- Auto-closing: `"`, `(`, `[`, `{`
- Indentation: increase after `=` at end of line
