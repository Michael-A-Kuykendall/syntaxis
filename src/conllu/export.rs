//! CoNLL-U export.
//!
//! Output is canonical: fixed MISC key order, LF endings, one blank line
//! between sentences, trailing newline. Export is therefore idempotent —
//! `export(import(export(x))) == export(x)` — which is what the round-trip
//! test actually asserts. Import is not required to preserve a hand-authored
//! file's incidental formatting, only its content.
//!
//! Where the engine has something UD has no column for, it goes in MISC under
//! an `ArcStatus`/`AltGroup` key rather than being dropped: a plain UD tool
//! still reads the file, and this engine still round-trips its own state.

use crate::parser_core;
use parser_core::analysis::Analysis;
use parser_core::ids::TokenId;
use parser_core::model::{ArcStatus, DependencyArc, Relation};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct ExportOptions {
    /// Emit `# sent_id` and `# text` comment lines.
    pub comments: bool,
    /// Emit non-accepted arcs in the DEPS column as `head:relation`.
    pub alternatives_in_deps: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            comments: true,
            alternatives_in_deps: true,
        }
    }
}

pub fn export(analysis: &Analysis) -> String {
    export_with(analysis, ExportOptions::default())
}

pub fn export_with(analysis: &Analysis, options: ExportOptions) -> String {
    let mut out = String::new();

    for sentence in analysis.sentences.values() {
        // token -> position within the sentence, 1-based as CoNLL-U requires.
        let positions: BTreeMap<TokenId, usize> = sentence
            .tokens
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index + 1))
            .collect();

        // Primary arc per token: the accepted one, else the first by id, so the
        // choice never depends on iteration order.
        let mut primary: BTreeMap<TokenId, &DependencyArc> = BTreeMap::new();
        let mut secondary: BTreeMap<TokenId, Vec<&DependencyArc>> = BTreeMap::new();
        for arc in analysis.arcs.values().filter(|a| a.sentence == sentence.id) {
            match primary.get(&arc.dependent) {
                Some(existing) if existing.status == ArcStatus::Accepted => {
                    secondary.entry(arc.dependent).or_default().push(arc);
                }
                Some(existing) => {
                    if arc.status == ArcStatus::Accepted {
                        secondary.entry(arc.dependent).or_default().push(existing);
                        primary.insert(arc.dependent, arc);
                    } else {
                        secondary.entry(arc.dependent).or_default().push(arc);
                    }
                }
                None => {
                    primary.insert(arc.dependent, arc);
                }
            }
        }

        if options.comments {
            out.push_str(&format!("# sent_id = {}\n", sentence.ordinal + 1));
            let text = sentence
                .span
                .slice(&analysis.document.text)
                .unwrap_or_default();
            out.push_str(&format!("# text = {text}\n"));
        }

        for (index, token_id) in sentence.tokens.iter().enumerate() {
            let Some(token) = analysis.tokens.get(token_id) else {
                continue;
            };
            let analysis_row = analysis.token_analyses.get(token_id);
            let arc = primary.get(token_id).copied();

            let (head, deprel) = match arc {
                None => ("_".to_string(), "_".to_string()),
                Some(arc) => {
                    let head = match arc.head {
                        None => "0".to_string(),
                        Some(h) => positions
                            .get(&h)
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "_".to_string()),
                    };
                    let label = match (&arc.raw_label, arc.relation) {
                        (Some(raw), _) => raw.clone(),
                        (None, Relation::Unsupported) => "dep".to_string(),
                        (None, relation) => relation.as_str().to_string(),
                    };
                    (head, label)
                }
            };

            let mut misc: Vec<String> = Vec::new();
            if !token.space_after {
                misc.push("SpaceAfter=No".to_string());
            }
            if let Some(arc) = arc {
                match &arc.status {
                    ArcStatus::Accepted => {}
                    ArcStatus::Alternative { group } => {
                        misc.push(format!("AltGroup={group}"));
                        if let Some(g) = analysis.alternative_groups.get(group) {
                            misc.push(format!("Resolution={}", g.resolution.as_str()));
                        }
                    }
                    other => misc.push(format!("ArcStatus={}", other.as_str())),
                }
            }
            if let Some(row) = analysis_row {
                misc.extend(row.unmapped_misc.iter().cloned());
            }

            let deps = if options.alternatives_in_deps {
                let mut entries: Vec<String> = secondary
                    .get(token_id)
                    .map(|arcs| {
                        arcs.iter()
                            .map(|a| {
                                let head = match a.head {
                                    None => "0".to_string(),
                                    Some(h) => positions
                                        .get(&h)
                                        .map(|p| p.to_string())
                                        .unwrap_or_else(|| "_".to_string()),
                                };
                                format!(
                                    "{head}:{}",
                                    a.raw_label
                                        .clone()
                                        .unwrap_or_else(|| a.relation.as_str().to_string())
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                entries.sort();
                entries.dedup();
                if entries.is_empty() {
                    "_".to_string()
                } else {
                    entries.join("|")
                }
            } else {
                "_".to_string()
            };

            let (lemma, upos, xpos, feats) = match analysis_row {
                Some(row) => (
                    if row.lemma.is_empty() {
                        "_".to_string()
                    } else {
                        row.lemma.clone()
                    },
                    row.upos.as_str().to_string(),
                    row.pos.as_str().to_string(),
                    {
                        let mut feats = row.morphology.to_feats();
                        if !row.unmapped_features.is_empty() {
                            let mut all: Vec<String> = if feats == "_" {
                                Vec::new()
                            } else {
                                feats.split('|').map(|s| s.to_string()).collect()
                            };
                            all.extend(row.unmapped_features.iter().cloned());
                            all.sort();
                            feats = all.join("|");
                        }
                        feats
                    },
                ),
                None => (
                    "_".to_string(),
                    "_".to_string(),
                    "_".to_string(),
                    "_".to_string(),
                ),
            };

            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                index + 1,
                token.surface,
                lemma,
                upos,
                xpos,
                feats,
                head,
                deprel,
                deps,
                if misc.is_empty() {
                    "_".to_string()
                } else {
                    misc.join("|")
                }
            ));
        }
        out.push('\n');
    }
    out
}
