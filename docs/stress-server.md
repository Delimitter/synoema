# Stress Test Server

`lang/examples/stress_server.sno` — HTTP-сервер для интерактивного дашборда нагрузочных тестов, написанный на Synoema.

Сервер демонстрирует возможности языка: сетевое I/O, SSE-стриминг, обработка строк, tail-рекурсивный event loop — всё в ~70 строках кода.

## Запуск

```bash
cd lang/
cargo run -p synoema-repl -- run examples/stress_server.sno
```

Откройте в браузере: **http://localhost:8765/stress_tests.html**

## Маршруты

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/` | Дашборд нагрузочных тестов (HTML) |
| `GET` | `/stress_tests.html` | То же |
| `GET` | `/run/<suite>` | SSE-стрим `cargo test` для указанного crate |
| `GET` | `/run/<suite>?slow=1` | То же (параметр slow игнорируется, передаётся для совместимости) |
| `GET` | всё остальное | 404 Not Found |

Доступные значения `<suite>`: `lexer`, `parser`, `types`, `core`, `eval`, `codegen`.

## SSE-протокол

Каждый ответ на `/run/<suite>` — это поток Server-Sent Events (Content-Type: `text/event-stream`).

Каждая строка вывода `cargo test`:
```
data: {"line":"test result: ok. 51 passed; 0 failed"}

```

Завершение прогона:
```
data: {"exit":0}

```

Строки JSON-эскейпятся (`json_escape` builtin): символы `\`, `"`, `\n`, `\r`, `\t` экранируются.

## Архитектура

```
tcp_listen 8765
    │
    ▼  (tail-recursive)
server_loop
    │  tcp_accept
    ▼
handle_client          ← читает первую строку HTTP-запроса
    │  parse_path
    ▼
dispatch
    ├── /run/*    → handle_run  → fd_popen "cargo test ..." → stream_proc (SSE loop)
    ├── /          → handle_file "stress_tests.html"
    └── *          → handle_404
```

`stream_proc` — tail-рекурсивная функция, которая читает строки из дочернего процесса и отправляет SSE-события клиенту:

```synoema
stream_proc proc_fd client_fd =
  line = fd_readline proc_fd
  ? line == "" -> (fd_write client_fd sse_exit ; fd_close proc_fd ; fd_close client_fd)
               : (fd_write client_fd (sse_data line) ; stream_proc proc_fd client_fd)
```

## Встроенные I/O-функции

Сервер использует fd-based I/O API интерпретатора. Все fd — целые числа ≥ 100.

| Функция | Тип | Описание |
|---------|-----|----------|
| `tcp_listen port` | `Int -> Int` | Создаёт TCP-слушатель на 127.0.0.1:port, возвращает fd |
| `tcp_accept fd` | `Int -> Int` | Принимает входящее соединение, возвращает client fd |
| `fd_readline fd` | `Int -> String` | Читает одну строку (без `\n`), возвращает `""` при EOF |
| `fd_write fd s` | `Int -> String -> ()` | Пишет строку в fd, сбрасывает буфер |
| `fd_close fd` | `Int -> ()` | Закрывает соединение / процесс |
| `fd_popen cmd` | `String -> Int` | Запускает `sh -c cmd`, возвращает fd stdout |

`fd_popen` автоматически добавляет `$HOME/.cargo/bin` в PATH, поэтому `cargo` доступен без абсолютного пути.

## Встроенные строковые функции

| Функция | Тип | Описание |
|---------|-----|----------|
| `str_len s` | `String -> Int` | Длина строки в байтах |
| `str_slice s from to` | `String -> Int -> Int -> String` | Подстрока `[from, to)` |
| `str_find s pat from` | `String -> String -> Int -> Int` | Первое вхождение `pat` начиная с `from`, -1 если нет |
| `str_starts_with s prefix` | `String -> String -> Bool` | Проверка префикса |
| `str_trim s` | `String -> String` | Удаление пробелов по краям |
| `json_escape s` | `String -> String` | Эскейп для JSON-строки |
| `file_read path` | `String -> String` | Чтение файла целиком |

## Ограничения

- Однопоточный: каждый запрос обрабатывается до конца перед принятием следующего. SSE-стрим блокирует accept loop до завершения `cargo test`.
- Слушает только `127.0.0.1` (loopback), не доступен извне.
- Только HTTP/1.0: нет keep-alive, нет chunked transfer.

## Сравнение с Python-версией

| | Synoema (`stress_server.sno`) | Python (`stress_server.py`) |
|---|---|---|
| Строк кода | ~70 | ~80 |
| Зависимости | встроенные builtins | stdlib (`http.server`, `subprocess`) |
| Запуск | `cargo run -p synoema-repl -- run ...` | `python3 stress_server.py` |
| API | идентично | идентично |
| Типизация | статическая (HM inference) | динамическая |
