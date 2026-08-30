//! First-gate grammar diagnostics over resolved dependency facts.
//!
//! Diagnostics are projections of the existing analysis. This module never
//! re-tokenizes text or invents a second parse.

use crate::parser_core;
use crate::rulepack::RulePack;
use parser_core::analysis::Analysis;
use parser_core::ids::{ArcId, MessageKey, RuleId, SentenceId, TokenId};
use parser_core::model::{
    Certainty, DiagnosticKind, GrammarDiagnostic, Person, Relation, Replacement, VerbForm,
};
use parser_core::support::{DerivationKind, SourceRef, SupportSet};

const AGREEMENT_RULE: &str = "GRAMMAR.AGREEMENT.SUBJECT_VERB.V1";

pub fn diagnose(analysis: &mut Analysis, pack: &RulePack) {
    let sentence_ids = analysis.document.sentences.clone();
    for sentence in sentence_ids {
        diagnose_agreement(analysis, sentence, pack);
    }
}

fn diagnose_agreement(analysis: &mut Analysis, sentence: SentenceId, pack: &RulePack) {
    let arcs: Vec<_> = analysis
        .arcs
        .values()
        .filter(|arc| arc.sentence == sentence)
        .cloned()
        .collect();

    for subject in arcs.iter().filter(|arc| arc.relation == Relation::Nsubj) {
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
                    "GRAMMAR.AGREEMENT.SUBJECT_VERB_PERSON",
                    "GRAMMAR.AGREEMENT.SUBJECT_VERB_PERSON.V1",
                    [subject.id],
                );
            }
        }
    }

    for copula in arcs.iter().filter(|arc| arc.relation == Relation::Cop) {
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

    for determiner in arcs.iter().filter(|arc| arc.relation == Relation::Det) {
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

    for auxiliary in arcs.iter().filter(|arc| arc.relation == Relation::Aux) {
        let Some(head) = auxiliary.head else { continue };
        let Some(aux) = analysis.token_analyses.get(&auxiliary.dependent) else {
            continue;
        };
        let Some(verb) = analysis.token_analyses.get(&head) else {
            continue;
        };
        if aux.lemma == "have" && verb.morphology.verb_form != VerbForm::Part {
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

    for negation in arcs.iter().filter(|arc| arc.relation == Relation::Neg) {
        let Some(head) = negation.head else { continue };
        if !arcs
            .iter()
            .any(|arc| arc.head == Some(head) && arc.relation == Relation::Aux)
        {
            add_diagnostic(
                analysis,
                pack,
                sentence,
                negation.dependent,
                head,
                DiagnosticKind::Placement,
                "GRAMMAR.PLACEMENT.NEGATION",
                "GRAMMAR.PLACEMENT.NEGATION.V1",
                [negation.id],
            );
        }
    }

    for coordination in arcs.iter().filter(|arc| arc.relation == Relation::Conj) {
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
                    "GRAMMAR.AGREEMENT.COORDINATION",
                    "GRAMMAR.AGREEMENT.COORDINATION.V1",
                    [coordination.id],
                );
            }
        }
    }
}

fn finite_head(
    analysis: &Analysis,
    arcs: &[parser_core::model::DependencyArc],
    head: TokenId,
) -> Option<TokenId> {
    let head_analysis = analysis.token_analyses.get(&head)?;
    if head_analysis.morphology.number.is_known() {
        return Some(head);
    }
    arcs.iter()
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
    arc_ids: [parser_core::ids::ArcId; N],
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

    #[test]
    fn diagnoses_the_three_motivating_agreement_errors() {
        let pack = RulePack::builtin().unwrap();
        let analysis = analyze(
            "The cat are sleeping. There is many reasons. Each of the students have a book.",
            &pack,
        );
        assert_eq!(analysis.diagnostics.len(), 3);
        assert!(analysis
            .diagnostics
            .values()
            .all(|diagnostic| diagnostic.kind == DiagnosticKind::Agreement));
    }

    #[test]
    fn diagnoses_determiner_and_verb_form_mismatches() {
        let pack = RulePack::builtin().unwrap();
        let determiner = analyze("A students run.", &pack);
        assert!(determiner
            .diagnostics
            .values()
            .any(|diagnostic| diagnostic.kind == DiagnosticKind::Determiner));

        let verb_form = analyze("They have sleeping.", &pack);
        assert!(verb_form
            .diagnostics
            .values()
            .any(|diagnostic| diagnostic.kind == DiagnosticKind::VerbForm));
    }

    #[test]
    fn diagnostics_retract_with_supporting_arc() {
        let pack = RulePack::builtin().unwrap();
        let mut analysis = analyze("The cat are sleeping.", &pack);
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
}
