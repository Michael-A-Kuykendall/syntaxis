//! The deterministic analysis data model and support graph.

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

/// Version of the analysis data model and serialization contract.
pub const ANALYSIS_VERSION: Version = Version::new(0, 1, 0);
