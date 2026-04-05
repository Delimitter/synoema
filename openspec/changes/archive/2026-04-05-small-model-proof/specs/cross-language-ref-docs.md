# Cross-Language Reference Documents

## Purpose

Create ICL (in-context learning) reference documents for Python and Haskell, matching Synoema's `docs/llm/synoema.md` format and token budget (~1800 tokens). This ensures fair comparison — each language gets equal context.

## Requirements

### R1: Python Reference Doc

File: `docs/llm/python.md`
- ~1800 cl100k_base tokens
- Same structure as synoema.md: overrides table, axioms, syntax examples, stdlib reference
- Focus on what a model needs to write correct Python for benchmark tasks
- Include: functions, pattern matching (match/case), list comprehensions, error handling, type hints basics

### R2: Haskell Reference Doc

File: `docs/llm/haskell.md`
- ~1800 cl100k_base tokens
- Same structure: overrides, axioms, syntax, stdlib
- Focus on: equations, pattern matching, guards, list comprehensions, ADTs, Maybe/Either, where clauses

### R3: Token Budget Verification

All three reference docs must be within 1600-2000 cl100k_base tokens.
Verify with: `python3 -c "import tiktoken; e=tiktoken.get_encoding('cl100k_base'); print(len(e.encode(open('file').read())))"`
