#!/usr/bin/env python3
"""Generate code via OpenRouter API and validate it."""

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

try:
    from openai import OpenAI
except ImportError:
    print("Error: openai not installed. Run: pip install openai", file=sys.stderr)
    sys.exit(1)

try:
    import tiktoken
except ImportError:
    print("Error: tiktoken not installed. Run: pip install tiktoken", file=sys.stderr)
    sys.exit(1)

LANG_PROMPTS = {
    "synoema": "Synoema",
    "python": "Python",
    "javascript": "JavaScript",
    "typescript": "TypeScript",
    "cpp": "C++",
}

# Task descriptions — loaded from task_dir/prompt.txt or generated from task name
def get_task_prompt(task_dir: Path) -> str:
    prompt_file = task_dir / "prompt.txt"
    if prompt_file.exists():
        return prompt_file.read_text().strip()
    # Fallback: derive from directory name
    name = task_dir.name.replace("_", " ")
    return f"Write a complete, runnable program that implements {name}. Include a main entry point that demonstrates the solution with example input and prints the result."


def validate_syntax(code: str, language: str, task_dir: Path) -> bool:
    """Check if generated code parses/compiles without errors."""
    try:
        with tempfile.NamedTemporaryFile(mode='w', suffix=get_ext(language), delete=False) as f:
            f.write(code)
            f.flush()
            path = f.name

        if language == "synoema":
            # Find synoema binary
            lang_dir = task_dir.parent.parent.parent / "lang"
            result = subprocess.run(
                ["cargo", "run", "--quiet", "--manifest-path", str(lang_dir / "Cargo.toml"),
                 "-p", "synoema-repl", "--", "run", path],
                capture_output=True, timeout=30
            )
            return result.returncode == 0
        elif language == "python":
            result = subprocess.run(
                ["python3", "-c", f"compile(open('{path}').read(), '{path}', 'exec')"],
                capture_output=True, timeout=10
            )
            return result.returncode == 0
        elif language == "javascript":
            result = subprocess.run(
                ["node", "--check", path],
                capture_output=True, timeout=10
            )
            return result.returncode == 0
        elif language == "typescript":
            bench_dir = task_dir.parent.parent  # benchmarks/
            result = subprocess.run(
                ["npx", "tsc", "--noEmit", "--allowJs", path],
                capture_output=True, timeout=30, cwd=bench_dir
            )
            return result.returncode == 0
        elif language == "cpp":
            result = subprocess.run(
                ["g++", "-fsyntax-only", path],
                capture_output=True, timeout=15
            )
            return result.returncode == 0
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return False
    finally:
        Path(path).unlink(missing_ok=True)
    return False


def validate_correctness(code: str, language: str, expected: str, task_dir: Path) -> bool:
    """Run the code and compare output to expected."""
    if not expected.strip():
        return False  # No expected output to compare against
    try:
        with tempfile.NamedTemporaryFile(mode='w', suffix=get_ext(language), delete=False) as f:
            f.write(code)
            f.flush()
            path = f.name

        if language == "synoema":
            lang_dir = task_dir.parent.parent.parent / "lang"
            result = subprocess.run(
                ["cargo", "run", "--quiet", "--manifest-path", str(lang_dir / "Cargo.toml"),
                 "-p", "synoema-repl", "--", "jit", path],
                capture_output=True, text=True, timeout=30
            )
        elif language == "python":
            result = subprocess.run(["python3", path], capture_output=True, text=True, timeout=15)
        elif language == "javascript":
            result = subprocess.run(["node", path], capture_output=True, text=True, timeout=15)
        elif language == "typescript":
            bench_dir = task_dir.parent.parent  # benchmarks/
            result = subprocess.run(["npx", "tsx", path], capture_output=True, text=True, timeout=30, cwd=bench_dir)
        elif language == "cpp":
            out_bin = f"/tmp/bench_validate_{Path(path).stem}"
            comp = subprocess.run(["g++", "-O2", "-o", out_bin, path], capture_output=True, timeout=15)
            if comp.returncode != 0:
                return False
            result = subprocess.run([out_bin], capture_output=True, text=True, timeout=15)
            Path(out_bin).unlink(missing_ok=True)
        else:
            return False

        actual = result.stdout.strip()
        return actual == expected.strip()
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return False
    finally:
        Path(path).unlink(missing_ok=True)


def get_ext(language: str) -> str:
    return {
        "synoema": ".sno",
        "python": ".py",
        "javascript": ".js",
        "typescript": ".ts",
        "cpp": ".cpp",
    }.get(language, ".txt")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--language", required=True)
    parser.add_argument("--task-dir", required=True, type=Path)
    parser.add_argument("--key", required=True)
    parser.add_argument("--context", type=Path, help="Synoema language reference file")
    parser.add_argument("--expected", type=Path, help="Expected output file")
    parser.add_argument("--base-url", default="https://openrouter.ai/api/v1",
                        help="API base URL (default: OpenRouter; use http://localhost:11434/v1 for ollama)")
    parser.add_argument("--save-dir", type=Path, help="Directory to save generated code for post-mortem")
    parser.add_argument("--attempt", type=int, default=0, help="Attempt number (for save filenames)")
    args = parser.parse_args()

    client = OpenAI(
        base_url=args.base_url,
        api_key=args.key,
    )

    enc = tiktoken.get_encoding("cl100k_base")
    task_prompt = get_task_prompt(args.task_dir)
    lang_name = LANG_PROMPTS.get(args.language, args.language)

    system_msg = f"You are a code generator. Output ONLY valid {lang_name} code. No explanations, no markdown, no comments unless required by the language."

    if args.language == "synoema" and args.context and args.context.exists():
        context = args.context.read_text()
        system_msg += f"\n\nLanguage reference:\n{context}"

    user_msg = f"Language: {lang_name}\n\n{task_prompt}"

    tokens_in = len(enc.encode(system_msg + user_msg))

    try:
        response = client.chat.completions.create(
            model=args.model,
            messages=[
                {"role": "system", "content": system_msg},
                {"role": "user", "content": user_msg},
            ],
            temperature=0.2,
            max_tokens=2048,
        )
        code = response.choices[0].message.content.strip()

        # Strip markdown code fences if present
        if code.startswith("```"):
            lines = code.split("\n")
            # Remove first and last fence lines
            if lines[-1].strip() == "```":
                lines = lines[1:-1]
            else:
                lines = lines[1:]
            code = "\n".join(lines)

    except Exception as e:
        print(json.dumps({
            "syntax_ok": False,
            "correct": False,
            "tokens_in": tokens_in,
            "tokens_out": 0,
            "code": "",
            "error": str(e),
        }))
        return

    tokens_out = len(enc.encode(code))

    # Save generated code for post-mortem analysis
    if args.save_dir and code:
        task_name = args.task_dir.name
        model_slug = args.model.replace("/", "_")
        save_path = args.save_dir / model_slug / task_name
        save_path.mkdir(parents=True, exist_ok=True)
        ext = get_ext(args.language)
        filename = f"{args.language}_{args.attempt}{ext}"
        (save_path / filename).write_text(code)

    syntax_ok = validate_syntax(code, args.language, args.task_dir)

    expected = ""
    if args.expected and args.expected.exists():
        expected = args.expected.read_text()
    correct = validate_correctness(code, args.language, expected, args.task_dir) if syntax_ok else False

    print(json.dumps({
        "syntax_ok": syntax_ok,
        "correct": correct,
        "tokens_in": tokens_in,
        "tokens_out": tokens_out,
        "code": code,
    }))


if __name__ == "__main__":
    main()
