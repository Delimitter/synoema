# Synoema IP Strategy

> Internal reference document. Not legal advice.

## 1. Copyright Portfolio

- **Compiler source code** — automatic protection under Berne Convention from moment of creation
- **Language specification** — copyrightable as a literary work
- **Documentation and articles** — `docs/` directory, blog posts, tutorials
- **Website content** — synoema.dev (when launched)
- **Benchmark suites** — creative selection and arrangement

**Recommendation:** Register with U.S. Copyright Office ($65 filing fee) to enable statutory damages and attorney's fees in infringement actions.

## 2. Trademark Portfolio

### Marks

| Mark | Type | USPTO Classes | Status |
|------|------|--------------|--------|
| Synoema | Word mark | 9, 42 | Intent-to-use filing planned |
| sno | Word mark | 9, 42 | Intent-to-use filing planned |
| Logo | Design mark | 9, 42 | Pending design |

### Filing Strategy

1. **Intent-to-use (ITU)** filing with USPTO — establishes priority date
2. File `Statement of Use` within 6 months of ITU approval (extendable to 36 months)
3. **Madrid Protocol** for international registration when revenue justifies cost
4. Maintenance: must show use in commerce — document downloads, consulting, support revenue

### Classes

- **Class 9:** Computer software; programming language compilers and interpreters; downloadable software development tools
- **Class 42:** Software as a service; computer programming; software design and development

## 3. Patent Strategy

### Chosen Approach: Defensive Publication (NOT Patenting)

Patenting is expensive ($15-30K per patent) and difficult to enforce for a solo developer. Instead, we establish prior art through publication.

### Publication Plan

- **arXiv** — cs.PL cross-listed with cs.CL and cs.AI. Creates timestamped, citable prior art
- **Zenodo** — DOI provides additional independent timestamp
- **GitHub commits** — GPG-signed commits provide chain of timestamps

### Innovations to Defensively Publish

1. **BPE-aligned lexer design methodology** — engineering lexemes to align with BPE tokenizer boundaries so each operator = 1 token
2. **Intent-Bridging Annotation (IBA) system** — `@?`/`@!`/`@!!` annotations for LLM generation, erased at compile time
3. **GBNF auto-generation from typed grammar** — producing constrained decoding grammars from language specs
4. **Context-budget-aware module interface generation** — `.snm` format optimized for LLM context windows (< 2KB)

### Optional Future

Consider provisional patent ($320) before publication if external funding allows. Provisional provides 12-month priority window. Must file non-provisional within 12 months or priority lapses.

## 4. Trade Secrets (Do NOT Publish)

- Internal benchmarking methodology for LLM error rate measurement
- Specific BPE vocabulary analysis tools and detailed results
- Customer/user data if any commercial offering is launched
- Internal pricing models for commercial licenses

## 5. Licensing Enforcement

### BSL-1.1 Monitoring

- Monitor for Commercial LLM Services using `synoema-codegen` or `tools/` without a license
- "Commercial LLM Service" definition is intentionally narrow — only hosted services that use our native codegen/tooling AND serve third parties AND generate revenue
- Consider a public FAQ: "Do I need a commercial license?" with clear yes/no scenarios
- Track BSL-1.1 change dates per release version

### Apache-2.0

No enforcement needed (permissive). Ensure NOTICE file attribution is maintained.

### Trademark Monitoring

- Set up Google Alerts for "synoema"
- Monitor GitHub for repos/orgs using the name
- Monitor package registries (crates.io, npm, PyPI) for name squatting
- Reserve `synoema` on major platforms proactively

## 6. Priority Timeline

| Priority | Action | Cost | Deadline |
|----------|--------|------|----------|
| **P0** | GPG-sign all commits | Free | Immediately |
| **P0** | Add SPDX license headers to all files | Free | This week |
| **P0** | Commit all IP/legal documents | Free | This week |
| **P1** | Publish defensive disclosure on arXiv | Free | Within 1 month |
| **P1** | File intent-to-use trademark (USPTO) | ~$250-350/class | Within 1 month |
| **P1** | Register GitHub org `synoema` | Free | Within 1 week |
| **P1** | Register `synoema.dev` domain | ~$12/year | Within 1 week |
| **P2** | Reserve `crates.io/synoema` | Free | Before first release |
| **P2** | Zenodo DOI for specification | Free | With arXiv publication |
| **P2** | U.S. Copyright Office registration | $65 | Within 3 months |
| **P3** | Madrid Protocol international trademark | ~$800+ | When revenue exists |
| **P3** | IP attorney consultation | ~$500-2000 | When budget allows |

## 7. Annual Review Checklist

- [ ] Trademark registrations current? Renewals filed?
- [ ] BSL change dates tracked for each release?
- [ ] New innovations requiring defensive publication?
- [ ] Google Alerts active and reviewed?
- [ ] Domain renewals current?
- [ ] NOTICE file updated with new third-party attributions?

---

*Last updated: 2025-04-03*
