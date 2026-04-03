---
id: spec-guide-files
type: spec
status: done
---

# Spec: Guide Files (исполняемые .sno с narrative)

## Концепция

Любой `.sno` файл с `---` комментариями = документ. Guide-файлы — .sno с metadata header для навигации.

**Ключевое свойство:** guide компилируется и запускается как обычная программа. Если API модуля изменился → guide не компилируется → дрифт обнаружен автоматически.

## Формат guide metadata

```synoema
--- guide: Working with 2D Vectors
--- order: 1
--- requires: basics
```

Metadata — doc-comment строки с prefix `guide:`, `order:`, `requires:`.
Парсятся из doc-comments файла (module-level или top-level до первого Decl).

| Поле | Формат | Обязательно | Описание |
|------|--------|-------------|----------|
| `guide:` | произвольный текст | Да (для guide) | Заголовок |
| `order:` | число или `N.M` | Нет (default: 0) | Позиция в навигации |
| `requires:` | comma-separated names | Нет | Зависимости (другие guides) |

Файл без `--- guide:` — обычный `.sno`, рендерится как документ, но не включается в guide-навигацию.

## Структура guide-файла

```synoema
--- guide: Working with 2D Vectors
--- order: 1

--- # Creating vectors                       ← H1 section
--- Import the Vec2 module:                  ← prose
use Vec2 (make add)                          ← executable code

--- Now create a vector:                     ← prose
v1 = make 3 4                                ← executable code
--- example: v1.x == 3                       ← doctest

--- # Vector arithmetic                      ← H1 section
--- Component-wise addition:                 ← prose
v2 = add v1 (make 1 1)                       ← executable code
--- example: v2 == make 4 5                  ← doctest

--- See also: rotation.guide.sno             ← cross-reference

main = v2                                    ← entry point
```

### Markdown в doc-comments

`---` строки поддерживают ограниченный Markdown:
- `--- # Heading` → H1 section
- `--- ## Heading` → H2 subsection
- `--- text` → paragraph
- `--- - item` → list item
- `--- > quote` → blockquote
- `--- example: expr` → doctest (special)
- `--- [text](file.sno)` → cross-reference

Рендеринг: при `synoema doc` → Markdown prose преобразуется в HTML/MD.

## Filesystem layout

```
myproject/
├── src/
│   ├── Vec2.sno                    ← library module
│   ├── Sort.sno                    ← library module
│   └── main.sno                    ← entry point
├── guides/                          ← guide files (optional convention)
│   ├── 01-basics.guide.sno
│   ├── 02-vectors.guide.sno
│   └── 03-sorting.guide.sno
└── tests/                           ← test files (optional)
```

**Конвенция, не требование:** `.guide.sno` suffix опционален. Определяющий фактор — наличие `--- guide:` metadata.

## Рендеринг (synoema doc)

```bash
synoema doc src/              → API reference из doc-comments
synoema doc guides/           → guide collection с навигацией
synoema doc .                 → всё: API + guides
synoema doc --format md       → Markdown output
synoema doc --format html     → HTML output (default)
```

### Алгоритм рендеринга .sno → документ

1. Lex + Parse → AST с doc-comments
2. Для каждого блока в файле (top-to-bottom):
   - `---` lines → render as Markdown prose
   - Code (Decl/Expr) → render as syntax-highlighted code block
   - `--- example:` → render as code + expected output
3. Metadata (`guide:`, `order:`) → navigation sidebar
4. Type signatures → auto-extract из type checker, show в docs

### Хронология и ветвления

- **Внутри guide:** линейная хронология = порядок строк в файле
- **Между guides:** `--- requires:` определяет граф зависимостей, `--- order:` — последовательность
- **Ветвления:** `--- See also: other.guide.sno` — ссылки на альтернативные пути

## Гарантии

| Свойство | Гарантия |
|----------|----------|
| Guide компилируется | `synoema run guide.sno` → success или ошибка |
| Doctests в guide проверены | `synoema test guide.sno` → all examples pass |
| API дрифт обнаружен | Если функция переименована → guide не компилируется |
| Type дрифт обнаружен | Если тип изменился → type checker ловит в guide |

## Scope этого change

**В scope:**
- Metadata parsing из doc-comments (guide/order/requires)
- `synoema test` для doctests в guide-файлах
- Базовый `synoema doc --format md` (Markdown output)

**Отложено:**
- HTML renderer с навигацией и стилями
- Search по doc-comments
- MCP tool `docs()` (отдельный change)
