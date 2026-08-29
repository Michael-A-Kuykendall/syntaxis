# Reference data provenance

Every linguistic artifact this engine consults is listed here. An artifact
without a complete entry is not mergeable, at any quality level. This is the
file a downstream legal review reads.

## Why this file is strict

Reference data is where a project like this actually gets into trouble. The
engine's own code is easy to license. A word list copied from a grammar checker
whose resource bundle carries copyleft obligations is not, and by the time
anyone notices, it is embedded in every release built since.

Two rules follow, and they are absolute:

- **No LanguageTool-derived content.** nlprule's binary resources are derived
  from LanguageTool and carry LGPL-2.1 obligations. They may be useful as a
  research comparator during development; they may not enter this tree.
- **No Harper-derived content** without a separate written licence review.

## Artifact register

| Artifact | Version | Licence | Source | Checksum recorded in |
| --- | --- | --- | --- | --- |
| `en.abbreviations` | 0.1.0 | Apache-2.0 | Original, compiled by hand | `rulepack.manifest` |
| `en.clitics` | 0.1.0 | Apache-2.0 | Original; Penn split-point convention | `rulepack.manifest` |
| `en.fused` | 0.1.0 | Apache-2.0 | Original; Penn split-point convention | `rulepack.manifest` |

### en.abbreviations

- **Path:** `resources/en/abbreviations.txt`
- **Contents:** English abbreviations whose trailing period must not trigger a
  sentence break.
- **Generation:** hand maintained. No extraction from any corpus or tool.
- **Normalization policy:** none. Matched byte-exact against the token surface,
  including the trailing period, and case-sensitively.
- **Fixture coverage:** `segment::tests::does_not_split_known_abbreviations`,
  `tokenize::tests::abbreviations_keep_their_period_and_record_the_entry`.
- **Retraction:** tokens that consult an entry carry a `SourceRef::Lexicon`, so
  retracting the entry retracts exactly those tokens. Tested in
  `pipeline::tests::retracting_a_lexicon_entry_cascades`.

### en.clitics

- **Path:** `resources/en/clitics.txt`
- **Contents:** clitic suffixes split into their own tokens.
- **Generation:** hand maintained.
- **Normalization policy:** ASCII lowercase; curly apostrophes folded to ASCII
  before matching, so `don't` and `don’t` split identically while the emitted
  surface keeps the original character.
- **Note on convention:** the split points follow the Penn Treebank *convention*
  — the conventional places to divide a contraction. A convention is not
  copyrightable and no Penn data is included or required.

### en.fused

- **Path:** `resources/en/fused.txt`
- **Contents:** fused forms and their conventional parts.
- **Generation:** hand maintained.
- **Loader validation:** the parts of every entry must concatenate to the
  surface exactly, or the artifact is rejected at load. Without that check a
  bad entry would silently corrupt token spans.

## Known gaps

- **No Unicode normalization (NFC/NFD).** It needs either a dependency or a
  large embedded table, and both conflict with the zero-dependency rule. Input
  is processed as given. Text mixing composed and decomposed forms will tokenize
  inconsistently between them. Recorded rather than hidden.
- **No lexicon yet.** POS and morphology are modelled and populated on CoNLL-U
  import, but nothing derives them from text. The next lexicon is the hardest
  provenance decision — see below.

## Adding an artifact

Required before merge, no exceptions:

1. **Source.** Where it came from. "Compiled by hand" is a valid answer;
   "found in another project" is not, without its licence.
2. **Licence.** The artifact's own licence, which is not necessarily the
   project's.
3. **Normalization policy.** Exactly how a lookup key is derived from a surface.
4. **Generation command.** If it was produced by a script, the script and its
   invocation, so the artifact can be regenerated and verified.
5. **Checksum.** SHA-256 in `rulepack.manifest`.
6. **Fixture coverage.** At least one test that fails if the artifact is wrong.
7. **A version.** Bumped whenever content changes, so consulting facts can be
   invalidated precisely.

## The lexicon decision

The open question, recorded here so it is not decided by accident:

| Option | Cost | Risk |
| --- | --- | --- |
| Build an original lexicon | Slow; broad coverage is a lot of hand work | Low legal risk, full control |
| Use a permissively licensed existing lexicon | Fast | Must audit its own provenance chain, not just its stated licence |
| Adapt around a pinned external tool | Fastest | Reintroduces a runtime dependency and its determinism assumptions |

Whichever is chosen, the entry above it in this file gets written **before** the
code that uses it.