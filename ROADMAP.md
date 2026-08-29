# Roadmap

Milestones are gates, not dates. A milestone is not complete because the code
compiles; it is complete when its acceptance criteria in
[RELEASE_GATES_CHECKLIST.md](RELEASE_GATES_CHECKLIST.md) pass on frozen data.

## Foundation — Feasibility and architecture proof ✅ implemented, ungated

- [x] Freeze the data model and rule-pack format
- [x] Tokenization, spans, canonical serialization
- [x] Import/export one hand-annotated CoNLL-U fixture
- [x] Demonstrate support tracking and one retraction
- [ ] Ungated: no corpus exists yet, so no measurement has been made

## Structural kernel

- [ ] POS and morphology candidate facts from a versioned lexicon
- [ ] Phrase and chunk boundaries
- [ ] Root, subject, object, auxiliary, determiner, negation, preposition arcs
- [ ] Alternative groups and unsupported status populated by real rules
- [ ] Challenge fixtures for the three agreement cases pass structurally
- [ ] **Decision required:** add `expl` to the relation set — existential
      *there* is currently inexpressible, which blocks one motivating case

## Grammar layer

- [ ] The initial grammar rules
- [ ] Grammar diagnostics kept separate from spelling, capitalization, style
- [ ] Precision, recall, and clean-text false-positive rates on frozen splits
- [ ] Every diagnostic carries rule id, supports, spans, and a stable message key

## Downstream adapter

- [ ] Project one analysis into the consuming system's graph
- [ ] Preserve supports and rule ids across the projection
- [ ] Retraction cascade tests across the boundary
- [ ] Disabling the adapter produces no parser-derived facts

## Expansion decision

Only after the earlier milestones pass. Then choose: more constructions, a
better lexicon, or selected external resources.

## Explicit non-goals

Not "later" — not planned:

- Neural or statistical inference of any kind
- Multilingual support
- Full Universal Dependencies parity
- Replacing LanguageTool, Grammarly, or any general proofreader
- Spelling correction
- Semantic role labelling, discourse analysis, or AI-text detection
- Claims of state-of-the-art parsing accuracy

## The corpus, which gates everything after the structural kernel

Nothing past the structural kernel can be honestly assessed without it:

- 500+ English sentences, held-out test split frozen **before** any tuning
- Formal prose, informal prose, non-native prose, questions, negation,
  coordination, subordination
- A focused challenge set for the three motivating cases
- Gold annotations for the supported relations
- Explicit licence and provenance for every component

Thresholds get proposed after a baseline is measured against this corpus. Any
threshold invented before that measurement is a number chosen to be passed.