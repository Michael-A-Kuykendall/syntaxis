Syntaxis is a standalone Rust crate implementing a deterministic English
structural-analysis engine. Internal modules are `parser_core` (data model,
spans, supports, fact graph, canonical JSON, SHA-256), `english_rules` (rule
pack loading, segmentation, tokenization, POS, parsing, grammar), and `conllu`
(strict CoNLL-U import/export and Penn<->UD mapping). The native CLI is
`src/bin/syntaxis.rs`; the optional browser ABI is `src/wasm.rs`. Core
invariants: one Analysis snapshot per document; byte-stable canonical
serialization; provenance on every fact; transitive support retraction;
uncertainty cannot be overstated. Read `mem:tech_stack` for toolchain and
`mem:task_completion` for gates.
