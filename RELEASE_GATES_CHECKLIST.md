# Release gates

Every box below is checked manually before a tagged release. `make gate` runs
the automated subset. A release does not ship on a red gate — including a red
gate whose failure "is only a fixture".

## Gate 1 — Build and hygiene

- [ ] `cargo build --offline --release` clean
- [ ] `cargo test --offline` — all tests pass, none `#[ignore]`d without a
      written reason in the test body
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `cargo deny check` clean
- [ ] Dependency count is still **zero** (`cargo tree --depth 1`)
- [ ] Builds on Linux, macOS, and Windows
- [ ] `wasm32-unknown-unknown` target still builds

## Gate 2 — Determinism

- [ ] Serialized output byte-identical across repeated runs
- [ ] Byte-identical across platforms for the fixture corpus
- [ ] No `HashMap`/`HashSet` in any path reaching output
      (`grep -rn "HashMap\|HashSet" crates/*/src` and justify each hit)
- [ ] No clock, RNG, env read, network, or filesystem access in the analysis
  path
- [ ] Digest of every fixture matches the recorded value, or the change is in
  the changelog as breaking

## Gate 3 — Structural validity

- [ ] 100% of fixture spans valid against their source text
- [ ] 100% token-order validity, no overlaps
- [ ] Every sentence is a valid tree over accepted arcs, or explicitly marked
      alternative/unsupported — never silently neither
- [ ] `Analysis::validate` returns empty for every fixture
- [ ] Tokenizer reconstruction invariant holds across the whole corpus

## Gate 4 — Honesty

This gate has no automated substitute. Read the diff.

- [ ] Every rule that fires is declared in the manifest
- [ ] Every declared rule states its supported constructions and blind spots
- [ ] No blind-spot entry was deleted rather than corrected this cycle
- [ ] No relation outside the supported set was mapped onto a supported one
- [ ] No diagnostic claims more certainty than its supports allow
- [ ] Every `SupportSet` added this cycle lists all sources materially consumed
- [ ] No accuracy, coverage, or performance number appears in docs or release
    notes that was not produced by a run recorded in this repository

## Gate 5 — Provenance

- [ ] Every artifact in `resources/` matches its manifest checksum
- [ ] Every artifact has source, licence, normalization policy, generation
    command, and fixture coverage in `docs/RESOURCES.md`
- [ ] No LanguageTool-, nlprule-, or Harper-derived content entered the tree
- [ ] Corpus components each carry explicit licence terms
- [ ] `NOTICE` still accurately describes what ships

## Gate 6 — Release mechanics

- [ ] Version bumped in `Cargo.toml`; rule-pack and mapping versions bumped if
    their content changed
- [ ] `CHANGELOG.md` entry names which of the three versions moved
- [ ] Breaking output changes flagged as breaking, even with no API change
- [ ] `README.md` "Not done yet" section still true
- [ ] `ROADMAP.md` milestone boxes reflect reality
- [ ] `cargo publish --dry-run` clean for each crate, in dependency order

## Gate 7 — Claims

- [ ] The README describes what the code does today, not what the milestone
    intends
- [ ] Anything unimplemented is listed as unimplemented, not omitted
- [ ] No milestone is described as passing its gate unless it measurably did