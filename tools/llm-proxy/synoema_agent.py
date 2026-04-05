#!/usr/bin/env python3
"""Synoema LLM Agent Proxy — bridges small LLMs (8B-20B) to Synoema MCP tools.

Uses Ollama or OpenRouter's OpenAI-compatible function calling API with a minimal
system prompt, so even small models can evaluate, run, and typecheck Synoema code.

Usage:
    # Ollama (local)
    python3 synoema_agent.py --model qwen3:8b

    # OpenRouter (free models)
    python3 synoema_agent.py --model nvidia/nemotron-nano-9b-v2:free \\
        --base-url https://openrouter.ai/api/v1 --api-key sk-or-...

    # With full language reference (~1500 tokens)
    python3 synoema_agent.py --model qwen3:8b --full-context
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

try:
    from openai import OpenAI
except ImportError:
    print("Error: openai not installed. Run: pip install openai", file=sys.stderr)
    sys.exit(1)


# ---------------------------------------------------------------------------
# Tool schemas (OpenAI function-calling format)
# ---------------------------------------------------------------------------

TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "eval",
            "description": "Evaluate a Synoema expression. Returns value and inferred type.",
            "parameters": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "A Synoema expression, e.g. '6 * 7' or 'map (\\x -> x * 2) [1 2 3]'",
                    }
                },
                "required": ["code"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "run",
            "description": "Run a full Synoema program. Must have a 'main' binding. Returns stdout and result.",
            "parameters": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "A complete Synoema program with 'main = <expr>'",
                    }
                },
                "required": ["code"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "typecheck",
            "description": "Type-check a Synoema program. Returns type of 'main' or error details.",
            "parameters": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "A complete Synoema program with 'main = <expr>'",
                    }
                },
                "required": ["code"],
            },
        },
    },
]

KNOWN_TOOLS = {t["function"]["name"] for t in TOOLS}

# ---------------------------------------------------------------------------
# System prompts
# ---------------------------------------------------------------------------

SYSTEM_PROMPT = """\
You are a Synoema programming assistant. Synoema is a functional language.
Use the provided tools to evaluate expressions, run programs, and check types.

Key syntax rules:
- Lists use spaces, not commas: [1 2 3]
- Conditional: ? cond -> then : else
- String concat: ++ (not +)
- No return keyword -- last expression is the result
- Bindings: name args = body (no def/fn/fun keyword)
- Lambda: \\x -> x * 2
- Cons pattern needs parens: head (x:_) = x
- Pipe: x |> f  means  f x
- Comments: -- (single line)
- Entry point: main = <expr>

Example:
fib 0 = 0
fib 1 = 1
fib n = fib (n - 1) + fib (n - 2)
main = fib 10

