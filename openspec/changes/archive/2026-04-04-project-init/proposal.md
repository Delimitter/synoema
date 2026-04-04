# Proposal: Project Init — `synoema init` command

## Problem
LLM не может инициализировать Synoema-проект с нуля. Нет scaffolding, нет конвенции структуры.

## Scope
- CLI команда `synoema init [name]` с шаблонами
- Конвенция структуры проекта (src/, tests/, project.sno, CLAUDE.md)
- Шаблоны в `lang/templates/`

## Not in Scope
- Prelude mechanism (уже работает)
- Build command (отдельный change)
- Package management
