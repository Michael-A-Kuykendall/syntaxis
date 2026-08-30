# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## Unreleased

- Declared and tested `GRAMMAR.AGREEMENT.SUBJECT_VERB_PERSON.V1` for known-person
  subject/finite mismatches. Unknown person is left silent.
- Narrowed negation placement so an accepted `do` auxiliary is a host for `not`,
  and so `never` is not treated as a hostless error.
- Added a bounded infinitival-`to` form diagnostic over an accepted parser
  `mark` relation; prepositional and unresolved `to` remain unsupported.
- Added coordination fixtures that require a resolved `conj` arc and leave
  shared-argument coordination unclaimed.
- Live morphology for `are` no longer claims `Person=3`, because that form is
  not unique to third person. The CoNLL-U gold fixture is unchanged. No
  `analysis_version`, rule-pack version, or mapping-version bump: the serialized
  model shape is the same.
- Live morphology for `have` likewise leaves person unknown because the form is
  shared by first, second, and plural third-person subjects.
- Consolidated the implementation into one publishable `syntaxis` package with
  `parser_core`, `english_rules`, and `conllu` library modules plus the
  `syntaxis` binary.
- Added the checksummed English rule-pack artifacts to the package so builds do
  not depend on the repository checkout.
- Added an optional `wasm` feature with `wasm-bindgen` exports for canonical JSON
  analysis, digests, and strict CoNLL-U import.

- Initial release-shaped API for deterministic, offline English structural
  analysis.
- The crate name is intentionally `syntaxis`; the Rust package and command-line
  application share that name, while the README describes this project as the
  zero-dependency Rust structural-analysis engine to distinguish it from
  unrelated projects with similar names.
- Canonical serialization is a compatibility contract. Changes to serialized
  field order or model shapes require a version bump.
