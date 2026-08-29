> ## ⚠️ Read this before submitting
>
> **This project is open source but not open contribution.**
> Unsolicited pull requests are closed without merge. See
> [CONTRIBUTING.md](../blob/main/CONTRIBUTING.md).
>
> This is not personal and not a judgment of your work. Every guarantee this
> project makes — deterministic output, complete provenance, declared blind
> spots — is a global invariant that is expensive to verify in review.
>
> **If you have a bug, please open an issue instead.** Reproducible defect
> reports are the most useful thing you can send, and they get acted on.
>
> If this PR was pre-agreed with the maintainer, delete this block and fill in
> the rest.

## Agreed in

Link the issue or discussion where this was scoped. PRs without one are closed.

## What this changes

## Which invariants it touches

- [ ] Data model or serialization order (breaking — output bytes change)
- [ ] Rule-pack content, artifacts, or checksums
- [ ] Determinism-sensitive code path
- [ ] Support/provenance construction
- [ ] None of the above

## Checks

- [ ] `cargo test --offline` passes
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] No new dependency
- [ ] No `HashMap`/`HashSet`, clock, RNG, or env read added to an output path
- [ ] New rules are declared in the manifest with supports and blind spots
- [ ] New `SupportSet`s list every source materially consumed
- [ ] No test was loosened, deleted, or ignored to get green
- [ ] Signed off per [DCO.md](../blob/main/DCO.md) (`git commit -s`)

## Blind spots this introduces or leaves open