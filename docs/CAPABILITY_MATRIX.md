# Syntaxis Capability Matrix

This is the product contract for the current release-shaped build. A capability
is advertised only when its implementation and evidence are named here. This
matrix describes a deterministic structural-analysis engine, not a complete
English grammar or a general proofreading product.

## Product Contract

Syntaxis accepts bounded English text or strict CoNLL-U and returns one
versioned `Analysis` snapshot. The snapshot is available as canonical JSON and
can be hashed, validated, exported, or consumed through the Rust and optional
Wasm APIs. Analysis facts carry their derivation and material supports.

Primary consumers are Rust applications, browser/Wasm applications, and data
pipelines that need reproducible and inspectable English structure.

## Supported Capabilities

| Capability | Status | Evidence |
| --- | --- | --- |
| Deterministic text analysis | Supported for bounded input | `src/english_rules/pipeline.rs`; `src/english_rules/evaluation.rs` |
| Sentence segmentation | Supported construction slice | `src/english_rules/segment.rs`; manifest `SEG.*` rules |
| Tokenization and spans | Supported construction slice | `src/english_rules/tokenize.rs`; pipeline invariant tests |
| Deterministic POS and morphology | Supported lexical/suffix inventory | `src/english_rules/pos.rs`; manifest `POS.*` rules |
| Bounded dependency attachment | Supported declared UD relations | `src/english_rules/parser.rs`; manifest `ATTACH.*` rules |
| Explicit unsupported structures | Supported contract | `Relation::Unsupported`; `ATTACH.UNSUPPORTED.V1` |
| Agreement diagnostics | Supported first grammar slice | `src/english_rules/grammar.rs`; evaluation gate |
| Determiner diagnostics | Supported `a/an` plus plural noun case | `GRAMMAR.DETERMINER.ARTICLE_NUMBER.V1` |
| Verb-form diagnostics | Supported bounded `have` chain case | `GRAMMAR.VERB_FORM.HAVE_PARTICIPLE.V1` |
| Negation-placement diagnostics | Supported hostless simple-negation case | `GRAMMAR.PLACEMENT.NEGATION.V1` |
| Coordination diagnostics | Supported when a resolved `conj` arc exists | `GRAMMAR.AGREEMENT.COORDINATION.V1` |
| Provenance and support tracking | Supported on derived facts | `src/parser_core/support.rs`; integration tests |
| Retraction cascade | Supported for dependent facts | `src/parser_core/factgraph.rs`; pipeline tests |
| Certainty enforcement | Supported for diagnostic insertion | `src/parser_core/analysis.rs`; `tests/certainty.rs` |
| Canonical JSON | Stable versioned output contract | `src/parser_core/serialize.rs`; digest tests |
| SHA-256 digest | Stable for bounded canonical payloads | `src/parser_core/hash.rs`; FIPS tests |
| Strict CoNLL-U import/export | Supported with explicit unsupported mapping | `src/conllu/`; `tests/conllu_roundtrip.rs` |
| Rust consumer API | Supported | `src/api.rs`; rustdoc and API tests |
| JavaScript/Wasm consumer API | Supported with optional `wasm` feature | `src/wasm.rs`; `wasm-pack test --node` |
| 500-case structural evaluation gate | Supported as a construction gate | `src/english_rules/evaluation.rs` |

## Explicit Boundaries

| Boundary | Current behavior |
| --- | --- |
| Unknown open-class words | Retained as explicit unknown analyses; not guessed |
| Ambiguous or unresolved structure | Preserved as unsupported/alternative; not silently accepted |
| Complete English parsing | Not supported |
| General proofreading or correction | Not supported |
| Full Universal Dependencies coverage | Not supported |
| Broad natural-language accuracy | Not established by the 500-case gate |
| Statistical or neural inference | Not supported |
| Multilingual analysis | Not supported |
| Browser filesystem or CLI flags | Not part of the Wasm API |
| Unbounded document processing | Not supported; consumer API input is limited to 128 KiB |
| CoNLL-U import as grammar inference | Not supported; import preserves supplied annotations |

## Evidence Rules

Every new advertised capability must identify its implementation, manifest or
model contract where applicable, positive and clean fixtures, nearest blind
spots, and a runnable verification gate. A passing synthetic case does not
establish broad language coverage. Claims must remain at the narrower boundary
demonstrated by the evidence.

## Expansion Direction

The next useful grammar expansions are clauses, auxiliaries, coordination,
subordination, questions, passive and copular forms, negation, and
prepositional attachment. They should be prioritized by consumer value and
implemented one construction at a time with independent fixtures and explicit
unsupported behavior.
