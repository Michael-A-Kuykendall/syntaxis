# Syntaxis

![Syntlogo](https://raw.githubusercontent.com/Michael-A-Kuykendall/syntaxis/main/assets/syntaxis--logo.png)

![Trans rights](https://pride-badges.pony.workers.dev/static/v1?label=trans%20rights&stripeWidth=6&stripeColors=5BCEFA,F5A9B8,FFFFFF,F5A9B8,5BCEFA)
![LGBTQ+ friendly](https://pride-badges.pony.workers.dev/static/v1?label=lgbtq%2B%20friendly&stripeWidth=6&stripeColors=E40303,FF8C00,FFED00,008026,24408E,732982)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/Michael-A-Kuykendall/syntaxis/workflows/CI/badge.svg)](https://github.com/Michael-A-Kuykendall/syntaxis/actions)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/syntaxis?style=social)](https://github.com/Michael-A-Kuykendall/syntaxis/stargazers)

Deterministic, offline English structural analysis. Rust, **zero dependencies** —
no model weights, no Python, no C++, no network, no wall-clock, no RNG.

This repository currently implements the M0 substrate plus the first M1/M2
structural slices from the `wv4` proposal: deterministic lexical
analysis, bounded dependency attachment, and provenance-backed grammar
diagnostics. It is deliberately not a complete English parser or proofreader.

```
cargo test                          # 79 tests, no network needed
cargo run -p engine-cli --example demo # live tokenization / import / retraction
cargo run -p engine-cli -- "The cat are sleeping." # canonical JSON analysis
```

## Layout

| crate | contains |
| --- | --- |
| `parser-core` | data model, spans, supports, fact graph, canonical JSON, SHA-256 |
| `english-rules` | rule-pack loader, normalization, segmentation, tokenization, M0 pipeline |
| `conllu` | strict UD import/export and the versioned Penn↔UD projection |
| `engine-cli` | command-line front end (**stub**; see Not done yet) |

`resources/en/` holds the versioned, checksummed reference artifacts.
`fixtures/` holds hand-annotated gold data.

The `syntaxis` binary accepts text and emits canonical JSON by default; use
`--digest`, `--validate`, `--conllu-in FILE`, `--conllu-out`, or
`--retract-token ID` for machine-readable lifecycle operations. Use
`--evaluation` to run the original 500-case structural gate; that gate is a
construction-focused baseline, not a claim about natural-language coverage.

## What M0 actually delivers

**One snapshot per document (§4.1).** `analyze()` returns a single `Analysis`.
POS, dependency, and grammar rules attach to this object and consume these
token identities; none re-tokenize.

**Determinism as a tested property (§4.2).** Serialization is canonical JSON
with declared member order, no floats, no timestamps. `Analysis::digest()` is a
SHA-256 over the compact form. Two runs over the same bytes produce identical
output, and a test asserts it.

**Provenance on every fact (§4.5).** Each token, sentence, arc, and diagnostic
carries a `SupportSet`: rule id, rule-pack id, derivation kind, and every source
it consumed. Sources are sorted and deduplicated so discovery order cannot leak
into the output.

**Retraction that cascades (§8).** `FactGraph` maintains a reverse support
index. Retracting a source removes exactly its transitive dependents, in sorted
order, with no caller-side cleanup. Cycles terminate; siblings survive. Live
example: retracting the analysis of `cat` in *The cat are sleeping.* removes the
two arcs resting on it and nothing else.

**Uncertainty as data (§4.4, §7).** `ArcStatus` and `Resolution` are modelled,
and `Analysis::certainty_for` propagates uncertainty from arcs to anything
derived from them. `validate()` reports an `OverconfidentDiagnostic` when a
diagnostic claims more certainty than its supports allow — the "fabricated
certainty" failure is a validation error, not a code review question.

**Rule packs are checksummed (§9).** The manifest declares versions, licences,
provenance, and SHA-256 for every artifact; the loader verifies embedded content
against it and refuses to start on mismatch. Every rule declares its supported
constructions, its blind spots, and its precision target — currently the honest
string "not yet measured; set after the frozen-corpus baseline", per §11.

**Strict CoNLL-U (§3).** Multi-word tokens and empty nodes are rejected, not
flattened. Every error carries a line number. A declared `# text` that disagrees
with its tokens is an error. The bundled fixture round-trips byte-for-byte.

Two tokenizer invariants are tested over every input: concatenated surfaces
reproduce the input with whitespace removed, and every span slices back to
exactly its surface.

## Findings for the cloud review

Two of the review questions have answers that fell out of building the fixtures:

1. **The frozen relation set cannot express existential `there`.** §5 lists no
   `expl`, so *There is many reasons.* imports with `There` as
   `Relation::Unsupported` carrying the raw label — correct behaviour, but the
   construction is one of the three motivating cases, and §6.4 rule 2 needs it.
   Recommend adding `expl` before M1.
2. **The relation set is now native UD.** `case`/`nmod` replace the earlier
   Stanford-style `prep`/`pobj`, and `expl` is supported for existential
   `there`. Legacy labels remain explicitly unsupported rather than silently
   converted.

## Support

This project is a safe space. Trans rights are human rights.

If you or someone you love needs support:

- [The Trevor Project](https://www.thetrevorproject.org/) — 24/7 for LGBTQ+ young people. Call 1-866-488-7386 or text START to 678-678
- [Trans Lifeline](https://translifeline.org/) — peer support run by and for trans people. US: 877-565-8860
- [988 Suicide & Crisis Lifeline](https://988lifeline.org/) — call or text 988

## Not done yet

M0 scope ends here. The following are deliberately absent, not overlooked:

- **Broader POS and morphology coverage.** The first closed-world lexical and
  suffix rules are implemented; unknown open-class words remain explicit.
- **Complete dependency parsing.** The current parser is bounded and emits
  explicit unsupported arcs outside its declared construction set. (M1)
- **Remaining grammar coverage.** Agreement, determiner, verb-form, and
  negation-placement diagnostics exist; broader clause/complement and
  ambiguity handling remain. (M2)
- **The CLI.** `crates/engine-cli/src/main.rs` is an empty stub; use
  `cargo run --example demo` meanwhile.
- **The 500-sentence corpus and the evaluation gate.** No thresholds are
  proposed yet, per §11.
- **NFC/NFD normalization**, which needs either a dependency or a large embedded
  table. Recorded as a known gap.
