//! The single analysis snapshot.
//!
//! One document produces exactly one `Analysis`. Grammar rules and dependency
//! rules are both *consumers* of this object; neither re-tokenizes, and neither
//! mutates the other's output. Everything they add lands here with a support
//! record, which is what makes retraction mechanical.

use crate::factgraph::{FactGraph, RetractionReport};
use crate::ids::*;
use crate::model::*;
use crate::span::SpanError;
use crate::support::{FactId, SourceRef};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Analysis {
    pub document: Document,
    pub sentences: BTreeMap<SentenceId, Sentence>,
    pub tokens: BTreeMap<TokenId, Token>,
    pub token_analyses: BTreeMap<TokenId, TokenAnalysis>,
    pub arcs: BTreeMap<ArcId, DependencyArc>,
    pub alternative_groups: BTreeMap<AlternativeGroupId, AlternativeGroup>,
    pub diagnostics: BTreeMap<DiagnosticId, GrammarDiagnostic>,
    pub graph: FactGraph,
    /// Audit log of retractions applied to this snapshot, in application order.
    pub retractions: Vec<RetractionReport>,
}

impl Analysis {
    pub fn new(document: Document) -> Self {
        Analysis {
            document,
            sentences: BTreeMap::new(),
            tokens: BTreeMap::new(),
            token_analyses: BTreeMap::new(),
            arcs: BTreeMap::new(),
            alternative_groups: BTreeMap::new(),
            diagnostics: BTreeMap::new(),
            graph: FactGraph::new(),
            retractions: Vec::new(),
        }
    }

    // -- insertion ---------------------------------------------------------

    pub fn add_sentence(&mut self, sentence: Sentence) {
        self.document.sentences.push(sentence.id);
        self.document.sentences.sort();
        self.document.sentences.dedup();
        self.sentences.insert(sentence.id, sentence);
    }

    pub fn add_token(&mut self, token: Token) {
        self.graph
            .assert_fact(FactId::Token(token.id), token.support.clone());
        self.tokens.insert(token.id, token);
    }

    pub fn add_token_analysis(&mut self, analysis: TokenAnalysis) {
        self.graph.assert_fact(
            FactId::TokenAnalysis(analysis.token),
            analysis.support.clone(),
        );
        self.token_analyses.insert(analysis.token, analysis);
    }

    pub fn add_arc(&mut self, arc: DependencyArc) {
        self.graph
            .assert_fact(FactId::Arc(arc.id), arc.support.clone());
        self.arcs.insert(arc.id, arc);
    }

    pub fn add_alternative_group(&mut self, group: AlternativeGroup) {
        self.graph
            .assert_fact(FactId::AlternativeGroup(group.id), group.support.clone());
        self.alternative_groups.insert(group.id, group);
    }

    pub fn add_diagnostic(&mut self, mut diagnostic: GrammarDiagnostic) {
        // Diagnostics cannot be more certain than the least certain evidence
        // they consume. Normalize at insertion so invalid state is impossible.
        diagnostic.certainty = diagnostic
            .certainty
            .max(self.certainty_for(&diagnostic.support.sources));
        self.graph.assert_fact(
            FactId::Diagnostic(diagnostic.id),
            diagnostic.support.clone(),
        );
        self.diagnostics.insert(diagnostic.id, diagnostic);
    }

    // -- queries -----------------------------------------------------------

    pub fn sentence_tokens(&self, sentence: SentenceId) -> Vec<&Token> {
        self.sentences
            .get(&sentence)
            .map(|s| s.tokens.iter().filter_map(|t| self.tokens.get(t)).collect())
            .unwrap_or_default()
    }

    pub fn surface_of(&self, token: TokenId) -> Option<&str> {
        self.tokens.get(&token).map(|t| t.surface.as_str())
    }

    pub fn next_arc_id(&self) -> ArcId {
        ArcId(self.arcs.keys().next_back().map(|a| a.0 + 1).unwrap_or(0))
    }

    pub fn next_diagnostic_id(&self) -> DiagnosticId {
        DiagnosticId(
            self.diagnostics
                .keys()
                .next_back()
                .map(|d| d.0 + 1)
                .unwrap_or(0),
        )
    }

