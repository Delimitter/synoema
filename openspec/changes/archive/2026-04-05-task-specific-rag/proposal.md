---
id: proposal
type: proposal
status: done
---

# Task-Specific RAG Templates

## Why

Static one-size-fits-all reference wastes context on irrelevant information. A task about sorting lists doesn't need the JSON or record sections. For small models (4B-32B), every token of context matters disproportionately.

Research (CodeRAG-Bench 2025, Context Engineering 2026) shows that task-aware retrieval of relevant examples + gotchas outperforms static references, especially for small models.

Goal: create a set of 5 category-specific prompt templates + a gotcha injection system that dynamically selects relevant warnings based on detected features in the task.

## What Changes

- New directory `docs/llm/templates/` with 5 category templates
- Each template: compact reference header + 1-2 category-specific examples + relevant gotchas + relevant stdlib signatures
- New `docs/llm/templates/README.md` with usage guide
- Gotcha injection map: feature → gotcha list

## Capabilities

### New Capabilities

- `llm-prompt-templates`: 5 task-category prompt templates for small model code generation
- `gotcha-injection`: Feature-to-gotcha mapping for dynamic warning injection

### Modified Capabilities

- None

## Scope

### In Scope

- 5 templates: arithmetic, lists, adt-patterns, records-maps, string-io
- Gotcha injection map (JSON or markdown)
- Usage documentation

### Out of Scope

- Embedding-based RAG (requires vector DB — future work)
- Runtime template selection (manual or by LLM orchestrator)
- Changes to compiler or language
