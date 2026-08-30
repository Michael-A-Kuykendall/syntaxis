# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## Unreleased

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
