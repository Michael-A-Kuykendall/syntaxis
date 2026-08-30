//! Support tracking and retraction.
//!
//! Removing a source retracts exactly the derived items that
//! depend on it, transitively, with no manual downstream cleanup. That is a
//! reverse index problem, not a rule problem, so it lives in `parser-core` and
//! every rule layer gets it for free.
//!
//! The cascade is *conservative*: a fact is removed when any of its sources is
//! removed. Facts that could be re-derived from other evidence are removed and
//! must be re-asserted by recomputation. This is the safe direction — the unsafe
//! direction is keeping a fact whose justification no longer exists.

use crate::support::{FactId, SourceRef, SupportSet};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default, Debug)]
pub struct FactGraph {
    supports: BTreeMap<FactId, SupportSet>,
    reverse: BTreeMap<SourceRef, BTreeSet<FactId>>,
}

/// What one retraction removed, in deterministic order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RetractionReport {
    pub retracted: SourceRef,
    /// Every fact removed, sorted. Includes facts removed transitively.
    pub removed: Vec<FactId>,
}

impl RetractionReport {
    pub fn is_empty(&self) -> bool {
        self.removed.is_empty()
    }
    pub fn contains(&self, fact: FactId) -> bool {
        self.removed.contains(&fact)
    }
}

impl FactGraph {
    pub fn new() -> Self {
        FactGraph::default()
    }

    /// Record a derived fact and its justification.
    ///
    /// Re-asserting the same fact replaces its support and reindexes cleanly,
    /// so recomputation is idempotent.
    pub fn assert_fact(&mut self, fact: FactId, support: SupportSet) {
        if self.supports.contains_key(&fact) {
            self.unindex(fact);
        }
        for source in &support.sources {
            self.reverse.entry(source.clone()).or_default().insert(fact);
        }
        self.supports.insert(fact, support);
    }

    fn unindex(&mut self, fact: FactId) {
        if let Some(support) = self.supports.get(&fact) {
            let sources = support.sources.clone();
            for source in sources {
                if let Some(set) = self.reverse.get_mut(&source) {
                    set.remove(&fact);
                    if set.is_empty() {
                        self.reverse.remove(&source);
                    }
                }
            }
        }
    }

    pub fn support_of(&self, fact: FactId) -> Option<&SupportSet> {
        self.supports.get(&fact)
    }

    pub fn contains(&self, fact: FactId) -> bool {
        self.supports.contains_key(&fact)
    }

    pub fn len(&self) -> usize {
        self.supports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.supports.is_empty()
    }

    pub fn facts(&self) -> impl Iterator<Item = (&FactId, &SupportSet)> {
        self.supports.iter()
    }

