---
name: Feature Request / RFC
about: Propose a new language feature or compiler improvement
labels: enhancement
---

## Summary

One-paragraph description of the proposed feature.

## Motivation

What problem does this solve? Who benefits?

## Proposed Syntax (if language feature)

```sno
-- Show how the feature would look in Synoema code
```

## Token Efficiency Impact

Every language feature must justify its token cost. How does this feature affect:
- Number of BPE tokens per typical usage?
- Operator count (must each be 1 BPE token)?
- Context window budget for `.snm` module interfaces?

## Alternatives Considered

What other approaches did you consider and why did you reject them?

## Implementation Notes (optional)

Any thoughts on how this could be implemented (interpreter, JIT, or both).
