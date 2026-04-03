# Contributing to Synoema

Thank you for your interest in contributing to Synoema! Whether you're fixing a bug, proposing a feature, improving documentation, or adding benchmarks, your contribution is welcome.

Before contributing, please read this guide and our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Bug Reports

Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) on GitHub. Include:
- Synoema version (`sno --version`)
- Operating system
- Minimal `.sno` reproduction
- Expected vs actual behavior
- Full compiler error output

### Feature Proposals (RFC)

Create a GitHub issue with the `[RFC]` prefix in the title. Include:
- Summary of the proposed feature
- Motivation and use cases
- Proposed syntax (if language feature)
- Impact on token efficiency (every feature must justify its token cost)
- Alternatives considered

### Code Contributions

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes
4. Ensure all checks pass (see Code Standards below)
5. Submit a pull request

### Other Contributions

- **Documentation** — improvements to `docs/`, tutorials, examples
- **Benchmarks** — additions to `benchmarks/`
- **Language spec** — clarifications to `spec/`
- **Examples** — new `.sno` examples in `examples/`

## Development Setup

```bash
git clone https://github.com/synoema/synoema.git
cd synoema/lang
cargo build
cargo test
```

### Running

```bash
cargo run -p synoema-repl -- run examples/quicksort.sno     # Interpreter
cargo run -p synoema-repl -- jit examples/factorial.sno      # JIT
cargo run -p synoema-repl -- eval "6 * 7"                    # Eval expression
```

## Code Standards

- `cargo fmt` before every commit — no formatting violations
- `cargo clippy` with zero warnings
- `cargo test` must pass with 0 failures, 0 warnings
- Every new feature requires tests (interpreter AND JIT where applicable)
- Compiler error messages must include source location and hint
- Every operator must be exactly 1 BPE token (cl100k_base)

## Developer Certificate of Origin (DCO)

We use the Developer Certificate of Origin (DCO) instead of a Contributor License Agreement (CLA). By contributing, you certify that you have the right to submit the work under the project's license.

**Sign off every commit:**

```bash
git commit -s -m "Add parser for let-expressions"
```

This adds a `Signed-off-by: Your Name <email>` line to the commit message. All commits in a pull request must be signed off.

**Why DCO over CLA?** DCO is lighter-weight, does not require signing a separate legal agreement, is used by the Linux kernel and many major projects, and does not transfer rights away from the contributor. It simply certifies the contributor has the right to submit, which is sufficient for our Apache-2.0/BSL-1.1 licensing model.

<details>
<summary>Full text of the Developer Certificate of Origin 1.1</summary>

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

</details>

## License Headers

All new source files must include the appropriate SPDX license header.

### Rust files in `lang/crates/` (EXCEPT `synoema-codegen/`)

```rust
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors
```

### Rust files in `lang/crates/synoema-codegen/`

```rust
// SPDX-License-Identifier: BUSL-1.1
// Copyright (c) 2025-present Andrey Bubnov
```

### Files in `tools/`

```python
# SPDX-License-Identifier: BUSL-1.1
# Copyright (c) 2025-present Andrey Bubnov
```

### Example files in `examples/`

```
-- SPDX-License-Identifier: MIT-0
```

**Note:** BSL-licensed files use the author's name in the copyright line (the licensor), not "Synoema Contributors", because BSL commercial licensing authority rests with the licensor.

You can run `scripts/add_headers.sh` to automatically add missing headers to all source files.

## Licensing Summary

| Directory | License | SPDX |
|-----------|---------|------|
| `lang/crates/` (except codegen) | Apache-2.0 | `Apache-2.0` |
| `lang/crates/synoema-codegen/` | BSL-1.1 | `BUSL-1.1` |
| `tools/` | BSL-1.1 | `BUSL-1.1` |
| `spec/` | CC-BY-SA-4.0 | `CC-BY-SA-4.0` |
| `docs/` | CC-BY-SA-4.0 | `CC-BY-SA-4.0` |
| `examples/` | MIT-0 | `MIT-0` |

## Code of Conduct

This project follows the [Contributor Covenant v2.1](CODE_OF_CONDUCT.md). Please read it before participating.

## Questions?

Open a GitHub Discussion or reach out at andbubnov@gmail.com.

---

*Thank you for helping make Synoema better!*
