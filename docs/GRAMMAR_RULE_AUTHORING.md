# Grammar Rule Authoring

This is the execution playbook for adding one English grammar rule to Syntaxis.
It is written for a fresh cloud agent with a repository checkout and Beads
access. A rule is not complete when it produces a plausible message; it is
complete when its contract, evidence, limitations, and deterministic regression
behavior are all checked in.

Codex / agent invocation: `$grammar-rule-authoring`
(`.agents/skills/grammar-rule-authoring/SKILL.md`). Keep that skill and this
document aligned.

## Rule Boundary

Grammar rules consume the existing `Analysis` snapshot. They do not re-tokenize,
re-parse, read files, call a service, use a clock, or infer facts outside their
declared evidence. A rule may inspect:

- Token surfaces, spans, and sentence membership.
- Token analyses and their morphology.
- Resolved dependency arcs and their status.
- Existing support sources and certainty.

A rule emits a `GrammarDiagnostic` only when its required evidence is present.
Unknown morphology, unsupported arcs, and alternative arcs must not be silently
treated as definite evidence.

## Start With Beads

1. Run `bd prime` and `bd ready`.
2. Inspect the assigned issue with `bd show <id>`.
3. If the issue mixes multiple constructions, split it before coding. Keep each
   implementation Bead at Fibonacci 8 points or less.
4. Claim exactly one implementation Bead with `bd update <id> --claim`.
5. Record any newly discovered construction, fixture, API, or documentation
   work as a dependent Bead before continuing.

Stop and update the plan if the proposed rule needs a new relation, a new
morphology field, a changed serialization shape, or behavior outside the issue.
Those are architecture changes, not hidden details of a grammar rule.

## Define the Contract

Before implementation, write down:

- Construction name and one-sentence purpose.
- Exact positive examples the rule supports.
- Exact negative or clean examples where it must not fire.
- Required token analyses and dependency relations.
- Diagnostic kind, message key, rule id, and span policy.
- Support sources required for every conclusion.
- Blind spots and unsupported forms that remain explicit.
- Whether the change affects engine, rule-pack, or CoNLL-U mapping versions.

Use stable identifiers. Rule ids are declared in
`resources/en/rulepack.manifest`; message keys are serialized API fields.
Changing either is a compatibility decision and must be recorded in the
changelog when released.

## Manifest First

Add the rule entry to `resources/en/rulepack.manifest` before writing the rule.
Include:

```text
[rule.GRAMMAR.<NAME>.V1]
stage = grammar
description = What the rule detects.
supports = The evidence and construction boundary.
blind_spots = Forms intentionally not handled.
precision_target = not yet measured; set after the frozen-corpus baseline
```

The loader rejects undeclared rule ids. Keep the manifest description narrower
than the implementation if necessary; never claim coverage that the fixture
set does not exercise.

## Implement in the Existing Layer

Grammar implementation belongs in `src/english_rules/grammar.rs` unless the
Bead explicitly changes the architecture. Follow the existing pattern:

1. Filter arcs for the current sentence.
2. Check that required heads, dependents, and token analyses exist.
3. Check relation status before treating an arc as accepted evidence.
4. Compare only known morphology values.
5. Build the diagnostic span from the involved token spans.
6. Include the sentence, token-analysis, and material arc sources.
7. Use `DerivationKind::GrammarRule` and the declared rule id.
8. Let `Analysis::add_diagnostic` enforce the certainty contract.

Do not add a replacement suggestion unless the replacement is deterministic and
the existing model can represent it without inventing a new semantic claim.

## Add Evidence Before Broadening Code

Add tests in `src/english_rules/grammar.rs` or a focused integration test. Each
new construction needs:

- At least one positive case that must emit the expected diagnostic kind.
- At least one clean case that must emit no diagnostic of that kind.
- A nearest blind-spot case that remains clean, unsupported, or explicitly
  alternative as designed.
- A provenance assertion covering every material source.
- A retraction assertion when the diagnostic depends on a retractable arc or
  token analysis.

If the construction is suitable for the frozen evaluation gate, add an explicit
expected diagnostic kind to `src/english_rules/evaluation.rs`. Do not derive a
new expected result from the implementation under test.

## Determinism and Safety

Run the rule repeatedly and check that:

- Canonical JSON is byte-identical.
- `Analysis::digest()` is unchanged across runs.
- Diagnostic ids and source ordering do not depend on hash iteration order.
- Unsupported and ambiguous structures are not silently accepted.
- Pathological input does not panic or grow without the documented bound.

The analysis path must remain offline and free of filesystem, environment,
network, clock, and randomness access. Do not introduce `HashMap` or `HashSet`
iteration into an output path.

## Required Verification

From the repository root, run:

```sh
cargo fmt --all
cargo test --all-targets --offline
cargo clippy --all-targets --all-features --offline -- -D warnings
cargo check --all-targets --all-features --offline
make gate
```

If the rule touches the consumer API or Wasm boundary, also run:

```sh
cargo check --target wasm32-unknown-unknown --features wasm --offline
wasm-pack test --node --no-default-features --features wasm
```

Record failures in the active Bead. Do not close the Bead with a skipped gate
unless the issue explicitly changes the gate and a replacement check is added.

## Documentation and Release Accounting

Update the capability matrix and README when the supported user-visible scope
changes. Update `ROADMAP.md` when a milestone status changes. Update
`docs/RESOURCES.md` and the manifest when reference data changes. Update
`CHANGELOG.md` when serialized output, rule behavior, or a public identifier
changes.

Do not describe a construction as generally supported if only one template or
one fixture demonstrates it. State the supported shape and the blind spots.

## Closeout

Before closing the Bead:

1. Review `git diff` and `git diff --check`.
2. Confirm all required tests are green.
3. Confirm the rule id is declared and the fixture is checked in.
4. Confirm README, roadmap, capability matrix, and changelog impact.
5. Add follow-up Beads for intentionally deferred constructions.
6. Close the implementation Bead with the exact verification commands and
   measured result.

The result should be reproducible by another agent from the Bead, the manifest,
the fixture, and this document alone.
