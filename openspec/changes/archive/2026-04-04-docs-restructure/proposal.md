---
id: proposal
type: proposal
status: done
---

# Proposal: Documentation Restructure — Three Documents, Three Audiences

## Problem

Документация Synoema страдает от дублирования и отсутствия единой концепции:

1. **10 README-файлов**, многие дублируют друг друга
2. **root README.md и lang/README.md — побайтовая копия** (359 строк каждый)
3. Root README пытается быть одновременно landing page, туториалом, справочником, гидом по установке, описанием архитектуры и roadmap
4. `docs/user/README.md` — скелет с 5 TODO-заглушками, не раскрыт
5. Информация об установке дублируется в 6+ местах (root README, lang/README, docs/install.md, releases/README, platform READMEs)
6. Нет единого справочника языка для человека — правила размазаны между root README (таблицы), docs/llm/synoema.md (LLM ref), docs/specs/language_reference.md (формальная спека)
7. Нет гида для контрибьюторов — архитектура и dev-инструкции утоплены в root README

**Результат:** даже автор проекта путается в собственной документации.

## Goal

Создать три чётких документа для трёх аудиторий:

| Документ | Аудитория | Назначение |
|----------|-----------|-----------|
| **README.md** (root) | Новый пользователь | Как попробовать за 2 минуты, Quick Wins, сценарии из коробки |
| **docs/LANGUAGE.md** | Программист на Synoema | Единый справочник языка с объяснениями и примерами |
| **CONTRIBUTING.md** (root) | Разработчик компилятора | Clone → build → test → architecture → как контрибьютить |

## Scope

### In-scope
- Переписать root `README.md` в Quick Wins формат
- Создать `docs/LANGUAGE.md` — единый user-facing справочник языка
- Создать `CONTRIBUTING.md` — developer guide
- Сократить `lang/README.md` до указателя на root
- Удалить `docs/user/README.md` (роль переходит к root README)

### Out-of-scope
- `docs/llm/*` — другая аудитория, не трогаем
- `docs/specs/*` — формальные спеки, не трогаем
- `docs/articles/*` — маркетинг, не трогаем
- `releases/**/README.md` — platform-specific, минимальные
- `vscode-extension/README.md` — npm package, не трогаем
- Изменения компилятора или тестов
- `docs/install.md` — оставляем как есть, ссылаемся

## Output files

- `README.md` — переписан (Quick Wins, ~150 строк вместо 359)
- `docs/LANGUAGE.md` — создан (~400 строк, единый справочник)
- `CONTRIBUTING.md` — создан (~150 строк, dev guide)
- `lang/README.md` — сжат до 5-строчного указателя
- `docs/user/README.md` — удалён
