//! Deterministic lexical and contextual English analysis.
//!
//! The current scope starts with a small, auditable lexicon and suffix rules.
//! This is not a statistical tagger: an unrecognized token remains `Unknown`
//! instead of receiving an unsupported guess. The dependency layer can use
//! the explicit lexical facts and their provenance to decide what it can
//! safely attach.

use crate::parser_core;
use crate::rulepack::RulePack;
use parser_core::ids::{RuleId, SentenceId};
use parser_core::model::{
    DetKind, Morphology, Number, Person, Pos, PronKind, Tense, Token, TokenAnalysis, UPos, VerbForm,
};
use parser_core::support::{DerivationKind, SourceRef, SupportSet};

const LEXICAL_RULE: &str = "POS.LEXICAL.V1";
const PUNCT_RULE: &str = "POS.PUNCT.V1";
const SUFFIX_RULE: &str = "POS.SUFFIX.V1";
const UNKNOWN_RULE: &str = "POS.UNKNOWN.V1";

pub fn analyze_token(token: &Token, sentence: SentenceId, pack: &RulePack) -> TokenAnalysis {
    let (pos, lemma, morphology, rule) = classify(token);
    let support = SupportSet::new(
        RuleId::new(rule),
        pack.id.clone(),
        if rule == UNKNOWN_RULE {
            DerivationKind::Contextual
        } else {
            DerivationKind::LexicalLookup
        },
        vec![
            SourceRef::Text(token.span),
            SourceRef::Sentence(sentence),
            SourceRef::Token(token.id),
        ],
    );
    TokenAnalysis {
        token: token.id,
        pos,
        upos: upos_for(pos),
        lemma,
        morphology,
        unmapped_features: Vec::new(),
        unmapped_misc: Vec::new(),
        support,
    }
}