    pub fn next_group_id(&self) -> AlternativeGroupId {
        AlternativeGroupId(
            self.alternative_groups
                .keys()
                .next_back()
                .map(|g| g.0 + 1)
                .unwrap_or(0),
        )
    }

    /// Certainty a derived item must carry, given what it rests on.
    ///
    /// A result may never be more certain than its least certain support.
    /// Token and token-analysis references identify observed facts but carry no
    /// independent uncertainty; unresolved structure is represented by arcs or
    /// alternative groups and is what lowers certainty.
    pub fn certainty_for(&self, sources: &[SourceRef]) -> Certainty {
        let mut certainty = Certainty::Definite;
        for source in sources {
            let arc_id = match source {
                SourceRef::Arc(a) => *a,
                _ => continue,
            };
            let Some(arc) = self.arcs.get(&arc_id) else {
                continue;
            };
            let level = match &arc.status {
                ArcStatus::Accepted => Certainty::Definite,
                ArcStatus::Rejected | ArcStatus::Unsupported => Certainty::Conditional,
                ArcStatus::Alternative { group } => match self
                    .alternative_groups
                    .get(group)
                    .map(|g| g.resolution)
                    .unwrap_or(Resolution::Ambiguous)
                {
                    Resolution::Unique => Certainty::Definite,
                    Resolution::Ranked => Certainty::Preferred,
                    Resolution::Ambiguous | Resolution::Unsupported => Certainty::Conditional,
                },
            };
            certainty = certainty.max(level);
        }
        certainty
    }

    // -- retraction --------------------------------------------------------

    /// Retract a source and everything derived from it, from both the fact
    /// graph and the collections. No caller-side cleanup is required.
    pub fn retract(&mut self, source: &SourceRef) -> RetractionReport {
        let report = self.graph.retract(source);
        for fact in &report.removed {
            self.remove_fact_from_collections(*fact);
        }
        if let Some(fact) = source.as_fact() {
            self.remove_fact_from_collections(fact);
        }
        // An alternative group that lost members must not keep dangling ids.
        let removed_arcs: BTreeSet<ArcId> = report
            .removed
            .iter()
            .filter_map(|f| match f {
                FactId::Arc(a) => Some(*a),
                _ => None,
            })
            .chain(match source {
                SourceRef::Arc(a) => Some(*a),
                _ => None,
            })
            .collect();
        if !removed_arcs.is_empty() {
            for group in self.alternative_groups.values_mut() {
                group.members.retain(|m| !removed_arcs.contains(m));
            }
            self.alternative_groups.retain(|_, g| !g.members.is_empty());
        }
        self.retractions.push(report.clone());
        report
    }

    fn remove_fact_from_collections(&mut self, fact: FactId) {
        match fact {
            FactId::Token(t) => {
                self.tokens.remove(&t);
                self.token_analyses.remove(&t);
                for sentence in self.sentences.values_mut() {
                    sentence.tokens.retain(|x| *x != t);
                }
            }
            FactId::TokenAnalysis(t) => {
                self.token_analyses.remove(&t);
            }
            FactId::Arc(a) => {
                self.arcs.remove(&a);
            }
            FactId::AlternativeGroup(g) => {
                self.alternative_groups.remove(&g);
            }
            FactId::Diagnostic(d) => {
                self.diagnostics.remove(&d);
            }
        }
    }

    // -- validation --------------------------------------------------------

