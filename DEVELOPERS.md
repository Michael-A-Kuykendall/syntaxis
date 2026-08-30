# Developer guide

## Requirements

Rust 1.75 or newer is required for native development. Browser/Wasm packaging
also requires the wasm32 target and `wasm-pack`; no C++ compiler, Python, or
protoc is required.

```
cargo build --offline
cargo test --offline
```

## Layout

```
src/parser_core/        data model, spans, supports, fact graph, JSON, SHA-256
src/english_rules/      rule pack, normalization, segmentation, tokenization
src/conllu/             strict UD import/export, Penn<->UD projection
src/bin/syntaxis.rs     command line front end
resources/en/           versioned, checksummed reference artifacts
fixtures/               hand-annotated gold data
docs/                   design notes, resource provenance, gate criteria
```

`parser_core` contains **no English knowledge**. If you find yourself adding a
word list or a language-specific rule to it, it belongs in `english_rules`.

## Common tasks

```
cargo test --offline                       # everything
cargo test parser_core --offline            # core module tests
cargo run --example demo                   # live tokenize/import/retract demo
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo deny check                           # licences and bans (cargo-deny 0.18.5)
make gate                                  # the full pre-release gate
```

## Adding a tokenizer or segmentation rule

1. Declare it in `resources/en/rulepack.manifest` first — `stage`,
   `description`, `supports`, `blind_spots`, `precision_target`. `RulePack::rule`
   returns an error for undeclared rule ids, so the manifest entry is not
   optional paperwork.
2. Implement it, emitting the rule id in the `SupportSet`.
3. Add a test for the construction it handles **and** a test pinning the
   behaviour on its nearest blind spot, so the limitation is documented
   executably rather than only in prose.
4. If it consults a reference artifact, add a `SourceRef::Lexicon` to the
   supports so retracting that entry retracts the token.

## Adding or changing a reference artifact

1. Edit the file in `resources/en/`.
2. Recompute the checksum and update the manifest:
   ```
   sha256sum resources/en/*.txt
   ```
3. Add or update its entry in `docs/RESOURCES.md`: source, licence,
   normalization policy, generation command, checksum, fixture coverage.
4. Bump the artifact version, and the rule-pack version if behaviour changed.
5. `cargo test` — the loader verifies checksums and will fail loudly.

## Changing the data model

This is the expensive one. Changing a struct in `parser_core::model` or the
field order in `parser_core::serialize` changes every consumer's bytes.

1. Bump `ANALYSIS_VERSION`.
2. Update `serialize.rs` — field order there is the contract, not an
   implementation detail.
3. Regenerate any stored digests in fixtures.
4. Note it in `CHANGELOG.md` under a breaking-change heading.

## Debugging determinism

If `serialization_is_byte_stable_across_runs` fails:

```
cargo test --offline 2>&1 | grep digest
```

Then diff the canonical JSON of two runs directly. The cause is almost always
one of: a `HashMap` reached a serialization path, a `SupportSet` was built
without `canonicalize`, or an id was allocated from a counter that depends on
visit order rather than from text position.
