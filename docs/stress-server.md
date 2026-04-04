# Stress Test Server

`lang/examples/stress_server.sno` — an HTTP server for an interactive stress test dashboard, written in Synoema.

The server demonstrates language capabilities: network I/O, SSE streaming, string processing, tail-recursive event loop — all in ~70 lines of code.

## Running

```bash
cd lang/
cargo run -p synoema-repl -- run examples/stress_server.sno
```

Open in your browser: **http://localhost:8765/stress_tests.html**

## Routes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | Stress test dashboard (HTML) |
| `GET` | `/stress_tests.html` | Same |
| `GET` | `/run/<suite>` | SSE stream of `cargo test` for the specified crate |
| `GET` | `/run/<suite>?slow=1` | Same (slow parameter is ignored, passed for compatibility) |
| `GET` | everything else | 404 Not Found |

Available `<suite>` values: `lexer`, `parser`, `types`, `core`, `eval`, `codegen`.

## SSE Protocol

Each response to `/run/<suite>` is a Server-Sent Events stream (Content-Type: `text/event-stream`).

Each line of `cargo test` output:
```
data: {"line":"test result: ok. 51 passed; 0 failed"}

```

End of test run:
```
data: {"exit":0}

```

Lines are JSON-escaped (`json_escape` builtin): characters `\`, `"`, `\n`, `\r`, `\t` are escaped.

## Architecture

```
tcp_listen 8765
    │
    ▼  (tail-recursive)
server_loop
    │  tcp_accept
    ▼
handle_client          ← reads the first line of the HTTP request
    │  parse_path
    ▼
dispatch
    ├── /run/*    → handle_run  → fd_popen "cargo test ..." → stream_proc (SSE loop)
    ├── /          → handle_file "stress_tests.html"
    └── *          → handle_404
```

`stream_proc` is a tail-recursive function that reads lines from the child process and sends SSE events to the client:

```synoema
stream_proc proc_fd client_fd =
  line = fd_readline proc_fd
  ? line == "" -> (fd_write client_fd sse_exit ; fd_close proc_fd ; fd_close client_fd)
               : (fd_write client_fd (sse_data line) ; stream_proc proc_fd client_fd)
```

## Built-in I/O Functions

The server uses the interpreter's fd-based I/O API. All fds are integers >= 100.

| Function | Type | Description |
|----------|------|-------------|
| `tcp_listen port` | `Int -> Int` | Creates a TCP listener on 127.0.0.1:port, returns fd |
| `tcp_accept fd` | `Int -> Int` | Accepts an incoming connection, returns client fd |
| `fd_readline fd` | `Int -> String` | Reads one line (without `\n`), returns `""` at EOF |
| `fd_write fd s` | `Int -> String -> ()` | Writes a string to fd, flushes buffer |
| `fd_close fd` | `Int -> ()` | Closes the connection / process |
| `fd_popen cmd` | `String -> Int` | Runs `sh -c cmd`, returns fd of stdout |

`fd_popen` automatically adds `$HOME/.cargo/bin` to PATH, so `cargo` is available without an absolute path.

## Built-in String Functions

| Function | Type | Description |
|----------|------|-------------|
| `str_len s` | `String -> Int` | String length in bytes |
| `str_slice s from to` | `String -> Int -> Int -> String` | Substring `[from, to)` |
| `str_find s pat from` | `String -> String -> Int -> Int` | First occurrence of `pat` starting from `from`, -1 if not found |
| `str_starts_with s prefix` | `String -> String -> Bool` | Prefix check |
| `str_trim s` | `String -> String` | Trim whitespace from both ends |
| `json_escape s` | `String -> String` | Escape for JSON string |
| `file_read path` | `String -> String` | Read entire file |

## Limitations

- Single-threaded: each request is handled to completion before accepting the next one. The SSE stream blocks the accept loop until `cargo test` finishes.
- Listens on `127.0.0.1` (loopback) only, not accessible externally.
- HTTP/1.0 only: no keep-alive, no chunked transfer.

## Comparison with Python Version

| | Synoema (`stress_server.sno`) | Python (`stress_server.py`) |
|---|---|---|
| Lines of code | ~70 | ~80 |
| Dependencies | built-in builtins | stdlib (`http.server`, `subprocess`) |
| Running | `cargo run -p synoema-repl -- run ...` | `python3 stress_server.py` |
| API | identical | identical |
| Typing | static (HM inference) | dynamic |
