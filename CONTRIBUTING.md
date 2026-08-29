# Contributing to syntaxis

## Open source, not open contribution

syntaxis is **open source** but **not open contribution**.

- The code is freely available under the terms in [LICENSE](LICENSE).
- Fork it, modify it, ship it, learn from it. No permission needed.
- **Unsolicited pull requests are closed without merge.**
- Architecture, roadmap, rule-pack content, and merge decisions rest with the
  maintainer.

This is the SQLite model, and it is deliberate. This project's entire value
proposition is that its output is deterministic, its rules are declared, and
its provenance is auditable. Every one of those properties is a global
invariant — a well-meaning patch that adds a `HashMap` iteration to a
serialization path, or a rule with an undeclared blind spot, silently breaks a
guarantee that downstream consumers are relying on. Reviewing for that class of
defect costs more than writing the change.

Closing your PR is not a judgment of the work. It is how the project operates.

## If you want to contribute anyway

1. Open an issue, or email the maintainer, **before** writing code.
2. Describe the change and which invariant it touches.
3. If there is alignment, scope gets agreed in writing first.
4. Only then will a PR be reviewed.

## Genuinely welcome, no permission needed

- **Bug reports.** Especially reproducible ones.
- **Linguistic defect reports** — a sentence that tokenizes, segments, or
  attaches wrongly. These are the most valuable thing an outsider can send.
  One sentence, expected output, actual output. That is a complete report.
- **Security reports.** See [SECURITY.md](SECURITY.md). Do not open a public
  issue.
- **Questions about the design.** Discussions, not PRs.

## Handled internally, do not send patches for

- The data model in `parser-core` and anything touching serialization order.
- Rule-pack content: rules, lexicons, and their declared blind spots.
- Dependencies. See the next section.
- Corpus and fixture additions. See the section after that.
- Evaluation thresholds.

## The dependency rule

**This workspace has zero external dependencies and that is a feature, not an
accident.** Determinism is unverifiable if it depends on transitive crates the
project does not control.

A PR adding a dependency will be closed regardless of merit. If you believe one
is unavoidable, open an issue and make the case: what it does, why it cannot be
vendored in under 300 lines, its licence, its transitive tree, and its
maintenance status. `cargo deny check` runs in CI and will reject copyleft and
unknown licences outright.

## The linguistic resource rule

Reference data — lexicons, rule tables, corpora — is where this project is most
exposed legally, and it is the reason the architecture exists in its current
form. Read [docs/RESOURCES.md](docs/RESOURCES.md) before proposing any.

Non-negotiable:

- **No LanguageTool-derived content.** That includes nlprule's binary resource
  bundles, which carry LGPL-2.1 obligations. It does not matter how convenient
  it is.
- **No Harper-derived content** without a separate written licence review.
- **No scraped corpora** without explicit licence terms for each component.
- Every artifact must ship with source, licence, normalization policy,
  generation command, checksum, and fixture coverage. An artifact without
  provenance is not mergeable at any quality level.

## What a good bug report contains

- The input text, exactly, in a code block. Copy-paste it; do not retype it —
  a curly apostrophe versus an ASCII one is frequently the whole bug.
- Expected output and actual output.
- Engine version, rule-pack version (both appear in serialized output), and
  `rustc --version`.
- For non-determinism reports: two `Analysis::digest()` values and what
  differed between the runs.

## Recognition

Reporters are credited in [CHANGELOG.md](CHANGELOG.md) for the release that
fixes their report. If a discussion leads to merged work, attribution is given
in the commit and the changelog.

---

**Maintainer:** <YOUR_NAME> <<YOUR_EMAIL>>