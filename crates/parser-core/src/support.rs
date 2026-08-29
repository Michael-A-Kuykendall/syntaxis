//! Provenance.
//!
//! Principle 4.5: a text span alone is not provenance. Every derived item
//! records *which facts it consumed*, *which rule consumed them*, *which rule
//! pack that rule came from*, and *what kind of derivation it was*. That is the
//! minimum needed to (a) explain a result to a human and (b) retract it
//! mechanically when an input goes away.

use crate::ids::*;
use crate::span::Span;
use std::fmt;

/// A reference to something a derivation consumed.
///
/// The ordering of this enum is part of the canonical serialization order, so
/// variants must not be reordered without a version bump.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum SourceRef {
    /// Raw input bytes. This is an axiom: nothing derives it.
    Text(Span),
    Sentence(SentenceId),
    Token(TokenId),
    TokenAnalysis(TokenId),
    Arc(ArcId),
    AlternativeGroup(AlternativeGroupId),
    Diagnostic(DiagnosticId),
    /// A specific entry in a versioned reference artifact.
    Lexicon(LexiconRef),
}

impl SourceRef {
    /// The fact this reference points at, if it is a retractable derived fact.
    /// `Text` and `Lexicon` are axioms for the engine's purposes; a consumer
    /// may still retract them through [`crate::factgraph::FactGraph::retract`].
    pub fn as_fact(&self) -> Option<FactId> {
        match self {
            SourceRef::Token(t) => Some(FactId::Token(*t)),
            SourceRef::TokenAnalysis(t) => Some(FactId::TokenAnalysis(*t)),
            SourceRef::Arc(a) => Some(FactId::Arc(*a)),
            SourceRef::AlternativeGroup(x) => Some(FactId::AlternativeGroup(*x)),
            SourceRef::Diagnostic(g) => Some(FactId::Diagnostic(*g)),
            SourceRef::Text(_) | SourceRef::Sentence(_) | SourceRef::Lexicon(_) => None,
        }
    }
}

impl fmt::Display for SourceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceRef::Text(s) => write!(f, "text:{s}"),
            SourceRef::Sentence(s) => write!(f, "sentence:{s}"),
            SourceRef::Token(t) => write!(f, "token:{t}"),
            SourceRef::TokenAnalysis(t) => write!(f, "analysis:{t}"),
            SourceRef::Arc(a) => write!(f, "arc:{a}"),
            SourceRef::AlternativeGroup(x) => write!(f, "altgroup:{x}"),
            SourceRef::Diagnostic(g) => write!(f, "diagnostic:{g}"),
            SourceRef::Lexicon(l) => write!(f, "lexicon:{}/{}@{}", l.artifact, l.entry, l.version),
        }
    }
}

/// A pinned entry in a reference artifact.
///
/// `version` is the artifact version, not the engine version: replacing a
/// lexicon must invalidate exactly the facts that consulted it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LexiconRef {
    pub artifact: String,
    pub entry: String,
    pub version: Version,
}

impl LexiconRef {
    pub fn new(artifact: &str, entry: &str, version: Version) -> Self {
        LexiconRef {
            artifact: artifact.to_string(),
            entry: entry.to_string(),
            version,
        }
    }
}

/// How a fact came to exist. Kept coarse on purpose: the fine detail is the
/// `RuleId`, and this field exists so consumers can filter whole classes of
/// derivation without enumerating rules.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum DerivationKind {
    /// Segmentation and tokenization directly over input bytes.
    Surface,
    /// Lookup in a versioned reference artifact.
    LexicalLookup,
    /// Deterministic contextual rule over lexical facts.
    Contextual,
    /// Dependency candidate proposal.
    Attachment,
    /// Constraint solving over candidates (accept, rank, reject).
    Constraint,
    /// Grammar rule over resolved structure.
    GrammarRule,
    /// Import of an externally authored analysis (e.g. gold CoNLL-U).
    Import,
    /// Projection into a downstream representation.
    Projection,
}

impl DerivationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DerivationKind::Surface => "surface",
            DerivationKind::LexicalLookup => "lexical_lookup",
            DerivationKind::Contextual => "contextual",
            DerivationKind::Attachment => "attachment",
            DerivationKind::Constraint => "constraint",
            DerivationKind::GrammarRule => "grammar_rule",
            DerivationKind::Import => "import",
            DerivationKind::Projection => "projection",
        }
    }
}

impl fmt::Display for DerivationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The complete justification for one derived fact.
///
/// `sources` must list **every materially contributing** source, including all
/// tokens consulted by a sentence-level calculation. Under-reporting sources is
/// the bug class this whole design exists to prevent: an unreported source is a
/// fact that will not be retracted when it should be.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SupportSet {
    pub sources: Vec<SourceRef>,
    pub rule: RuleId,
    pub rule_pack: RulePackId,
    pub derivation: DerivationKind,
}

impl SupportSet {
    pub fn new(
        rule: RuleId,
        rule_pack: RulePackId,
        derivation: DerivationKind,
        sources: Vec<SourceRef>,
    ) -> Self {
        let mut set = SupportSet {
            sources,
            rule,
            rule_pack,
            derivation,
        };
        set.canonicalize();
        set
    }

    /// Sort and deduplicate sources so that two runs that discover the same
    /// sources in different orders serialize identically.
    pub fn canonicalize(&mut self) {
        self.sources.sort();
        self.sources.dedup();
    }

    pub fn with_source(mut self, source: SourceRef) -> Self {
        self.sources.push(source);
        self.canonicalize();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

/// Identity of a retractable derived fact.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum FactId {
    Token(TokenId),
    TokenAnalysis(TokenId),
    Arc(ArcId),
    AlternativeGroup(AlternativeGroupId),
    Diagnostic(DiagnosticId),
}

impl FactId {
    pub fn as_source(&self) -> SourceRef {
        match self {
            FactId::Token(t) => SourceRef::Token(*t),
            FactId::TokenAnalysis(t) => SourceRef::TokenAnalysis(*t),
            FactId::Arc(a) => SourceRef::Arc(*a),
            FactId::AlternativeGroup(x) => SourceRef::AlternativeGroup(*x),
            FactId::Diagnostic(g) => SourceRef::Diagnostic(*g),
        }
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_source())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_canonicalize_regardless_of_discovery_order() {
        let a = SupportSet::new(
            RuleId::new("R"),
            RulePackId::new("p@0.1.0"),
            DerivationKind::Contextual,
            vec![SourceRef::Token(TokenId(3)), SourceRef::Token(TokenId(1))],
        );
        let b = SupportSet::new(
            RuleId::new("R"),
            RulePackId::new("p@0.1.0"),
            DerivationKind::Contextual,
            vec![
                SourceRef::Token(TokenId(1)),
                SourceRef::Token(TokenId(3)),
                SourceRef::Token(TokenId(1)),
            ],
        );
        assert_eq!(a, b);
    }
}
