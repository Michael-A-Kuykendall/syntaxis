//! First-gate grammar diagnostics over resolved dependency facts.
//!
//! Diagnostics are projections of the existing analysis. This module never
//! re-tokenizes text or invents a second parse.

use crate::parser_core;
use crate::rulepack::RulePack;
use parser_core::analysis::Analysis;
use parser_core::ids::{ArcId, MessageKey, RuleId, SentenceId, TokenId};
use parser_core::model::{
    ArcStatus, Certainty, DependencyArc, DiagnosticKind, GrammarDiagnostic, Person, Relation,
    Replacement, VerbForm,
};
use parser_core::support::{DerivationKind, SourceRef, SupportSet};

const AGREEMENT_RULE: &str = "GRAMMAR.AGREEMENT.SUBJECT_VERB.V1";
const PERSON_RULE: &str = "GRAMMAR.AGREEMENT.SUBJECT_VERB_PERSON.V1";
const PERSON_MESSAGE: &str = "GRAMMAR.AGREEMENT.SUBJECT_VERB_PERSON";
const COORDINATION_RULE: &str = "GRAMMAR.AGREEMENT.COORDINATION.V1";
const COORDINATION_MESSAGE: &str = "GRAMMAR.AGREEMENT.COORDINATION";
const PLACEMENT_RULE: &str = "GRAMMAR.PLACEMENT.NEGATION.V1";
const PLACEMENT_MESSAGE: &str = "GRAMMAR.PLACEMENT.NEGATION";

pub fn diagnose(analysis: &mut Analysis, pack: &RulePack) {
    let sentence_ids = analysis.document.sentences.clone();
    for sentence in sentence_ids {
        diagnose_sentence(analysis, sentence, pack);
    }
}

fn diagnose_sentence(analysis: &mut Analysis, sentence: SentenceId, pack: &RulePack) {
    let arcs: Vec<_> = analysis
        .arcs
        .values()
        .filter(|arc| arc.sentence == sentence)
        .cloned()
        .collect();

    for subject in accepted(&arcs).filter(|arc| arc.relation == Relation::Nsubj) {
        let Some(head) = subject.head else { continue };
        let finite = finite_head(analysis, &arcs, head).unwrap_or(head);
        if let Some((subject_number, finite_number)) = numbers(analysis, subject.dependent, finite)
        {
            if subject_number != finite_number {
                add_agreement_diagnostic(
                    analysis,
                    pack,
                    sentence,
                    subject.dependent,
                    finite,
                    [subject.id],
                );
            }
        }
        if let Some((subject_person, finite_person)) = persons(analysis, subject.dependent, finite)
        {
            if subject_person != finite_person {
                add_diagnostic(
                    analysis,
                    pack,
                    sentence,
                    subject.dependent,
                    finite,
                    DiagnosticKind::Agreement,
                    PERSON_MESSAGE,
                    PERSON_RULE,
                    [subject.id],
                );
            }
        }
    }

    for copula in accepted(&arcs).filter(|arc| arc.relation == Relation::Cop) {
        let Some(head) = copula.head else { continue };
        if let Some((copula_number, predicate_number)) = numbers(analysis, copula.dependent, head) {
            if copula_number != predicate_number {
                add_agreement_diagnostic(
                    analysis,
                    pack,
                    sentence,
                    copula.dependent,
                    head,
                    [copula.id],
                );
            }
        }
    }

    for determiner in accepted(&arcs).filter(|arc| arc.relation == Relation::Det) {
        let Some(head) = determiner.head else {
            continue;
        };
        let Some(det) = analysis.token_analyses.get(&determiner.dependent) else {
            continue;
        };
        let Some(noun) = analysis.token_analyses.get(&head) else {
            continue;
        };
        let Some(surface) = analysis.surface_of(determiner.dependent) else {
            continue;
        };
        if det.morphology.det_kind == parser_core::model::DetKind::Article
            && (surface.eq_ignore_ascii_case("a") || surface.eq_ignore_ascii_case("an"))
            && noun.morphology.number == parser_core::model::Number::Plur
        {
            add_diagnostic(
                analysis,
                pack,
                sentence,
                determiner.dependent,
                head,
                DiagnosticKind::Determiner,
                "GRAMMAR.DETERMINER.ARTICLE_NUMBER",
                "GRAMMAR.DETERMINER.ARTICLE_NUMBER.V1",
                [determiner.id],
            );
        }
    }

    for auxiliary in accepted(&arcs).filter(|arc| arc.relation == Relation::Aux) {
        let Some(head) = auxiliary.head else { continue };
        let Some(aux) = analysis.token_analyses.get(&auxiliary.dependent) else {
            continue;
        };
        let Some(verb) = analysis.token_analyses.get(&head) else {
            continue;
        };
        if aux.lemma == "have"
            && verb.morphology.verb_form.is_known()
            && verb.morphology.verb_form != VerbForm::Part
        {
            add_diagnostic(
                analysis,
                pack,
                sentence,
                auxiliary.dependent,
                head,
                DiagnosticKind::VerbForm,
                "GRAMMAR.VERB_FORM.HAVE_PARTICIPLE",
                "GRAMMAR.VERB_FORM.HAVE_PARTICIPLE.V1",
                [auxiliary.id],
            );
        }
    }

    for negation in accepted(&arcs).filter(|arc| arc.relation == Relation::Neg) {
        let Some(head) = negation.head else { continue };
        let Some(surface) = analysis.surface_of(negation.dependent) else {
            continue;
        };
        // `never` is grammatical without an auxiliary host. Only `not` is in
        // the do-support placement contract.
        if !surface.eq_ignore_ascii_case("not") {
            continue;
        }
        if has_accepted_verbal_host(&arcs, head) {
            continue;
        }
        add_diagnostic(
            analysis,
            pack,
            sentence,
            negation.dependent,
            head,
            DiagnosticKind::Placement,
            PLACEMENT_MESSAGE,
            PLACEMENT_RULE,
            [negation.id],
        );
    }

    for coordination in accepted(&arcs).filter(|arc| arc.relation == Relation::Conj) {
        let Some(head) = coordination.head else {
            continue;
        };
        if let Some((left, right)) = numbers(analysis, head, coordination.dependent) {
            if left != right {
                add_diagnostic(
                    analysis,
                    pack,
                    sentence,
                    head,
                    coordination.dependent,
                    DiagnosticKind::Agreement,
                    COORDINATION_MESSAGE,
                    COORDINATION_RULE,
                    [coordination.id],
                );
            }
        }
    }
}

