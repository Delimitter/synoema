#!/usr/bin/env python3
"""
Synoema stress test dashboard server.

Usage:
    cd lang/
    python3 stress_server.py
    open http://localhost:8765/stress_tests.html
"""
import http.server
import subprocess
import os
import json

PORT = 8765
CARGO = os.path.expanduser("~/.cargo/bin/cargo")
WORK_DIR = os.path.dirname(os.path.abspath(__file__))

SUITES = {
    "lexer":  ["-p", "synoema-lexer"],
    "types":  ["-p", "synoema-types"],
    "eval":   ["-p", "synoema-eval"],
    "jit":    ["-p", "synoema-codegen"],
    "all":    ["-p", "synoema-lexer", "-p", "synoema-types",
               "-p", "synoema-eval", "-p", "synoema-codegen"],
}


class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        # /run/<suite>?slow=1
        if self.path.startswith("/run/"):
            parts = self.path[5:].split("?", 1)
            suite = parts[0]
            slow = len(parts) > 1 and "slow=1" in parts[1]
            self._stream_run(suite, slow)
            return
        http.server.SimpleHTTPRequestHandler.do_GET(self)

    def _stream_run(self, suite, slow):
        if suite not in SUITES:
            self.send_error(404, f"Unknown suite: {suite}")
            return

        cmd = [CARGO, "test", "--test", "stress"] + SUITES[suite] + [
            "--", "--nocapture", "--test-threads=1",
        ]
        if slow:
            cmd.append("--include-ignored")

        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream; charset=utf-8")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()

        def send(payload: dict):
            line = "data: " + json.dumps(payload, ensure_ascii=False) + "\n\n"
            self.wfile.write(line.encode("utf-8"))
            self.wfile.flush()

        try:
            proc = subprocess.Popen(
                cmd, cwd=WORK_DIR,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                text=True, bufsize=1, errors="replace",
            )
            for raw in proc.stdout:
                send({"line": raw.rstrip("\n")})
            proc.wait()
            send({"exit": proc.returncode})
        except BrokenPipeError:
            pass
        except Exception as exc:
            try:
                send({"error": str(exc)})
            except Exception:
                pass

    def log_message(self, fmt, *args):
        pass  # silence access log


if __name__ == "__main__":
    os.chdir(WORK_DIR)
    server = http.server.HTTPServer(("localhost", PORT), Handler)
    print(f"Synoema stress dashboard → http://localhost:{PORT}/stress_tests.html")
    print("Ctrl-C to stop.")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopped.")
