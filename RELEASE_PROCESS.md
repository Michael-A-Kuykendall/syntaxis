# Release process

## Versioning

Three versions move independently and all three appear in serialized output:

| Version | Bump when |
| --- | --- |
| Engine (`Cargo.toml`) | any code change |
| Rule pack (`resources/en/rulepack.manifest`) | any rule or artifact change |
| CoNLL-U mapping (`syntaxis::conllu::MAPPING_VERSION`) | the Penn↔UD projection changes |

**A change to output bytes is a breaking change**, even with no API change.
Consumers pin digests; that is the interface.

## Steps

1. **Freeze.** No feature merges after this point.
2. **Run the gates.** [RELEASE_GATES_CHECKLIST.md](RELEASE_GATES_CHECKLIST.md),
   every box. `make gate` covers the automated subset.
3. **Bump versions.** Whichever of the three moved.
4. **Write the changelog.** Name which versions moved and whether output bytes
   changed. Credit reporters.
5. **Regenerate fixture digests** if output changed, and eyeball the diff — an
   unexpected fixture diff means an unintended behaviour change.
6. **Dry run.** `cargo publish --dry-run` for the single `syntaxis` package.
7. **Tag.** `git tag -s v0.1.0 -m "v0.1.0"` and push the tag.
8. **Publish** the `syntaxis` package.
9. **GitHub release.** Paste the changelog section. Attach nothing that was not
   built from the tag.

## Publishing

The package contains the library modules and the `syntaxis` binary, so there is
no inter-package publish order. If publishing fails, do not yank and retry the
same version — bump the patch and go again. Yanked versions still occupy the
number.

## Hotfixes

Same gates, no exceptions. A hotfix that skips Gate 2 can ship a determinism
regression to every consumer pinning digests, which is worse than the bug being
fixed.

## After release

- Move `[Unreleased]` items into the new section
- Update `ROADMAP.md` checkboxes
- Verify `cargo install` from crates.io works on a clean machine