    /// Facts directly justified by `source`.
    pub fn dependents(&self, source: &SourceRef) -> Vec<FactId> {
        self.reverse
            .get(source)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Facts justified by `source` directly or transitively.
    pub fn transitive_dependents(&self, source: &SourceRef) -> Vec<FactId> {
        let mut removed: BTreeSet<FactId> = BTreeSet::new();
        let mut frontier: BTreeSet<SourceRef> = BTreeSet::new();
        frontier.insert(source.clone());
        let mut seen_sources: BTreeSet<SourceRef> = BTreeSet::new();

        while let Some(current) = frontier.iter().next().cloned() {
            frontier.remove(&current);
            if !seen_sources.insert(current.clone()) {
                continue;
            }
            for fact in self.dependents(&current) {
                if removed.insert(fact) {
                    frontier.insert(fact.as_source());
                }
            }
        }
        removed.into_iter().collect()
    }

    /// Remove `source` and everything that rests on it.
    ///
    /// Deterministic: the removed list is sorted, and the traversal order does
    /// not affect the result set.
    pub fn retract(&mut self, source: &SourceRef) -> RetractionReport {
        let removed = self.transitive_dependents(source);
        for fact in &removed {
            self.unindex(*fact);
            self.supports.remove(fact);
        }
        // The retracted source may itself have been a fact.
        if let Some(fact) = source.as_fact() {
            if self.supports.contains_key(&fact) {
                self.unindex(fact);
                self.supports.remove(&fact);
            }
        }
        self.reverse.remove(source);
        RetractionReport {
            retracted: source.clone(),
            removed,
        }
    }

    /// One derivation step, for human-facing explanations.
    pub fn explain(&self, fact: FactId) -> Option<Explanation> {
        let support = self.supports.get(&fact)?;
        Some(Explanation {
            fact,
            rule: support.rule.as_str().to_string(),
            rule_pack: support.rule_pack.as_str().to_string(),
            derivation: support.derivation.as_str().to_string(),
            sources: support.sources.clone(),
        })
    }

    /// Full derivation chain for a fact, deepest-last, deduplicated and sorted
    /// at each level so the output is stable.
    pub fn explain_deep(&self, fact: FactId) -> Vec<Explanation> {
        let mut out = Vec::new();
        let mut seen: BTreeSet<FactId> = BTreeSet::new();
        let mut frontier: BTreeSet<FactId> = BTreeSet::new();
        frontier.insert(fact);
        while let Some(current) = frontier.iter().copied().next() {
            frontier.remove(&current);
            if !seen.insert(current) {
                continue;
            }
            if let Some(explanation) = self.explain(current) {
                for source in &explanation.sources {
                    if let Some(f) = source.as_fact() {
                        if !seen.contains(&f) {
                            frontier.insert(f);
                        }
                    }
                }
                out.push(explanation);
            }
        }
        out
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Explanation {
    pub fact: FactId,
    pub rule: String,
    pub rule_pack: String,
    pub derivation: String,
    pub sources: Vec<SourceRef>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::*;
    use crate::support::DerivationKind;

    fn support(sources: Vec<SourceRef>) -> SupportSet {
        SupportSet::new(
            RuleId::new("TEST"),
            RulePackId::new("test@0.1.0"),
            DerivationKind::Contextual,
            sources,
        )
    }

    /// analysis(t1) -> arc(a0) -> diagnostic(g0)
    fn three_level() -> FactGraph {
        let mut g = FactGraph::new();
        g.assert_fact(
            FactId::TokenAnalysis(TokenId(1)),
            support(vec![SourceRef::Token(TokenId(1))]),
        );
        g.assert_fact(
            FactId::TokenAnalysis(TokenId(2)),
            support(vec![SourceRef::Token(TokenId(2))]),
        );
        g.assert_fact(
            FactId::Arc(ArcId(0)),
            support(vec![
                SourceRef::TokenAnalysis(TokenId(1)),
                SourceRef::TokenAnalysis(TokenId(2)),
            ]),
        );
        g.assert_fact(
            FactId::Diagnostic(DiagnosticId(0)),
            support(vec![
                SourceRef::Arc(ArcId(0)),
                SourceRef::TokenAnalysis(TokenId(2)),
            ]),
        );
        g
    }

    #[test]
    fn retraction_cascades_transitively() {
        let mut g = three_level();
        let report = g.retract(&SourceRef::TokenAnalysis(TokenId(1)));
        assert_eq!(
            report.removed,
            vec![FactId::Arc(ArcId(0)), FactId::Diagnostic(DiagnosticId(0))]
        );
        assert!(!g.contains(FactId::Arc(ArcId(0))));
        assert!(!g.contains(FactId::Diagnostic(DiagnosticId(0))));
        // Independent facts survive.
        assert!(g.contains(FactId::TokenAnalysis(TokenId(2))));
    }

    #[test]
    fn retraction_leaves_unrelated_facts_alone() {
        let mut g = three_level();
        g.assert_fact(
            FactId::Arc(ArcId(9)),
            support(vec![SourceRef::Token(TokenId(7))]),
        );
        let report = g.retract(&SourceRef::TokenAnalysis(TokenId(1)));
        assert!(!report.contains(FactId::Arc(ArcId(9))));
        assert!(g.contains(FactId::Arc(ArcId(9))));
    }

    #[test]
    fn retracting_an_arc_keeps_sibling_alternatives() {
        let mut g = FactGraph::new();
        // Two competing attachments over the same tokens.
        g.assert_fact(
            FactId::Arc(ArcId(0)),
            support(vec![SourceRef::Token(TokenId(1))]),
        );
        g.assert_fact(
            FactId::Arc(ArcId(1)),
            support(vec![SourceRef::Token(TokenId(1))]),
        );
        g.assert_fact(
            FactId::Diagnostic(DiagnosticId(0)),
            support(vec![SourceRef::Arc(ArcId(0))]),
        );
        let report = g.retract(&SourceRef::Arc(ArcId(0)));
        assert_eq!(report.removed, vec![FactId::Diagnostic(DiagnosticId(0))]);
        assert!(g.contains(FactId::Arc(ArcId(1))));
    }

    #[test]
    fn reassertion_is_idempotent() {
        let mut g = three_level();
        let before = g.len();
        g.assert_fact(
            FactId::Arc(ArcId(0)),
            support(vec![SourceRef::TokenAnalysis(TokenId(1))]),
        );
        assert_eq!(g.len(), before);
        // Reindexed: t2 no longer supports the arc.
        assert!(!g
            .dependents(&SourceRef::TokenAnalysis(TokenId(2)))
            .contains(&FactId::Arc(ArcId(0))));
    }

    #[test]
    fn cycles_terminate() {
        let mut g = FactGraph::new();
        g.assert_fact(
            FactId::Arc(ArcId(0)),
            support(vec![SourceRef::Arc(ArcId(1))]),
        );
        g.assert_fact(
            FactId::Arc(ArcId(1)),
            support(vec![SourceRef::Arc(ArcId(0))]),
        );
        let report = g.retract(&SourceRef::Arc(ArcId(0)));
        assert!(report.contains(FactId::Arc(ArcId(1))));
        assert!(g.is_empty());
    }

    #[test]
    fn retraction_report_is_order_independent() {
        let mut a = three_level();
        let mut b = FactGraph::new();
        // Same facts, asserted in reverse order.
        b.assert_fact(
            FactId::Diagnostic(DiagnosticId(0)),
            support(vec![
                SourceRef::Arc(ArcId(0)),
                SourceRef::TokenAnalysis(TokenId(2)),
            ]),
        );
        b.assert_fact(
            FactId::Arc(ArcId(0)),
            support(vec![
                SourceRef::TokenAnalysis(TokenId(2)),
                SourceRef::TokenAnalysis(TokenId(1)),
            ]),
        );
        b.assert_fact(
            FactId::TokenAnalysis(TokenId(2)),
            support(vec![SourceRef::Token(TokenId(2))]),
        );
        b.assert_fact(
            FactId::TokenAnalysis(TokenId(1)),
            support(vec![SourceRef::Token(TokenId(1))]),
        );
        assert_eq!(
            a.retract(&SourceRef::TokenAnalysis(TokenId(1))).removed,
            b.retract(&SourceRef::TokenAnalysis(TokenId(1))).removed
        );
    }
}