fn classify(token: &Token) -> (Pos, String, Morphology, &'static str) {
    let word = token.normalized.as_str();
    if let Some(pos) = punctuation_pos(token.surface.as_str()) {
        return (
            pos,
            token.surface.clone(),
            Morphology::default(),
            PUNCT_RULE,
        );
    }

    let mut morphology = Morphology::default();
    let (pos, lemma) = match word {
        "a" | "an" => {
            morphology.det_kind = DetKind::Article;
            (Pos::DT, word.to_string())
        }
        "the" => {
            morphology.det_kind = DetKind::Article;
            (Pos::DT, word.to_string())
        }
        "this" | "that" | "these" | "those" => {
            morphology.det_kind = DetKind::Demonstrative;
            (Pos::DT, word.to_string())
        }
        "each" | "every" => {
            morphology.det_kind = DetKind::Total;
            morphology.number = Number::Sing;
            (Pos::DT, word.to_string())
        }
        "all" | "some" | "any" | "no" => {
            morphology.det_kind = DetKind::Quantifier;
            (Pos::DT, word.to_string())
        }
        "many" | "few" | "several" => {
            morphology.det_kind = DetKind::Quantifier;
            (Pos::JJ, word.to_string())
        }
        "i" | "me" | "we" | "us" | "you" | "he" | "him" | "she" | "her" | "it" | "they"
        | "them" => {
            morphology.pron_kind = PronKind::Personal;
            morphology.person = if matches!(word, "i" | "me" | "we" | "us") {
                Person::First
            } else if matches!(word, "you") {
                Person::Second
            } else {
                Person::Third
            };
            morphology.number = if matches!(word, "we" | "us" | "they" | "them") {
                Number::Plur
            } else {
                Number::Sing
            };
            (Pos::PRP, word.to_string())
        }
        "there" => {
            morphology.pron_kind = PronKind::Expletive;
            (Pos::EX, word.to_string())
        }
        "what" | "which" | "who" | "whom" => {
            morphology.pron_kind = PronKind::Interrogative;
            (Pos::WP, word.to_string())
        }
        "and" | "or" | "but" => (Pos::CC, word.to_string()),
        "of" | "in" | "on" | "at" | "for" | "from" | "with" | "by" | "about" | "as" => {
            (Pos::IN, word.to_string())
        }
        "to" => (Pos::TO, word.to_string()),
        "not" | "never" => (Pos::RB, word.to_string()),
        "is" => {
            morphology.number = Number::Sing;
            morphology.person = Person::Third;
            morphology.tense = Tense::Pres;
            morphology.verb_form = VerbForm::Fin;
            (Pos::VBZ, "be".to_string())
        }
        "are" | "am" => {
            morphology.number = if word == "am" {
                Number::Sing
            } else {
                Number::Plur
            };
            // `are` is used with second person and with plural subjects.
            // Person stays unknown so agreement cannot invent a unique person.
            if word == "am" {
                morphology.person = Person::First;
            }
            morphology.tense = Tense::Pres;
            morphology.verb_form = VerbForm::Fin;
            (Pos::VBP, "be".to_string())
        }
        "was" | "were" => {
            morphology.number = if word == "was" {
                Number::Sing
            } else {
                Number::Plur
            };
            morphology.tense = Tense::Past;
            morphology.verb_form = VerbForm::Fin;
            (Pos::VBD, "be".to_string())
        }
        "be" | "been" | "being" => {
            morphology.verb_form = if word == "be" {
                VerbForm::Inf
            } else if word == "been" {
                VerbForm::Part
            } else {
                VerbForm::Ger
            };
            (Pos::VB, "be".to_string())
        }
        "has" => {
            morphology.number = Number::Sing;
            morphology.person = Person::Third;
            morphology.tense = Tense::Pres;
            morphology.verb_form = VerbForm::Fin;
            (Pos::VBZ, "have".to_string())
        }
        "have" | "do" => {
            if word == "have" {
                morphology.number = Number::Plur;
            }
            morphology.verb_form = VerbForm::Fin;
            morphology.tense = Tense::Pres;
            (Pos::VBP, word.to_string())
        }
        "go" | "run" | "send" | "think" | "sleep" | "agree" => {
            morphology.verb_form = VerbForm::Fin;
            (Pos::VB, word.to_string())
        }
        "did" => {
            morphology.verb_form = VerbForm::Fin;
            morphology.tense = Tense::Past;
            (Pos::VBD, "do".to_string())
        }
        "can" | "could" | "may" | "might" | "must" | "shall" | "should" | "will" | "would" => {
            (Pos::MD, word.to_string())
        }
        "cat" | "book" | "reason" | "student" | "students" | "smith" | "example" | "home"
        | "case" | "reasons" | "people" => {
            let plural = (word.ends_with('s') && !matches!(word, "is")) || word == "people";
            morphology.number = if plural { Number::Plur } else { Number::Sing };
            (
                if plural { Pos::NNS } else { Pos::NN },
                word.trim_end_matches('s').to_string(),
            )
        }
        _ if word.ends_with('s') && word.len() > 2 => {
            morphology.number = Number::Plur;
            (Pos::NNS, word.trim_end_matches('s').to_string())
        }
        _ if word.ends_with("ing") && word.len() > 4 => {
            morphology.verb_form = VerbForm::Ger;
            morphology.tense = Tense::Pres;
            (Pos::VBG, word.to_string())
        }
        _ if word.ends_with("ed") && word.len() > 3 => {
            morphology.verb_form = VerbForm::Part;
            morphology.tense = Tense::Past;
            (Pos::VBN, word.to_string())
        }
        _ => return (Pos::Unknown, word.to_string(), morphology, UNKNOWN_RULE),
    };
    let rule = if matches!(pos, Pos::NNS | Pos::VBG | Pos::VBN) {
        SUFFIX_RULE
    } else {
        LEXICAL_RULE
    };
    (pos, lemma, morphology, rule)
}

