//! Canonical serialization of an [`Analysis`].
//!
//! Field order below *is* the contract. Collections are emitted in id order
//! because they are stored in `BTreeMap`s, so no sorting step can be forgotten.
//! Nothing here reads the clock, the environment, or a hash map iterator.

use crate::analysis::Analysis;
use crate::hash::sha256_hex;
use crate::json::Json;
use crate::model::*;
use crate::span::Span;
use crate::support::{SourceRef, SupportSet};

pub const SCHEMA: &str = "syntaxis/analysis";
pub const SCHEMA_VERSION: &str = "0.1.0";

pub fn span_json(span: &Span) -> Json {
    Json::object(vec![
        ("byte_start", Json::Uint(span.byte_start as u64)),
        ("byte_end", Json::Uint(span.byte_end as u64)),
        ("char_start", Json::Uint(span.char_start as u64)),
        ("char_end", Json::Uint(span.char_end as u64)),
    ])
}

pub fn support_json(support: &SupportSet) -> Json {
    Json::object(vec![
        ("rule", Json::str(support.rule.as_str())),
        ("rule_pack", Json::str(support.rule_pack.as_str())),
        ("derivation", Json::str(support.derivation.as_str())),
        (
            "sources",
            Json::array(support.sources.iter().map(source_json)),
        ),
    ])
}

fn source_json(source: &SourceRef) -> Json {
    Json::str(source.to_string())
}

fn morphology_json(m: &Morphology) -> Json {
    Json::object(vec![
        ("number", Json::str(m.number.as_str())),
        ("person", Json::str(m.person.as_str())),
        ("tense", Json::str(m.tense.as_str())),
        ("verb_form", Json::str(m.verb_form.as_str())),
        ("det_kind", Json::str(m.det_kind.as_str())),
        ("pron_kind", Json::str(m.pron_kind.as_str())),
    ])
}

fn arc_status_json(status: &ArcStatus) -> Json {
    match status {
        ArcStatus::Alternative { group } => Json::object(vec![
            ("kind", Json::str(status.as_str())),
            ("group", Json::str(group.to_string())),
        ]),
        other => Json::object(vec![("kind", Json::str(other.as_str()))]),
    }
}

pub fn analysis_json(analysis: &Analysis) -> Json {
    let document = &analysis.document;

    let sentences = Json::array(analysis.sentences.values().map(|s| {
        Json::object(vec![
            ("id", Json::str(s.id.to_string())),
            ("ordinal", Json::Uint(s.ordinal as u64)),
            ("span", span_json(&s.span)),
            (
                "tokens",
                Json::array(s.tokens.iter().map(|t| Json::str(t.to_string()))),
            ),
            ("support", support_json(&s.support)),
        ])
    }));

    let tokens = Json::array(analysis.tokens.values().map(|t| {
        Json::object(vec![
            ("id", Json::str(t.id.to_string())),
            ("sentence", Json::str(t.sentence.to_string())),
            ("ordinal", Json::Uint(t.ordinal as u64)),
            ("span", span_json(&t.span)),
            ("surface", Json::str(&t.surface)),
            ("normalized", Json::str(&t.normalized)),
            ("space_after", Json::Bool(t.space_after)),
            ("support", support_json(&t.support)),
        ])
    }));

    let token_analyses = Json::array(analysis.token_analyses.values().map(|a| {
        Json::object(vec![
            ("token", Json::str(a.token.to_string())),
            ("pos", Json::str(a.pos.as_str())),
            ("upos", Json::str(a.upos.as_str())),
            ("lemma", Json::str(&a.lemma)),
            ("morphology", morphology_json(&a.morphology)),
            (
                "unmapped_features",
                Json::array(a.unmapped_features.iter().map(Json::str)),
            ),
            (
                "unmapped_misc",
                Json::array(a.unmapped_misc.iter().map(Json::str)),
            ),
            ("support", support_json(&a.support)),
        ])
    }));

    let arcs = Json::array(analysis.arcs.values().map(|arc| {
        Json::object(vec![
            ("id", Json::str(arc.id.to_string())),
            ("sentence", Json::str(arc.sentence.to_string())),
            (
                "head",
                match arc.head {
                    Some(h) => Json::str(h.to_string()),
                    None => Json::Null,
                },
            ),
            ("dependent", Json::str(arc.dependent.to_string())),
            ("relation", Json::str(arc.relation.as_str())),
            (
                "raw_label",
                match &arc.raw_label {
                    Some(l) => Json::str(l),
                    None => Json::Null,
                },
            ),
            ("status", arc_status_json(&arc.status)),
            ("support", support_json(&arc.support)),
        ])
    }));

    let groups = Json::array(analysis.alternative_groups.values().map(|g| {
        Json::object(vec![
            ("id", Json::str(g.id.to_string())),
            ("sentence", Json::str(g.sentence.to_string())),
            (
                "members",
                Json::array(g.members.iter().map(|m| Json::str(m.to_string()))),
            ),
            ("resolution", Json::str(g.resolution.as_str())),
            ("support", support_json(&g.support)),
        ])
    }));

    let diagnostics = Json::array(analysis.diagnostics.values().map(|d| {
        Json::object(vec![
            ("id", Json::str(d.id.to_string())),
            ("sentence", Json::str(d.sentence.to_string())),
            ("kind", Json::str(d.kind.as_str())),
            ("span", span_json(&d.span)),
            ("message_key", Json::str(d.message_key.as_str())),
            ("certainty", Json::str(d.certainty.as_str())),
            (
                "replacements",
                Json::array(d.replacements.iter().map(|r| {
                    Json::object(vec![
                        ("span", span_json(&r.span)),
                        ("text", Json::str(&r.text)),
                    ])
                })),
            ),
            ("support", support_json(&d.support)),
        ])
    }));

    let retractions = Json::array(analysis.retractions.iter().map(|r| {
        Json::object(vec![
            ("retracted", Json::str(r.retracted.to_string())),
            (
                "removed",
                Json::array(r.removed.iter().map(|f| Json::str(f.to_string()))),
            ),
        ])
    }));

    Json::object(vec![
        ("schema", Json::str(SCHEMA)),
        ("schema_version", Json::str(SCHEMA_VERSION)),
        (
            "document",
            Json::object(vec![
                ("id", Json::str(document.id.to_string())),
                (
                    "analysis_version",
                    Json::str(document.analysis_version.to_string()),
                ),
                ("rule_pack", Json::str(document.rule_pack.as_str())),
                (
                    "conllu_mapping_version",
                    Json::str(document.conllu_mapping_version.to_string()),
                ),
                ("dialect", Json::str(&document.dialect)),
                ("text", Json::str(&document.text)),
            ]),
        ),
        ("sentences", sentences),
        ("tokens", tokens),
        ("token_analyses", token_analyses),
        ("arcs", arcs),
        ("alternative_groups", groups),
        ("diagnostics", diagnostics),
        ("retractions", retractions),
    ])
}

impl Analysis {
    pub fn to_json(&self) -> Json {
        analysis_json(self)
    }

    /// Canonical pretty JSON. This is what regression fixtures store.
    pub fn to_canonical_json(&self) -> String {
        analysis_json(self).to_canonical_string()
    }

    /// Content digest over the compact canonical form. Two runs that agree on
    /// this digest agree on the analysis, timing metadata excluded (there is
    /// none in the serialization).
    pub fn digest(&self) -> String {
        sha256_hex(analysis_json(self).to_compact_string().as_bytes())
    }
}
