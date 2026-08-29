# Governance

## Decision making

syntaxis is maintained by its original author. There is no committee, no
voting, and no RFC process. Technical direction, scope, and releases are
decided by the maintainer.

Decisions are made to preserve, in priority order:

1. **Determinism.** Same input bytes, same engine version, same rule pack,
   same output bytes. Nothing ships that weakens this.
2. **Explainability.** Every derived fact names the rule and sources that
   produced it. A result that cannot be explained is a defect regardless of
   whether it is correct.
3. **Honest uncertainty.** The engine reports ambiguity and unsupported
   constructions rather than guessing. Coverage is never bought with fabricated
   confidence.
4. **Auditable provenance.** Every reference artifact has a licence, a source,
   and a checksum.
5. Long-term maintainability by one person.

Where these conflict, the higher-numbered goal yields. Notably: **accuracy
loses to determinism.** A change that improves parse quality but makes output
depend on iteration order, threading, or an unpinned resource is rejected.

## What is stable and what is not

- **Stable, changes require a version bump and a changelog entry:** the data
  model in `parser-core`, serialization field order, rule ids, message keys,
  the CoNLL-U mapping, and rule-pack format.
- **Not stable, may change without notice before 1.0:** every Rust API surface,
  crate boundaries, rule-pack content, and the CLI.

Rule ids and message keys are treated as public API even though they are
strings, because consumers key policy off them.

## Forking

Forking is explicitly fine and requires no notice. If the fork diverges,
please rename the crates so users can tell the artifacts apart, and keep the
`NOTICE` file intact.

## Succession

If the maintainer becomes unavailable for six months, the repository should be
considered unmaintained. Fork it. The licence permits this and the
architecture — versioned rule packs, checksummed artifacts, frozen fixtures —
was chosen so that a fork can verify it inherited exactly what it thinks it did.

---

**Maintainer:** Michael A. Kuykendall <michaelallenkuykendall@gmail.com>