fn punctuation_pos(surface: &str) -> Option<Pos> {
    Some(match surface {
        "." | "!" | "?" | "..." => Pos::PunctSent,
        "," => Pos::PunctComma,
        ":" | ";" => Pos::PunctColon,
        "(" | "[" | "{" => Pos::PunctLeftBracket,
        ")" | "]" | "}" => Pos::PunctRightBracket,
        "\"" | "'" | "\u{201c}" | "\u{201d}" | "\u{2018}" | "\u{2019}" => Pos::PunctOther,
        "-" => Pos::PunctHyph,
        _ if surface.chars().count() == 1 && !surface.chars().next()?.is_alphanumeric() => {
            Pos::PunctOther
        }
        _ => return None,
    })
}

fn upos_for(pos: Pos) -> UPos {
    use Pos::*;
    match pos {
        CC => UPos::CCONJ,
        CD => UPos::NUM,
        DT | PDT | WDT => UPos::DET,
        EX | PRP | PRPS | WP | WPS => UPos::PRON,
        FW | LS | Unknown => UPos::X,
        IN => UPos::ADP,
        JJ | JJR | JJS => UPos::ADJ,
        MD => UPos::AUX,
        NN | NNS => UPos::NOUN,
        NNP | NNPS => UPos::PROPN,
        POS | TO => UPos::PART,
        RB | RBR | RBS | WRB => UPos::ADV,
        RP => UPos::ADP,
        SYM => UPos::SYM,
        UH => UPos::INTJ,
        VB | VBD | VBG | VBN | VBP | VBZ => UPos::VERB,
        PunctSent | PunctComma | PunctColon | PunctLeftBracket | PunctRightBracket
        | PunctOpenQuote | PunctCloseQuote | PunctHyph | PunctOther => UPos::PUNCT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::analyze;

    #[test]
    fn assigns_motivating_morphology() {
        let pack = RulePack::builtin().unwrap();
        let analysis = analyze(
            "The cat are sleeping. There is many reasons. Each of the students have a book.",
            &pack,
        );
        let values: Vec<(&str, Pos, Number)> = analysis
            .token_analyses
            .values()
            .filter_map(|a| {
                analysis
                    .tokens
                    .get(&a.token)
                    .map(|t| (t.surface.as_str(), a.pos, a.morphology.number))
            })
            .collect();
        assert!(values.contains(&("cat", Pos::NN, Number::Sing)));
        assert!(values.contains(&("are", Pos::VBP, Number::Plur)));
        assert!(values.contains(&("students", Pos::NNS, Number::Plur)));
        assert!(values.contains(&("There", Pos::EX, Number::Unknown)));
    }

    #[test]
    fn are_does_not_claim_a_unique_person() {
        let pack = RulePack::builtin().unwrap();
        let analysis = analyze("You are sleeping.", &pack);
        let are = analysis
            .token_analyses
            .values()
            .find(|item| analysis.surface_of(item.token) == Some("are"))
            .unwrap();
        assert!(!are.morphology.person.is_known());
    }

    #[test]
    fn have_does_not_claim_a_unique_person() {
        let pack = RulePack::builtin().unwrap();
        let analysis = analyze("I have sleeping.", &pack);
        let have = analysis
            .token_analyses
            .values()
            .find(|item| analysis.surface_of(item.token) == Some("have"))
            .unwrap();
        assert!(!have.morphology.person.is_known());
    }

    #[test]
    fn unknown_is_explicit_and_provenanced() {
        let pack = RulePack::builtin().unwrap();
        let analysis = analyze("Blorf.", &pack);
        let fact = analysis.token_analyses.values().next().unwrap();
        assert_eq!(fact.pos, Pos::Unknown);
        assert_eq!(fact.upos, UPos::X);
        assert_eq!(fact.support.rule.as_str(), UNKNOWN_RULE);
        assert!(!fact.support.sources.is_empty());
    }
}
