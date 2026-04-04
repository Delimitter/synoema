---
id: proposal
type: proposal
status: done
---

# Proposal: Documentation Language Compliance

## Problem

Правила проекта (RULES.md §7) определяют аудитории документации и директории, но не фиксируют язык документов. В результате:

1. `docs/testing.md` и `docs/stress-server.md` — на русском, хотя это пользовательские документы
2. `docs/llm/synoema.md` — написан human-readable prose, хотя правило 7b требует "нет prose — только таблицы, списки, однострочные примеры"
3. Нет формального правила, на каком языке писать документы каждой аудитории

## Goal

1. Добавить в RULES.md §7 правила языка документации:
   - LLM-документы (`docs/llm/`) — минифицированный английский (без prose)
   - Пользовательские документы — только английский, человекопонятный
2. Привести 3 файла-нарушителя в соответствие

## Scope

### In-scope
- Добавить секцию 7d "Язык документации" в `context/RULES.md`
- Перевести `docs/testing.md` на английский
- Перевести `docs/stress-server.md` на английский
- Минифицировать prose в `docs/llm/synoema.md` (убрать полные предложения, оставить таблицы/списки)

### Out-of-scope
- `context/` файлы — они для LLM-разработчика (Claude agent), не затронуты новыми правилами
- `CLAUDE.md` — точка входа для Claude, на русском by design
- Разбивка `docs/llm/synoema.md` на фрагменты (правило 7c) — отдельный change
- Обновление числовых метрик — отдельный change

## Output files

- `context/RULES.md` — добавлена секция 7d
- `docs/testing.md` — переведён на EN
- `docs/stress-server.md` — переведён на EN
- `docs/llm/synoema.md` — минифицирован
