//! Bounded deterministic dependency attachment.
//!
//! This is deliberately a safe structural kernel, not a claim of complete
//! English parsing. It emits one accepted arc per token when a declared rule
//! can justify the attachment and marks the remainder unsupported.

use crate::parser_core;
use crate::rulepack::RulePack;
use parser_core::analysis::Analysis;
use parser_core::ids::{RuleId, SentenceId, TokenId};
use parser_core::model::{ArcStatus, Pos, Relation, TokenAnalysis};
use parser_core::support::{DerivationKind, SourceRef, SupportSet};
use std::collections::BTreeSet;

const ROOT_RULE: &str = "ATTACH.ROOT.V1";
const SUBJECT_RULE: &str = "ATTACH.SUBJECT.V1";
const EXPL_RULE: &str = "ATTACH.EXPLETIVE.V1";
const AUX_RULE: &str = "ATTACH.AUXILIARY.V1";
const COP_RULE: &str = "ATTACH.COPULA.V1";
const DET_RULE: &str = "ATTACH.DETERMINER.V1";
const CASE_RULE: &str = "ATTACH.CASE.V1";
const NMOD_RULE: &str = "ATTACH.NMOD.V1";
const OBJECT_RULE: &str = "ATTACH.OBJECT.V1";
const MODIFIER_RULE: &str = "ATTACH.MODIFIER.V1";
const TO_MARK_RULE: &str = "ATTACH.TO_MARK.V1";
const COORD_RULE: &str = "ATTACH.COORDINATION.V1";
const PUNCT_RULE: &str = "ATTACH.PUNCTUATION.V1";
const UNSUPPORTED_RULE: &str = "ATTACH.UNSUPPORTED.V1";