    /// Structural checks. An empty result
    /// means: spans valid, token order valid, and every sentence is either a
    /// valid tree over accepted arcs or explicitly marked otherwise.
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        self.validate_spans(&mut issues);
        self.validate_token_order(&mut issues);
        self.validate_arcs(&mut issues);
        self.validate_trees(&mut issues);
        self.validate_supports(&mut issues);
        issues
    }

    fn validate_spans(&self, issues: &mut Vec<ValidationIssue>) {
        for token in self.tokens.values() {
            match token.span.validate(&self.document.text) {
                Err(e) => issues.push(ValidationIssue::BadSpan(token.id, e)),
                Ok(()) => {
                    if token.span.slice(&self.document.text) != Some(token.surface.as_str()) {
                        issues.push(ValidationIssue::SurfaceMismatch(token.id));
                    }
                }
            }
        }
        for sentence in self.sentences.values() {
            if let Err(e) = sentence.span.validate(&self.document.text) {
                issues.push(ValidationIssue::BadSentenceSpan(sentence.id, e));
            }
        }
    }

    fn validate_token_order(&self, issues: &mut Vec<ValidationIssue>) {
        for sentence in self.sentences.values() {
            let mut previous_end = None;
            for (index, token_id) in sentence.tokens.iter().enumerate() {
                let Some(token) = self.tokens.get(token_id) else {
                    issues.push(ValidationIssue::DanglingToken(sentence.id, *token_id));
                    continue;
                };
                if token.ordinal as usize != index {
                    issues.push(ValidationIssue::OrdinalMismatch(*token_id));
                }
                if token.sentence != sentence.id {
                    issues.push(ValidationIssue::SentenceMismatch(*token_id));
                }
                if let Some(end) = previous_end {
                    if token.span.byte_start < end {
                        issues.push(ValidationIssue::OverlappingTokens(*token_id));
                    }
                }
                previous_end = Some(token.span.byte_end);
                if !sentence.span.contains(&token.span) {
                    issues.push(ValidationIssue::TokenOutsideSentence(*token_id));
                }
            }
        }
    }

    fn validate_arcs(&self, issues: &mut Vec<ValidationIssue>) {
        for arc in self.arcs.values() {
            if !self.tokens.contains_key(&arc.dependent) {
                issues.push(ValidationIssue::ArcDependentMissing(arc.id));
            }
            if let Some(head) = arc.head {
                if !self.tokens.contains_key(&head) {
                    issues.push(ValidationIssue::ArcHeadMissing(arc.id));
                }
                if head == arc.dependent {
                    issues.push(ValidationIssue::SelfLoop(arc.id));
                }
            } else if arc.relation != Relation::Root {
                issues.push(ValidationIssue::HeadlessNonRoot(arc.id));
            }
            if arc.relation == Relation::Unsupported && arc.status != ArcStatus::Unsupported {
                issues.push(ValidationIssue::UnsupportedRelationNotMarked(arc.id));
            }
            if let ArcStatus::Alternative { group } = &arc.status {
                match self.alternative_groups.get(group) {
                    None => issues.push(ValidationIssue::DanglingAlternativeGroup(arc.id)),
                    Some(g) if !g.members.contains(&arc.id) => {
                        issues.push(ValidationIssue::AlternativeNotInGroup(arc.id))
                    }
                    Some(_) => {}
                }
            }
        }
        for group in self.alternative_groups.values() {
            if group.members.len() < 2 && group.resolution != Resolution::Unique {
                issues.push(ValidationIssue::DegenerateAlternativeGroup(group.id));
            }
        }
    }

    fn validate_trees(&self, issues: &mut Vec<ValidationIssue>) {
        for sentence in self.sentences.values() {
            let mut heads: BTreeMap<TokenId, Option<TokenId>> = BTreeMap::new();
            let mut roots = 0usize;
            let mut covered: BTreeSet<TokenId> = BTreeSet::new();

            for arc in self.arcs.values().filter(|a| a.sentence == sentence.id) {
                covered.insert(arc.dependent);
                if arc.status != ArcStatus::Accepted {
                    continue;
                }
                if heads.insert(arc.dependent, arc.head).is_some() {
                    issues.push(ValidationIssue::MultipleAcceptedHeads(arc.dependent));
                }
                if arc.head.is_none() {
                    roots += 1;
                }
            }

            if heads.is_empty() {
                continue; // nothing accepted for this sentence; not a tree claim
            }
            if roots != 1 {
                issues.push(ValidationIssue::RootCount(sentence.id, roots));
            }
            for token_id in &sentence.tokens {
                if !covered.contains(token_id) {
                    issues.push(ValidationIssue::TokenUnattached(*token_id));
                }
            }
            // Cycle detection over accepted heads.
            for start in heads.keys() {
                let mut current = *start;
                let mut steps = 0usize;
                loop {
                    match heads.get(&current).copied().flatten() {
                        None => break,
                        Some(head) => {
                            current = head;
                            steps += 1;
                            if current == *start || steps > heads.len() + 1 {
                                issues.push(ValidationIssue::Cycle(*start));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn validate_supports(&self, issues: &mut Vec<ValidationIssue>) {
        for arc in self.arcs.values() {
            if arc.support.is_empty() {
                issues.push(ValidationIssue::EmptySupport(FactId::Arc(arc.id)));
            }
        }
        for diagnostic in self.diagnostics.values() {
            if diagnostic.support.is_empty() {
                issues.push(ValidationIssue::EmptySupport(FactId::Diagnostic(
                    diagnostic.id,
                )));
            }
            let required = self.certainty_for(&diagnostic.support.sources);
            if diagnostic.certainty < required {
                issues.push(ValidationIssue::OverconfidentDiagnostic(diagnostic.id));
            }
        }
        for analysis in self.token_analyses.values() {
            if analysis.support.is_empty() {
                issues.push(ValidationIssue::EmptySupport(FactId::TokenAnalysis(
                    analysis.token,
                )));
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ValidationIssue {
    BadSpan(TokenId, SpanError),
    BadSentenceSpan(SentenceId, SpanError),
    SurfaceMismatch(TokenId),
    DanglingToken(SentenceId, TokenId),
    OrdinalMismatch(TokenId),
    SentenceMismatch(TokenId),
    OverlappingTokens(TokenId),
    TokenOutsideSentence(TokenId),
    ArcHeadMissing(ArcId),
    ArcDependentMissing(ArcId),
    SelfLoop(ArcId),
    HeadlessNonRoot(ArcId),
    UnsupportedRelationNotMarked(ArcId),
    DanglingAlternativeGroup(ArcId),
    AlternativeNotInGroup(ArcId),
    DegenerateAlternativeGroup(AlternativeGroupId),
    MultipleAcceptedHeads(TokenId),
    RootCount(SentenceId, usize),
    TokenUnattached(TokenId),
    Cycle(TokenId),
    EmptySupport(FactId),
    OverconfidentDiagnostic(DiagnosticId),
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValidationIssue::*;
        match self {
            BadSpan(t, e) => write!(f, "token {t}: {e}"),
            BadSentenceSpan(s, e) => write!(f, "sentence {s}: {e}"),
            SurfaceMismatch(t) => write!(f, "token {t}: surface does not match its span"),
            DanglingToken(s, t) => write!(f, "sentence {s} lists missing token {t}"),
            OrdinalMismatch(t) => write!(f, "token {t}: ordinal disagrees with position"),
            SentenceMismatch(t) => write!(f, "token {t}: sentence back-reference is wrong"),
            OverlappingTokens(t) => write!(f, "token {t} overlaps its predecessor"),
            TokenOutsideSentence(t) => write!(f, "token {t} falls outside its sentence span"),
            ArcHeadMissing(a) => write!(f, "arc {a}: head token does not exist"),
            ArcDependentMissing(a) => write!(f, "arc {a}: dependent token does not exist"),
            SelfLoop(a) => write!(f, "arc {a}: head equals dependent"),
            HeadlessNonRoot(a) => write!(f, "arc {a}: no head but relation is not root"),
            UnsupportedRelationNotMarked(a) => {
                write!(
                    f,
                    "arc {a}: unsupported relation without unsupported status"
                )
            }
            DanglingAlternativeGroup(a) => write!(f, "arc {a}: alternative group does not exist"),
            AlternativeNotInGroup(a) => write!(f, "arc {a}: not listed in its alternative group"),
            DegenerateAlternativeGroup(g) => {
                write!(f, "alternative group {g}: fewer than two members")
            }
            MultipleAcceptedHeads(t) => write!(f, "token {t} has more than one accepted head"),
            RootCount(s, n) => write!(f, "sentence {s}: expected exactly one root, found {n}"),
            TokenUnattached(t) => write!(f, "token {t} has no arc of any status"),
            Cycle(t) => write!(f, "token {t} participates in a head cycle"),
            EmptySupport(fact) => write!(f, "{fact}: derived without any support"),
            OverconfidentDiagnostic(d) => {
                write!(
                    f,
                    "diagnostic {d} claims more certainty than its supports allow"
                )
            }
        }
    }
}
