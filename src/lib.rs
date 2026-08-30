//! Syntaxis — deterministic, offline English structural analysis.
//!
//! The package exposes the analysis model, English rules, and strict CoNLL-U
//! interoperability as modules of one crate. The command-line binary is
//! available as the `syntaxis` executable.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod conllu;
pub mod english_rules;
pub mod parser_core;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;

pub use conllu::{export, export_with, import_str, ConlluError, ExportOptions, MAPPING_VERSION};
pub use conllu::{export as conllu_export, import as conllu_import, mapping as conllu_mapping};
pub use english_rules::rulepack::RulePack;
pub use english_rules::{
    evaluation, grammar, parser, pipeline, pos, rulepack, segment, text, tokenize,
};
pub use parser_core::{
    analysis, factgraph, hash, ids, json, model, serialize, span, support, AlternativeGroup,
    AlternativeGroupId, Analysis, ArcId, ArcStatus, Certainty, DependencyArc, DerivationKind,
    DiagnosticId, DiagnosticKind, Document, DocumentId, Explanation, FactGraph, FactId,
    GrammarDiagnostic, Json, LexiconRef, MessageKey, Morphology, Number, Person, Pos, PronKind,
    Relation, Replacement, Resolution, RetractionReport, RuleId, RulePackId, Sentence, SentenceId,
    SourceRef, Span, SpanError, SupportSet, Tense, Token, TokenAnalysis, TokenId, UPos,
    ValidationIssue, VerbForm, Version, ANALYSIS_VERSION,
};