When asked to write or evaluate Synoema code, use the tools.
Use 'eval' for expressions, 'run' for full programs, 'typecheck' for types.
If a tool returns an error, read the message and fix the code."""


def load_system_prompt(full_context: bool) -> str:
    if not full_context:
        return SYSTEM_PROMPT
    ref_path = Path(__file__).resolve().parent.parent.parent / "docs" / "llm" / "synoema.md"
    if ref_path.exists():
        ref = ref_path.read_text()
        return (
            "You are a Synoema programming assistant. "
            "Use the provided tools to evaluate, run, and typecheck Synoema code.\n"
            "If a tool returns an error, read the message and fix the code.\n\n"
            "# Synoema Language Reference\n\n" + ref
        )
    print(f"Warning: {ref_path} not found, using condensed prompt", file=sys.stderr)
    return SYSTEM_PROMPT


# ---------------------------------------------------------------------------
# MCP Client (JSON-RPC 2.0 over stdio)
# ---------------------------------------------------------------------------

class MCPClient:
    def __init__(self, binary: str, env: dict | None = None):
        self._binary = binary
        self._env = env
        self._proc = None
        self._next_id = 1

    def start(self):
        import os

        run_env = os.environ.copy()
        if self._env:
            run_env.update(self._env)

        self._proc = subprocess.Popen(
            [self._binary],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=run_env,
        )
        # Initialize handshake
        self._send("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "synoema-agent", "version": "0.1.0"},
        })
        resp = self._recv()
        if "error" in resp:
            raise RuntimeError(f"MCP initialize failed: {resp['error']}")
        # Send initialized notification (no id = no response expected)
        self._send_raw({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def call_tool(self, name: str, arguments: dict) -> tuple[str, bool]:
        self._send("tools/call", {"name": name, "arguments": arguments})
        resp = self._recv()
        if "error" in resp:
            return f"Error: {resp['error'].get('message', 'unknown')}", True
        result = resp.get("result", {})
        content = result.get("content", [])
        text = content[0]["text"] if content else "(empty)"
        is_error = result.get("isError", False)
        return text, is_error

    def close(self):
        if self._proc:
            self._proc.terminate()
            try:
                self._proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self._proc.kill()

    def _send(self, method: str, params: dict):
        msg = {
            "jsonrpc": "2.0",
            "id": self._next_id,
            "method": method,
            "params": params,
        }
        self._next_id += 1
        self._send_raw(msg)

    def _send_raw(self, msg: dict):
        line = json.dumps(msg) + "\n"
        self._proc.stdin.write(line.encode())
        self._proc.stdin.flush()

    def _recv(self) -> dict:
        line = self._proc.stdout.readline()
        if not line:
            raise RuntimeError("MCP server closed connection")
        return json.loads(line)


# ---------------------------------------------------------------------------
# Agent loop
# ---------------------------------------------------------------------------

def agent_loop(
    client: OpenAI,
    mcp: MCPClient,
    model: str,
    messages: list,
    max_turns: int = 10,
    verbose: bool = False,
) -> str:
    for turn in range(max_turns):
        try:
            response = client.chat.completions.create(
                model=model,
                messages=messages,
                tools=TOOLS,
                max_tokens=2048,
            )
        except Exception as e:
            return f"API error: {e}"

        choice = response.choices[0]
        message = choice.message

        if choice.finish_reason == "stop" or not message.tool_calls:
            return message.content or "(no response)"

        # Model wants to call tools
        messages.append(message)

        for tc in message.tool_calls:
            name = tc.function.name
            if verbose:
                print(f"  [{name}] {tc.function.arguments}", file=sys.stderr)

            if name not in KNOWN_TOOLS:
                result_text = f"Error: unknown tool '{name}'. Available: eval, run, typecheck"
                is_error = True
            else:
                try:
                    args = json.loads(tc.function.arguments)
                except json.JSONDecodeError:
                    args = None

                if args is None or "code" not in args:
                    result_text = 'Error: invalid arguments. Expected {"code": "..."}'
                    is_error = True
                else:
                    result_text, is_error = mcp.call_tool(name, args)

            if verbose:
                tag = "ERR" if is_error else "OK"
                print(f"  -> [{tag}] {result_text}", file=sys.stderr)

            messages.append({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result_text,
            })

    return "(max tool-call turns reached)"


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def find_mcp_binary() -> str | None:
    base = Path(__file__).resolve().parent.parent.parent / "mcp" / "target" / "release" / "synoema-mcp"
    if base.exists():
        return str(base)
    # Try cross-compiled path
    alt = base.parent.parent / "aarch64-apple-darwin" / "release" / "synoema-mcp"
    if alt.exists():
        return str(alt)
    return None


def main():
    parser = argparse.ArgumentParser(
        description="Synoema LLM Agent Proxy -- small models + MCP tools"
    )
    parser.add_argument("--model", default="qwen3:8b", help="Model name (default: qwen3:8b)")
    parser.add_argument(
        "--base-url",
        default="http://localhost:11434/v1",
        help="API base URL (default: Ollama local)",
    )
    parser.add_argument("--api-key", default="ollama", help="API key (default: 'ollama')")
    parser.add_argument("--mcp-binary", default=None, help="Path to synoema-mcp binary")
    parser.add_argument(
        "--full-context",
        action="store_true",
        help="Load full synoema.md reference (~1500 tokens)",
    )
    parser.add_argument("--max-turns", type=int, default=10, help="Max tool-call rounds (default: 10)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Print tool calls to stderr")
    args = parser.parse_args()

    # Find MCP binary
    mcp_path = args.mcp_binary or find_mcp_binary()
    if not mcp_path or not Path(mcp_path).exists():
        print(
            "Error: synoema-mcp binary not found.\n"
            "Build it: cd mcp && cargo build --release\n"
            "Or specify: --mcp-binary /path/to/synoema-mcp",
            file=sys.stderr,
        )
        sys.exit(1)

    # Detect SYNOEMA_ROOT
    root = Path(__file__).resolve().parent.parent.parent
    mcp_env = {"SYNOEMA_ROOT": str(root)}

    # Start MCP
    mcp = MCPClient(mcp_path, env=mcp_env)
    try:
        mcp.start()
    except Exception as e:
        print(f"Error starting MCP server: {e}", file=sys.stderr)
        sys.exit(1)

    if args.verbose:
        print(f"MCP server started: {mcp_path}", file=sys.stderr)
        print(f"Model: {args.model} @ {args.base_url}", file=sys.stderr)

    # Setup LLM client
    client = OpenAI(base_url=args.base_url, api_key=args.api_key)
    system_prompt = load_system_prompt(args.full_context)
    messages = [{"role": "system", "content": system_prompt}]

    print("Synoema Agent (type /q to quit, /clear to reset)")
    print()

    try:
        while True:
            try:
                user_input = input("> ")
            except EOFError:
                break

            stripped = user_input.strip()
            if not stripped:
                continue
            if stripped.lower() in ("/q", "/quit", "exit", "quit"):
                break
            if stripped.lower() == "/clear":
                messages = [{"role": "system", "content": system_prompt}]
                print("Context cleared.")
                continue

            messages.append({"role": "user", "content": stripped})
            response = agent_loop(
                client, mcp, args.model, messages,
                max_turns=args.max_turns, verbose=args.verbose,
            )
            messages.append({"role": "assistant", "content": response})
            print(f"\n{response}\n")

    except KeyboardInterrupt:
        print()
    finally:
        mcp.close()


if __name__ == "__main__":
    main()
