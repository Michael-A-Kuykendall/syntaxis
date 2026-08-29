//! The M0 pipeline: normalization and segmentation, then tokenization.
//!
//! The pipeline is staged internally but its durable result is one [`Analysis`]
//! (§6). Later stages — POS at M1, attachment at M1, grammar at M2 — attach to
//! this same object and consume these same token identities. They will not
//! re-tokenize, which is the whole point of §4.1.

use crate::grammar::diagnose;
use crate::parser::parse;
use crate::pos::analyze_token;
use crate::rulepack::RulePack;
use crate::segment::segment;
use crate::text::CharMap;
use crate::tokenize::tokenize_sentence;
use parser_core::analysis::Analysis;
use parser_core::ids::{DocumentId, SentenceId, TokenId};
use parser_core::model::{Document, Sentence, Token};
use parser_core::support::{DerivationKind, LexiconRef, SourceRef, SupportSet};

/// Analyse `text` with `pack`. Pure: no I/O, no clock, no randomness.
pub fn analyze(text: &str, pack: &RulePack) -> Analysis {
    analyze_with_id(text, pack, DocumentId(0))
}

pub fn analyze_with_id(text: &str, pack: &RulePack, id: DocumentId) -> Analysis {
    let document = Document {
        id,
        text: text.to_string(),
        sentences: Vec::new(),
        analysis_version: pack.analysis_version,
        rule_pack: pack.id.clone(),
        conllu_mapping_version: pack.conllu_mapping_version,
        dialect: pack.dialect.clone(),
    };
    let mut analysis = Analysis::new(document);
    let map = CharMap::new(text);

    let mut next_token = 0u32;
    // (token id, byte end) of the previous token, for `space_after`.
    let mut previous: Option<(TokenId, u32)> = None;

    for (ordinal, segmented) in segment(text, pack, &map).into_iter().enumerate() {
        let sentence_id = SentenceId(ordinal as u32);

        // A sentence rests on its text and on every abbreviation entry that
        // suppressed a break inside it: change that artifact and this sentence
        // must be recomputed.
        let mut sources = vec![SourceRef::Text(segmented.span)];
        for suppression in &segmented.suppressed {
            if let Some(entry) = &suppression.lexicon_entry {
                sources.push(SourceRef::Lexicon(LexiconRef::new(
                    "en.abbreviations",
                    entry,
                    pack.artifact_version("abbreviations"),
                )));
            }
        }
        let sentence_support = SupportSet::new(
            segmented.rule.clone(),
            pack.id.clone(),
            DerivationKind::Surface,
            sources,
        );

        let raw_tokens = tokenize_sentence(text, segmented.span, pack, &map);
        let mut token_ids = Vec::with_capacity(raw_tokens.len());

        for (index, raw) in raw_tokens.into_iter().enumerate() {
            let token_id = TokenId(next_token);
            next_token += 1;

            let mut sources = vec![SourceRef::Text(raw.span), SourceRef::Sentence(sentence_id)];
            if let Some(lexicon) = raw.lexicon.clone() {
                sources.push(SourceRef::Lexicon(lexicon));
            }

            if let Some((previous_id, previous_end)) = previous {
                if previous_end == raw.span.byte_start {
                    if let Some(token) = analysis.tokens.get_mut(&previous_id) {
                        token.space_after = false;
                    }
                }
            }

            analysis.add_token(Token {
                id: token_id,
                sentence: sentence_id,
                ordinal: index as u32,
                span: raw.span,
                surface: raw.surface,
                normalized: raw.normalized,
                // Provisionally true; corrected when the next token is seen.
                space_after: true,
                support: SupportSet::new(
                    raw.rule.clone(),
                    pack.id.clone(),
                    DerivationKind::Surface,
                    sources,
                ),
            });
            previous = Some((token_id, raw.span.byte_end));
            token_ids.push(token_id);
        }

        for token_id in token_ids {
            let token = analysis
                .tokens
                .get(&token_id)
                .expect("token was just added")
                .clone();
            analysis.add_token_analysis(analyze_token(&token, sentence_id, pack));
        }

        analysis.add_sentence(Sentence {
            id: sentence_id,
            ordinal: ordinal as u32,
            span: segmented.span,
            tokens: analysis
                .tokens
                .values()
                .filter(|token| token.sentence == sentence_id)
                .map(|token| token.id)
                .collect(),
            support: sentence_support,
        });
    }

    parse(&mut analysis, pack);
    diagnose(&mut analysis, pack);
    analysis
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser_core::support::FactId;

    fn analyse(text: &str) -> Analysis {
        let pack = RulePack::builtin().unwrap();
        analyze(text, &pack)
    }

    #[test]
    fn produces_one_snapshot_that_validates() {
        let analysis = analyse("The cat are sleeping. Each of the students have a book.");
        assert_eq!(analysis.sentences.len(), 2);
        assert_eq!(analysis.tokens.len(), 5 + 8);
        assert_eq!(analysis.validate(), vec![]);
    }

    #[test]
    fn token_ids_are_document_wide_and_ordinals_are_sentence_local() {
        let analysis = analyse("Go home. Stay here.");
        let second = analysis.sentence_tokens(SentenceId(1));
        assert_eq!(second[0].id, TokenId(3));
        assert_eq!(second[0].ordinal, 0);
    }

    #[test]
    fn space_after_marks_adjacency() {
        let analysis = analyse("Hi there.");
        let tokens: Vec<(&str, bool)> = analysis
            .tokens
            .values()
            .map(|t| (t.surface.as_str(), t.space_after))
            .collect();
        assert_eq!(tokens, [("Hi", true), ("there", false), (".", true)]);
    }

    #[test]
    fn every_token_carries_text_and_sentence_support() {
        let analysis = analyse("Dr. Smith left.");
        for token in analysis.tokens.values() {
            let support = analysis.graph.support_of(FactId::Token(token.id)).unwrap();
            assert!(support
                .sources
                .iter()
                .any(|s| matches!(s, SourceRef::Text(_))));
            assert!(support
                .sources
                .iter()
                .any(|s| matches!(s, SourceRef::Sentence(_))));
        }
    }

    /// §4.2: same bytes in, same bytes out.
    #[test]
    fn serialization_is_byte_stable_across_runs() {
        let text = "There is many reasons. Dr. Smith didn't agree.";
        let first = analyse(text);
        let second = analyse(text);
        assert_eq!(first.to_canonical_json(), second.to_canonical_json());
        assert_eq!(first.digest(), second.digest());
    }

    /// Retracting the abbreviation entry must remove exactly the token that
    /// consulted it, and nothing else.
    #[test]
    fn retracting_a_lexicon_entry_cascades() {
        let pack = RulePack::builtin().unwrap();
        let mut analysis = analyze("Dr. Smith left.", &pack);
        let before = analysis.tokens.len();
        let entry = SourceRef::Lexicon(LexiconRef::new(
            "en.abbreviations",
            "Dr.",
            pack.artifact_version("abbreviations"),
        ));
        let report = analysis.retract(&entry);
        assert!(report.contains(FactId::Token(TokenId(0))));
        assert_eq!(analysis.tokens.len(), before - 1);
        assert!(!analysis.token_analyses.contains_key(&TokenId(0)));
        assert!(analysis.tokens.contains_key(&TokenId(1)));
    }
}
