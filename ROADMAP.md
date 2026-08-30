# Roadmap

Milestones are gates, not dates. A milestone is not complete because the code
compiles; it is complete when its acceptance criteria in
[RELEASE_GATES_CHECKLIST.md](RELEASE_GATES_CHECKLIST.md) pass on frozen data.

## Foundation — Feasibility and architecture proof ✅ implemented

- [x] Freeze the data model and rule-pack format
- [x] Tokenization, spans, canonical serialization
- [x] Import/export one hand-annotated CoNLL-U fixture
- [x] Demonstrate support tracking and one retraction
- [x] Construction-focused 500-case evaluation gate is frozen and repeatable

## Structural kernel ✅ first slices implemented

- [x] POS and morphology candidate facts from a versioned lexicon
- [ ] Phrase and chunk boundaries beyond the current construction slices
- [x] Root, subject, object, auxiliary, determiner, negation, preposition, and
      expletive arcs for the supported constructions
- [x] Alternative groups and unsupported status populated by declared rules
- [x] Challenge fixtures for the three agreement cases pass structurally

## Grammar layer ✅ first slices implemented

- [x] Initial agreement, determiner, verb-form, and negation-placement rules
- [x] Known-person subject/finite agreement when both persons are known
- [x] Do-support as an accepted auxiliary host for `not`
- [x] Infinitival `to` marks a following known non-finite verb for bounded form diagnostics
- [x] Coordination number agreement only over resolved `conj` arcs
- [x] Grammar diagnostics kept separate from spelling, capitalization, and style
- [x] Construction-focused precision, recall, and clean-text results on frozen data
- [x] Every diagnostic carries rule id, supports, spans, and a stable message key

## Downstream adapter

- [ ] Project one analysis into the consuming system's graph
- [ ] Preserve supports and rule ids across the projection
- [ ] Retraction cascade tests across the boundary
- [ ] Disabling the adapter produces no parser-derived facts

## Expansion decision

The first structural gate passes. Expansion now chooses among more
constructions, a better lexicon, or selected external resources; each addition
requires new frozen fixtures and explicit provenance.

## Post-v0.1 grammar priorities

Rank reflects consumer value multiplied by fit with the current deterministic
pipeline. Each item is a separate implementation slice, not a promise of broad
coverage. No item is advertised until its fixture and criterion pass.

1. **Auxiliary chains and do-support.** Extend accepted `aux` evidence for
   `be`, `have`, and `do` without rewriting the sentence. Freeze clean and
   hostless-negation cases in `fixtures/expansion_auxiliary.conllu`; require
   exact relation status, placement diagnostic kind, and complete provenance.
2. **Basic clausal complements.** Add one resolved verb-complement shape using
   an existing relation, with no control or semantic inference. Freeze one
   clean, one incompatible-form, and one unsupported-attachment case in
   `fixtures/expansion_complement.conllu`; require the unsupported case to
   remain explicit and the 500-case gate to remain unchanged.
3. **Nominal and clause coordination.** Separate resolved nominal `conj` from
   shared-argument and clause coordination. Freeze agreement-clean,
   agreement-error, and ambiguous-scope cases in
   `fixtures/expansion_coordination.conllu`; require no diagnostic from an
   alternative or unsupported arc.
4. **Subordination.** Support one bounded marker-plus-clause shape only after
   the parser can supply accepted evidence. Freeze marker attachment and
   sentence-scope cases in `fixtures/expansion_subordination.conllu`; require
   deterministic arcs, retraction coverage, and no fabricated clause relation.
5. **Questions and passive/copular variants.** Choose one of these only after
   priorities 1-4 establish the required auxiliary and clause evidence. Freeze
   paired clean/error examples in a construction-specific fixture; require
   exact diagnostic labels, stable JSON/digest, and a measured held-out result.

Deferred until the prerequisites or evidence exist:

- Person on ambiguous verb forms (`have`, `do`, `was`) because the surface form
  does not uniquely encode person.
- Infinitival `to` diagnostics until an accepted `mark` relation is emitted by
  the parser for the supported construction.
- Broadening the 500-case gate until a licensed held-out corpus and hand-written
  expected labels exist.
- Full phrase/chunk boundaries, semantic role labeling, and general attachment
  ambiguity because they exceed the current structural contract.

Deferred follow-up from the first-slice expansion:

- Person on ambiguous verb forms other than `are` (`have`, `do`, `was`)
- Clause-level and shared-argument coordination
- Prepositional `to` vs infinitival `to` beyond the current unsupported remainder
- Broadening the 500-case gate to person, placement, or infinitive kinds

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

Broad natural-language coverage cannot be measured reliably without a larger
corpus:

- 500+ English sentences, held-out test split frozen **before** any tuning
- Formal prose, informal prose, non-native prose, questions, negation,
  coordination, subordination
- A focused challenge set for the three motivating cases
- Gold annotations for the supported relations
- Explicit licence and provenance for every component

Thresholds get proposed after a baseline is measured against this corpus. Any
threshold invented before that measurement is a number chosen to be passed.