fn accepted(arcs: &[DependencyArc]) -> impl Iterator<Item = &DependencyArc> {
    arcs.iter().filter(|arc| arc.status == ArcStatus::Accepted)
}

fn has_accepted_verbal_host(arcs: &[DependencyArc], head: TokenId) -> bool {
    accepted(arcs)
        .any(|arc| arc.head == Some(head) && matches!(arc.relation, Relation::Aux | Relation::Cop))
}

fn finite_head(analysis: &Analysis, arcs: &[DependencyArc], head: TokenId) -> Option<TokenId> {
    let head_analysis = analysis.token_analyses.get(&head)?;
    if head_analysis.morphology.number.is_known() || head_analysis.morphology.person.is_known() {
        return Some(head);
    }
    accepted(arcs)
        .find(|arc| arc.head == Some(head) && arc.relation == Relation::Aux)
        .map(|arc| arc.dependent)
}

fn numbers(
    analysis: &Analysis,
    left: TokenId,
    right: TokenId,
) -> Option<(parser_core::model::Number, parser_core::model::Number)> {
    let left = analysis.token_analyses.get(&left)?.morphology.number;
    let right = analysis.token_analyses.get(&right)?.morphology.number;
    if left.is_known() && right.is_known() {
        Some((left, right))
    } else {
        None
    }
}

fn persons(analysis: &Analysis, left: TokenId, right: TokenId) -> Option<(Person, Person)> {
    let left = analysis.token_analyses.get(&left)?.morphology.person;
    let right = analysis.token_analyses.get(&right)?.morphology.person;
    if left.is_known() && right.is_known() {
        Some((left, right))
    } else {
        None
    }
}

fn add_agreement_diagnostic<const N: usize>(
    analysis: &mut Analysis,
    pack: &RulePack,
    sentence: SentenceId,
    left: TokenId,
    right: TokenId,
    arc_ids: [ArcId; N],
) {
    add_diagnostic(
        analysis,
        pack,
        sentence,
        left,
        right,
        DiagnosticKind::Agreement,
        "GRAMMAR.AGREEMENT.SUBJECT_VERB",
        AGREEMENT_RULE,
        arc_ids,
    );
}

