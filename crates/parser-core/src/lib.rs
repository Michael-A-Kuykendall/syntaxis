//! `parser-core` — the shared deterministic substrate.
//!
//! This crate owns the data model and nothing else. It contains no English
//! knowledge, no lexicon, no rules, and no I/O. The grammar layer and the
//! dependency layer are both *consumers* of what is defined here — the
//! concrete meaning of "sibling products over one substrate".
//!
//! Invariants this crate enforces mechanically rather than by convention:
//!
//! * every derived fact carries a [`support::SupportSet`] naming its rule,
//!   rule pack, derivation kind, and every source it consumed;
//! * retracting a source removes exactly its transitive dependents, with no
//!   caller-side cleanup;
//! * serialization is byte-for-byte stable and contains no timing data;
//! * a result may not claim more certainty than its supports allow, and
//!   [`analysis::Analysis::validate`] reports it when one does.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod analysis;
pub mod factgraph;
pub mod hash;
pub mod ids;
pub mod json;
pub mod model;
pub mod serialize;
pub mod span;
pub mod support;

pub use analysis::{Analysis, ValidationIssue};
pub use factgraph::{Explanation, FactGraph, RetractionReport};
pub use ids::{
    AlternativeGroupId, ArcId, DiagnosticId, DocumentId, MessageKey, RuleId, RulePackId,
    SentenceId, TokenId, Version,
};
pub use json::Json;
pub use model::{
    AlternativeGroup, ArcStatus, Certainty, DependencyArc, DetKind, DiagnosticKind, Document,
    GrammarDiagnostic, Morphology, Number, Person, Pos, PronKind, Relation, Replacement,
    Resolution, Sentence, Tense, Token, TokenAnalysis, UPos, VerbForm,
};
pub use span::{Span, SpanError};
pub use support::{DerivationKind, FactId, LexiconRef, SourceRef, SupportSet};

/// Version of the analysis data model itself. Any change to the shapes in
/// [`model`] or to serialization order requires a bump here.
pub const ANALYSIS_VERSION: Version = Version::new(0, 1, 0);
