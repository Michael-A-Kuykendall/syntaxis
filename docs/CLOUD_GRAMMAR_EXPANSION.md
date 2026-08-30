# Cloud Task: Expand the First Grammar Slice

This document is a self-contained coding task. It is intended for a cloud
coding agent that can read and modify this Git repository but cannot access
local Beads databases, prior conversations, local generated artifacts, or
maintainer-only state.

## Objective

Extend Syntaxis beyond its first agreement/determiner/verb-form grammar slice
without turning it into an opaque general-purpose proofreader. Add safe,
deterministic diagnostics for:

1. Subject/finite-verb person agreement when both persons are known.
2. Basic negation with do-support where the existing dependency structure can
   establish that `do` is the auxiliary host.
3. A narrowly defined basic complement or infinitive compatibility case only if
   the existing relation vocabulary and parser can represent its evidence.
4. Coordination agreement only where a resolved `conj` relation supplies the
   required evidence; leave ambiguous coordination explicit and unclaimed.

Do not fabricate a relation, guess unknown morphology, add a statistical model,
or broaden the rule to make a fixture pass.

## Repository Contract

The relevant implementation is in:

- `src/english_rules/grammar.rs`: grammar diagnostics over one `Analysis`.
- `src/english_rules/parser.rs`: bounded dependency attachment.
- `src/english_rules/pos.rs`: deterministic lexical and suffix morphology.
- `src/parser_core/model.rs`: relations, morphology, diagnostics, and spans.
- `resources/en/rulepack.manifest`: declared rule and blind-spot contracts.
- `src/english_rules/evaluation.rs`: frozen construction-focused evaluation.
- `tests/`: integration and serialization regression coverage.

`parser_core` must remain language-independent. Grammar behavior belongs in
`english_rules`. The analysis path must remain offline, deterministic, and free
of filesystem, network, clock, environment, randomness, and output-path hash
iteration.

## Required Implementation

### Person Agreement

- Use only known `Person` values on the subject and finite head.
- Emit no diagnostic when either side is unknown.
- Use a declared rule id such as
  `GRAMMAR.AGREEMENT.SUBJECT_VERB_PERSON.V1`.
- Use message key `GRAMMAR.AGREEMENT.SUBJECT_VERB_PERSON`.
- Include sentence, subject/finite token-analysis, and subject-arc supports.
- Add positive and clean fixtures, including a mismatch and an unknown-person
  case.

### Do-Support Negation

- Inspect the actual resolved arcs before emitting a placement diagnostic.
- Do not flag a clause merely because it contains `not` or `never`.
- If `do` is attached as an accepted auxiliary host, the placement rule must
  not report the clause as hostless.
- If the parser cannot establish this safely, retain an explicit unsupported
  or unresolved structure and document the blind spot instead of guessing.

### Complement or Infinitive Compatibility

- Select one small construction that the existing token and relation model can
  represent without adding an unreviewed relation.
- Define the exact supported forms before implementation.
- Require known morphology and accepted structural evidence.
- Add a clean case, an error case, and a nearest unsupported case.
- Stop and report an architecture decision if the construction requires new
  serialized fields, relation semantics, or a new public version.

### Coordination

- Preserve the current coordination diagnostic behavior.
- Add coverage for a resolved nominal coordination mismatch and a clean pair.
- Add a limitation fixture where coordination scope or shared arguments are
  ambiguous; it must remain unsupported or produce no unsupported claim.

## Manifest Contract

Add every new rule id to `resources/en/rulepack.manifest` before using it in
code. Each entry must include:

```text
stage = grammar
description = precise behavior being implemented
supports = evidence and construction boundary
blind_spots = nearest unsupported or ambiguous forms
precision_target = not yet measured; set after the frozen-corpus baseline
```

Do not delete or soften an existing blind spot to make a new test pass. If a
rule changes serialized output or rule-pack behavior, identify the required
engine/rule-pack/mapping version change in the changelog.

## Test Requirements

Add executable tests for each implemented construction:

- Positive diagnostic with exact diagnostic kind and message key.
- Clean input with no diagnostic of that kind.
- Nearest blind spot remaining unsupported, unresolved, or clean as specified.
- Provenance containing every material token-analysis and arc source.
- Retraction removing a diagnostic when its supporting arc is retracted.
- Repeatability of canonical JSON and digest.

Add cases to `src/english_rules/evaluation.rs` only with independently written
expected diagnostic kinds. Do not derive expected results from the code under
test. Keep the existing 500-case construction gate and its scope statement.

## Documentation Requirements

Update only when the implementation and tests support the claim:

- `resources/en/rulepack.manifest` for rule contracts.
- `README.md` capability and limitation statements.
- `ROADMAP.md` milestone status or deferred follow-up.
- `CHANGELOG.md` for user-visible rule or serialized-output changes.
- `docs/RESOURCES.md` only if reference artifacts change.

Do not claim complete English grammar, general proofreading, or broad accuracy.

## Verification Gate

Run these commands from the repository root:

```sh
cargo fmt --all
cargo test --all-targets --offline
cargo clippy --all-targets --all-features --offline -- -D warnings
cargo check --all-targets --all-features --offline
make gate
```

If the public API or Wasm boundary changes, also run:

```sh
cargo check --target wasm32-unknown-unknown --features wasm --offline
wasm-pack test --node --no-default-features --features wasm
```

The task is not complete if any command fails. Include the exact command and
result in the pull request description.

## Stop Conditions

Stop implementation and describe the blocker if any of the following occurs:

- A requested diagnostic needs unknown morphology treated as known.
- A relation must be fabricated or reinterpreted.
- A parser ambiguity cannot be represented explicitly.
- A dependency, network call, clock, RNG, or filesystem access appears needed.
- A serialized model or public version must change without an explicit design.
- The requested breadth cannot be covered by deterministic fixtures.

In those cases, leave the current behavior intact and propose a smaller
follow-up task in the pull request rather than weakening the contract.

## Completion Statement

The implementation is complete when the selected constructions have declared
rules, positive/clean/limitation fixtures, provenance and retraction coverage,
independently specified evaluation expectations, updated capability docs, and a
green verification gate. The result must be understandable and reproducible
from this document and the repository alone.
