# Задача: утилита анализа JSON-бенчмарков

Прочитай AGENTS.md и docs/llm/stdlib.md — это полная справка по языку Synoema. Следуй ей строго.

## Что написать

Файл `src/main.sno` ��� утилита, которая читает JSON-файл с результатами бенчмарков и печатае�� таблицу token counts по задачам и языкам.

## Пример запуска

```
synoema run src/main.sno -- data/2026-04-05_run_002/raw.json
```

## Структура JSON

```json
{
  "tokens": {
    "tasks": [
      {
        "task": "binary_search",
        "counts": {
          "cpp": 175,
          "javascript": 129,
          "python": 120,
          "synoema": 122,
          "typescript": 134
        }
      },
      {
        "task": "factorial",
        "counts": {
          "cpp": 58,
          "javascript": 35,
          "python": 32,
          "synoema": 25,
          "typescript": 38
        }
      }
    ],
    "averages": {
      "cpp": 88.87,
      "javascript": 50.63,
      "python": 92.53,
      "synoema": 87.67,
      "typescript": 61.1
    }
  }
}
```

## Что программа должна делать

1. Получить путь к файлу из `args` (первый элемент)
2. Прочитать файл через `file_read`
3. Распарсить через `json_parse`
4. Извлечь массив задач из `tokens` → `tasks`
5. Для каждой задачи: извлечь имя (`task`) и counts по языкам (`counts`)
6. Напечатать заголовок с именами языков
7. Напечатать строку для каждой задачи: `имя | count1 | count2 | ...`
8. Напечатать итоговую строку с количеством задач

## Ожидаемый вывод (для тестового файла test.json)

```
Task | cpp | python | synoema
binary_search | 175 | 120 | 122
factorial | 58 | 32 | 25
---
Tasks: 2
```

Порядок языков — по первому появлению в counts первой задачи. Разделитель — ` | `.

## Подсказки по API

```
file_read  : String -> String
args       : [String]              -- CLI аргументы после --
json_parse : String -> Result JsonValue String
json_get   : String -> JsonValue -> Result JsonValue String
unwrap     : Result a e -> a
show       : a -> String
length     : [a] -> Int
map        : (a -> b) -> [a] -> [b]
```

`JsonValue` конструкторы: `JNull`, `JBool Bool`, `JNum Int`, `JStr String`, `JArr [JsonValue]`, `JObj [Pair String JsonValue]`.

## Требования к коду

- Следуй конвенциям из AGENTS.md (File Conventions)
- Используй doc-comments (`---`) для нетривиальных функций
- Определяй функции до их первого использования
- Тест: `synoema test src/main.sno` должен проходить
