# Proposal: Result Type — расширение prelude

## Problem
Result + 7 комбинаторов уже в prelude. Для больших программ нужны: `fold_result`, `sequence_results`. Документация stdlib.md не содержит Result (уже обновлена пользователем).

## Scope
- Добавить `fold_result` и `sequence_results` в prelude.sno
- Проверить что всё компилируется с prelude через cargo test