#[allow(clippy::too_many_arguments)]
fn add_diagnostic<const N: usize>(
    analysis: &mut Analysis,
    pack: &RulePack,
    sentence: SentenceId,
    left: TokenId,
    right: TokenId,
    kind: DiagnosticKind,
    message: &str,
    rule: &str,
    arc_ids: [ArcId; N],
) {
    let Some(left_token) = analysis.tokens.get(&left) else {
        return;
    };
    let Some(right_token) = analysis.tokens.get(&right) else {
        return;
    };
    let mut sources = vec![
        SourceRef::Sentence(sentence),
        SourceRef::TokenAnalysis(left),
        SourceRef::TokenAnalysis(right),
    ];
    sources.extend(arc_ids.into_iter().map(SourceRef::Arc));
    let diagnostic = GrammarDiagnostic {
        id: analysis.next_diagnostic_id(),
        sentence,
        kind,
        span: left_token.span.cover(&right_token.span),
        message_key: MessageKey::new(message),
        certainty: Certainty::Definite,
        replacements: Vec::<Replacement>::new(),
        support: SupportSet::new(
            RuleId::new(rule),
            pack.id.clone(),
            DerivationKind::GrammarRule,
            sources,
        ),
    };
    analysis.add_diagnostic(diagnostic);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::analyze;

    fn pack() -> RulePack {
        RulePack::builtin().unwrap()
    }

    fn keys(analysis: &Analysis) -> Vec<&str> {
        analysis
            .diagnostics
            .values()
            .map(|diagnostic| diagnostic.message_key.as_str())
            .collect()
    }

    fn has_key(analysis: &Analysis, key: &str) -> bool {
        keys(analysis).contains(&key)
    }

    #[test]
    fn diagnoses_the_three_motivating_agreement_errors() {
        let analysis = analyze(
            "The cat are sleeping. There is many reasons. Each of the students have a book.",
            &pack(),
        );
        assert_eq!(analysis.diagnostics.len(), 3);
        assert!(analysis
            .diagnostics
            .values()
            .all(|diagnostic| diagnostic.kind == DiagnosticKind::Agreement));
    }

    #[test]
    fn diagnoses_determiner_and_verb_form_mismatches() {
        let determiner = analyze("A students run.", &pack());
        assert!(determiner
            .diagnostics
            .values()
            .any(|diagnostic| diagnostic.kind == DiagnosticKind::Determiner));

        let verb_form = analyze("They have sleeping.", &pack());
        assert!(verb_form
            .diagnostics
            .values()
            .any(|diagnostic| diagnostic.kind == DiagnosticKind::VerbForm));
    }

    #[test]
    fn person_agreement_requires_known_person_on_both_sides() {
        assert!(pack().rule(PERSON_RULE).is_ok());

        let mismatch = analyze("I is sleeping.", &pack());
        assert!(has_key(&mismatch, PERSON_MESSAGE));
        let person = mismatch
            .diagnostics
            .values()
            .find(|diagnostic| diagnostic.message_key.as_str() == PERSON_MESSAGE)
            .unwrap();
        assert_eq!(person.kind, DiagnosticKind::Agreement);
        assert_eq!(person.support.rule.as_str(), PERSON_RULE);
        assert!(person
            .support
            .sources
            .iter()
            .any(|source| matches!(source, SourceRef::Sentence(_))));
        assert_eq!(
            person
                .support
                .sources
                .iter()
                .filter(|source| matches!(source, SourceRef::TokenAnalysis(_)))
                .count(),
            2
        );
        assert!(person
            .support
            .sources
            .iter()
            .any(|source| matches!(source, SourceRef::Arc(_))));

        let clean = analyze("He is sleeping.", &pack());
        assert!(!has_key(&clean, PERSON_MESSAGE));

        let unknown_noun = analyze("The cat is sleeping.", &pack());
        assert!(!has_key(&unknown_noun, PERSON_MESSAGE));
        let cat = unknown_noun
            .token_analyses
            .values()
            .find(|analysis| unknown_noun.surface_of(analysis.token) == Some("cat"))
            .unwrap();
        assert!(!cat.morphology.person.is_known());
    }

    #[test]
    fn person_diagnostic_retracts_with_subject_arc() {
        let mut analysis = analyze("I is sleeping.", &pack());
        let diagnostic = analysis
            .diagnostics
            .values()
            .find(|item| item.message_key.as_str() == PERSON_MESSAGE)
            .unwrap()
            .id;
        let subject_arc = analysis
            .arcs
            .values()
            .find(|arc| arc.relation == Relation::Nsubj)
            .unwrap()
            .id;
        analysis.retract(&SourceRef::Arc(subject_arc));
        assert!(!analysis.diagnostics.contains_key(&diagnostic));
    }

    #[test]
    fn do_support_is_an_accepted_host_for_not() {
        let hosted = analyze("They do not sleep.", &pack());
        assert!(!has_key(&hosted, PLACEMENT_MESSAGE));
        assert!(hosted.arcs.values().any(|arc| {
            arc.relation == Relation::Aux
                && arc.status == ArcStatus::Accepted
                && hosted
                    .token_analyses
                    .get(&arc.dependent)
                    .is_some_and(|analysis| analysis.lemma == "do")
        }));

        let hostless = analyze("They not sleep.", &pack());
        assert!(has_key(&hostless, PLACEMENT_MESSAGE));

        let never = analyze("They never sleep.", &pack());
        assert!(!has_key(&never, PLACEMENT_MESSAGE));
    }

    #[test]
    fn negation_placement_retracts_with_negation_arc() {
        let mut analysis = analyze("They not sleep.", &pack());
        let diagnostic = analysis
            .diagnostics
            .values()
            .find(|item| item.message_key.as_str() == PLACEMENT_MESSAGE)
            .unwrap()
            .id;
        let negation_arc = analysis
            .arcs
            .values()
            .find(|arc| arc.relation == Relation::Neg)
            .unwrap()
            .id;
        analysis.retract(&SourceRef::Arc(negation_arc));
        assert!(!analysis.diagnostics.contains_key(&diagnostic));
    }

    #[test]
    fn coordination_agreement_uses_only_resolved_conj_arcs() {
        let mismatch = analyze("The cat and dogs sleep.", &pack());
        assert!(has_key(&mismatch, COORDINATION_MESSAGE));

        let clean = analyze("The cats and dogs sleep.", &pack());
        assert!(!has_key(&clean, COORDINATION_MESSAGE));

        let clause_coordination = analyze("They send books and run.", &pack());
        assert!(!has_key(&clause_coordination, COORDINATION_MESSAGE));
        let conj_arcs = clause_coordination
            .arcs
            .values()
            .filter(|arc| arc.relation == Relation::Conj && arc.status == ArcStatus::Accepted)
            .count();
        assert_eq!(
            conj_arcs, 0,
            "shared-argument coordination must not be claimed as conj"
        );
    }

    #[test]
    fn coordination_diagnostic_retracts_with_conj_arc() {
        let mut analysis = analyze("The cat and dogs sleep.", &pack());
        let diagnostic = analysis
            .diagnostics
            .values()
            .find(|item| item.message_key.as_str() == COORDINATION_MESSAGE)
            .unwrap()
            .id;
        let conj_arc = analysis
            .arcs
            .values()
            .find(|arc| arc.relation == Relation::Conj)
            .unwrap()
            .id;
        analysis.retract(&SourceRef::Arc(conj_arc));
        assert!(!analysis.diagnostics.contains_key(&diagnostic));
    }

    #[test]
    fn named_coordination_limitation_fixture_stays_explicit() {
        for line in include_str!("../../fixtures/coordination_limitations.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let (_, text) = line.split_once('|').unwrap();
            let analysis = analyze(text, &pack());
            assert!(!has_key(&analysis, COORDINATION_MESSAGE));
            assert!(analysis
                .arcs
                .values()
                .all(|arc| arc.relation != Relation::Conj || arc.status != ArcStatus::Accepted));
        }
    }

    #[test]
    fn diagnostics_retract_with_supporting_arc() {
        let mut analysis = analyze("The cat are sleeping.", &pack());
        let diagnostic = analysis.diagnostics.values().next().unwrap().id;
        let subject_arc = analysis
            .arcs
            .values()
            .find(|arc| arc.relation == Relation::Nsubj)
            .unwrap()
            .id;
        analysis.retract(&SourceRef::Arc(subject_arc));
        assert!(!analysis.diagnostics.contains_key(&diagnostic));
    }

    #[test]
    fn new_constructions_are_byte_stable() {
        for text in [
            "I is sleeping.",
            "They do not sleep.",
            "They go to sleeping.",
            "The cat and dogs sleep.",
        ] {
            let first = analyze(text, &pack());
            let second = analyze(text, &pack());
            assert_eq!(first.to_canonical_json(), second.to_canonical_json());
            assert_eq!(first.digest(), second.digest());
        }
    }
}