type Attachment = (TokenId, Option<TokenId>, Relation, &'static str);

pub fn parse(analysis: &mut Analysis, pack: &RulePack) {
    let sentence_ids = analysis.document.sentences.clone();
    for sentence_id in sentence_ids {
        parse_sentence(analysis, sentence_id, pack);
    }
}

fn parse_sentence(analysis: &mut Analysis, sentence_id: SentenceId, pack: &RulePack) {
    let tokens: Vec<_> = analysis
        .sentence_tokens(sentence_id)
        .into_iter()
        .cloned()
        .collect();
    let analyses: Vec<_> = tokens
        .iter()
        .filter_map(|token| analysis.token_analyses.get(&token.id).cloned())
        .collect();
    let root = choose_root(&tokens, &analyses);
    let root_ordinal = root.and_then(|root_id| {
        tokens
            .iter()
            .find(|token| token.id == root_id)
            .map(|token| token.ordinal)
    });
    let mut attached = BTreeSet::new();

    if let Some(root_id) = root {
        add_arc(
            analysis,
            pack,
            sentence_id,
            (root_id, None, Relation::Root, ROOT_RULE),
            &mut attached,
        );
    }

    for token in &tokens {
        if attached.contains(&token.id) {
            continue;
        }
        let Some(token_analysis) = analysis.token_analyses.get(&token.id).cloned() else {
            add_arc(
                analysis,
                pack,
                sentence_id,
                (token.id, root, Relation::Unsupported, UNSUPPORTED_RULE),
                &mut attached,
            );
            continue;
        };
        let prior: Vec<_> = tokens
            .iter()
            .filter(|candidate| candidate.ordinal < token.ordinal)
            .collect();
        let following: Vec<_> = tokens
            .iter()
            .filter(|candidate| candidate.ordinal > token.ordinal)
            .collect();
        let root_id = root;

        let (head, relation, rule) = if is_punctuation(token_analysis.pos) {
            (root_id, Relation::Punct, PUNCT_RULE)
        } else if token.normalized == "there" {
            (root_id, Relation::Expl, EXPL_RULE)
        } else if token_analysis.pos == Pos::DT {
            if token.normalized == "each" || token.normalized == "every" {
                (root_id, Relation::Nsubj, SUBJECT_RULE)
            } else if let Some(noun) = following
                .iter()
                .find(|candidate| is_nominal(analysis, candidate.id))
            {
                (Some(noun.id), Relation::Det, DET_RULE)
            } else {
                (root_id, Relation::Unsupported, UNSUPPORTED_RULE)
            }
        } else if token.normalized == "of" {
            if let Some(noun) = following
                .iter()
                .find(|candidate| is_nominal(analysis, candidate.id))
            {
                (Some(noun.id), Relation::Case, CASE_RULE)
            } else {
                (root_id, Relation::Unsupported, UNSUPPORTED_RULE)
            }
        } else if is_auxiliary(&token_analysis) {
            if let Some(head) = root_id {
                let relation = if token_analysis.lemma == "be" && root_is_nominal(analysis, head) {
                    Relation::Cop
                } else {
                    Relation::Aux
                };
                (
                    Some(head),
                    relation,
                    if relation == Relation::Cop {
                        COP_RULE
                    } else {
                        AUX_RULE
                    },
                )
            } else {
                (None, Relation::Unsupported, UNSUPPORTED_RULE)
            }
        } else if token_analysis.pos == Pos::TO {
            if let Some(verb) = following.iter().find(|candidate| {
                analysis
                    .token_analyses
                    .get(&candidate.id)
                    .is_some_and(|candidate| {
                        matches!(
                            candidate.pos,
                            Pos::VB | Pos::VBG | Pos::VBN | Pos::VBP | Pos::VBZ
                        )
                    })
            }) {
                (Some(verb.id), Relation::Mark, TO_MARK_RULE)
            } else {
                (root_id, Relation::Unsupported, UNSUPPORTED_RULE)
            }
        } else if token_analysis.pos == Pos::CC {
            (root_id, Relation::Cc, COORD_RULE)
        } else if token_analysis.pos == Pos::RB
            && matches!(token.normalized.as_str(), "not" | "never")
        {
            (root_id, Relation::Neg, MODIFIER_RULE)
        } else if token_analysis.pos == Pos::RB {
            (root_id, Relation::Advmod, MODIFIER_RULE)
        } else if is_nominal_analysis(&token_analysis)
            && prior
                .iter()
                .any(|candidate| matches!(candidate.normalized.as_str(), "and" | "or" | "but"))
        {
            (
                prior
                    .iter()
                    .rev()
                    .find(|candidate| is_nominal(analysis, candidate.id))
                    .map(|candidate| candidate.id),
                Relation::Conj,
                COORD_RULE,
            )
        } else if is_nominal_analysis(&token_analysis)
            && prior.iter().all(|candidate| Some(candidate.id) != root_id)
            && !prior.iter().any(|candidate| candidate.normalized == "of")
        {
            (root_id, Relation::Nsubj, SUBJECT_RULE)
        } else if is_nominal_analysis(&token_analysis)
            && prior.iter().any(|candidate| candidate.normalized == "of")
            && root_ordinal.is_some_and(|ordinal| token.ordinal < ordinal)
        {
            (
                prior
                    .iter()
                    .rev()
                    .find(|candidate| candidate.normalized == "of")
                    .and_then(|of| {
                        prior
                            .iter()
                            .find(|candidate| candidate.ordinal < of.ordinal)
                            .map(|head| head.id)
                    }),
                Relation::Nmod,
                NMOD_RULE,
            )
        } else if is_nominal_analysis(&token_analysis) && root_id.is_some() {
            (root_id, Relation::Obj, OBJECT_RULE)
        } else if token_analysis.pos == Pos::JJ {
            if let Some(noun) = following
                .iter()
                .find(|candidate| is_nominal(analysis, candidate.id))
            {
                (Some(noun.id), Relation::Amod, MODIFIER_RULE)
            } else {
                (root_id, Relation::Advmod, MODIFIER_RULE)
            }
        } else if token_analysis.pos == Pos::IN {
            (root_id, Relation::Mark, MODIFIER_RULE)
        } else {
            (root_id, Relation::Unsupported, UNSUPPORTED_RULE)
        };

        add_arc(
            analysis,
            pack,
            sentence_id,
            (token.id, head, relation, rule),
            &mut attached,
        );
    }
}

fn choose_root(
    tokens: &[parser_core::model::Token],
    analyses: &[TokenAnalysis],
) -> Option<TokenId> {
    for (index, analysis) in analyses.iter().enumerate().rev() {
        if !matches!(
            analysis.pos,
            Pos::VB | Pos::VBD | Pos::VBG | Pos::VBN | Pos::VBP | Pos::VBZ | Pos::MD
        ) {
            continue;
        }
        if analysis.lemma == "be" {
            if let Some(predicate) = analyses
                .iter()
                .skip(index + 1)
                .find(|candidate| is_nominal_analysis(candidate))
            {
                return Some(predicate.token);
            }
        }
        return Some(analysis.token);
    }
    tokens
        .iter()
        .rev()
        .find(|token| token.surface.chars().any(char::is_alphanumeric))
        .map(|token| token.id)
}

fn is_auxiliary(analysis: &TokenAnalysis) -> bool {
    matches!(
        analysis.pos,
        Pos::MD | Pos::VB | Pos::VBD | Pos::VBP | Pos::VBZ
    ) && matches!(analysis.lemma.as_str(), "be" | "have" | "do")
}

fn is_nominal(analysis: &Analysis, token: TokenId) -> bool {
    analysis
        .token_analyses
        .get(&token)
        .is_some_and(is_nominal_analysis)
}

fn is_nominal_analysis(analysis: &TokenAnalysis) -> bool {
    matches!(
        analysis.pos,
        Pos::NN | Pos::NNS | Pos::NNP | Pos::NNPS | Pos::PRP | Pos::EX | Pos::DT
    )
}

fn root_is_nominal(analysis: &Analysis, token: TokenId) -> bool {
    is_nominal(analysis, token)
}

fn is_punctuation(pos: Pos) -> bool {
    matches!(
        pos,
        Pos::PunctSent
            | Pos::PunctComma
            | Pos::PunctColon
            | Pos::PunctLeftBracket
            | Pos::PunctRightBracket
            | Pos::PunctOpenQuote
            | Pos::PunctCloseQuote
            | Pos::PunctHyph
            | Pos::PunctOther
    )
}

fn add_arc(
    analysis: &mut Analysis,
    pack: &RulePack,
    sentence: SentenceId,
    attachment: Attachment,
    attached: &mut BTreeSet<TokenId>,
) {
    let (dependent, head, relation, rule) = attachment;
    let mut sources = vec![
        SourceRef::Sentence(sentence),
        SourceRef::TokenAnalysis(dependent),
    ];
    if let Some(head) = head {
        sources.push(SourceRef::TokenAnalysis(head));
    }
    let status = if relation == Relation::Unsupported {
        ArcStatus::Unsupported
    } else {
        ArcStatus::Accepted
    };
    let raw_label = (relation == Relation::Unsupported).then(|| "dep:unsupported".to_string());
    let arc = parser_core::model::DependencyArc {
        id: analysis.next_arc_id(),
        sentence,
        head,
        dependent,
        relation,
        raw_label,
        status,
        support: SupportSet::new(
            RuleId::new(rule),
            pack.id.clone(),
            DerivationKind::Attachment,
            sources,
        ),
    };
    analysis.add_arc(arc);
    attached.insert(dependent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::analyze;

    #[test]
    fn parses_the_three_motivating_shapes() {
        let pack = RulePack::builtin().unwrap();
        let analysis = analyze(
            "The cat are sleeping. There is many reasons. Each of the students have a book.",
            &pack,
        );
        let labels: BTreeSet<&str> = analysis
            .arcs
            .values()
            .map(|arc| arc.relation.as_str())
            .collect();
        assert!(labels.contains("root"));
        assert!(labels.contains("nsubj"));
        assert!(labels.contains("aux"));
        assert!(labels.contains("expl"));
        assert!(labels.contains("case"));
        assert!(labels.contains("nmod"));
        assert!(labels.contains("obj"));
    }

    #[test]
    fn every_arc_is_supported_by_its_token_analysis() {
        let pack = RulePack::builtin().unwrap();
        let analysis = analyze("The cat are sleeping.", &pack);
        for arc in analysis.arcs.values() {
            assert!(arc
                .support
                .sources
                .contains(&SourceRef::TokenAnalysis(arc.dependent)));
        }
    }

    #[test]
    fn infinitival_to_marks_a_known_nonfinite_verb() {
        let pack = RulePack::builtin().unwrap();
        let analysis = analyze("They go to sleeping.", &pack);
        let marker = analysis
            .arcs
            .values()
            .find(|arc| arc.relation == Relation::Mark)
            .unwrap();
        assert_eq!(marker.status, ArcStatus::Accepted);
        assert_eq!(marker.support.rule.as_str(), TO_MARK_RULE);
        assert!(marker
            .support
            .sources
            .contains(&SourceRef::TokenAnalysis(marker.dependent)));
        assert!(marker
            .support
            .sources
            .contains(&SourceRef::TokenAnalysis(marker.head.unwrap())));
    }
}
