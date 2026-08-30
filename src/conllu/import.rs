//! Strict CoNLL-U import.
//!
//! "Strict" means: every deviation is an error with a line number, and nothing
//! is guessed. A deprel outside the supported relation set becomes an
//! `Unsupported` arc carrying its original label, never a nearby supported
//! relation. Multi-word tokens and empty nodes are rejected outright
//! rather than silently flattened, because flattening them would break the
//! span contract.
//!
//! Imported facts carry `DerivationKind::Import`. They are evidence about what
//! an annotator said, not output of this engine, and the CLI labels them that
//! way.

use super::mapping;
use crate::english_rules;
use crate::parser_core;
use parser_core::analysis::Analysis;
use parser_core::ids::*;
use parser_core::model::*;
use parser_core::support::{DerivationKind, SourceRef, SupportSet};
use std::fmt;

/// Text joining policy: sentences are separated by a single newline when the
/// document text is reconstructed from a CoNLL-U file. Part of the mapping
/// version.
pub const SENTENCE_SEPARATOR: &str = "\n";

const IMPORT_RULE: &str = "IMPORT.CONLLU";

pub fn import_str(input: &str, rule_pack: &RulePackId) -> Result<Analysis, ConlluError> {
    let blocks = parse_blocks(input)?;

    // Pass 1: reconstruct the document text so spans can be assigned.
    let mut text = String::new();
    let mut sentence_offsets: Vec<usize> = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        if index > 0 {
            text.push_str(SENTENCE_SEPARATOR);
        }
        sentence_offsets.push(text.len());
        let rendered = render_sentence(block);
        if let Some(declared) = &block.text {
            if declared != &rendered {
                return Err(ConlluError::TextMismatch {
                    line: block.line,
                    declared: declared.clone(),
                    rendered,
                });
            }
        }
        text.push_str(&rendered);
    }

    let document = Document {
        id: DocumentId(0),
        text: text.clone(),
        sentences: Vec::new(),
        analysis_version: parser_core::ANALYSIS_VERSION,
        rule_pack: rule_pack.clone(),
        conllu_mapping_version: mapping::MAPPING_VERSION,
        dialect: "en-US".to_string(),
    };
    let mut analysis = Analysis::new(document);
    let map = english_rules::text::CharMap::new(&text);

    let mut next_token = 0u32;
    let mut next_arc = 0u32;

    for (index, block) in blocks.iter().enumerate() {
        let sentence_id = SentenceId(index as u32);
        let base = sentence_offsets[index];

        // Token spans, walking the rendered sentence.
        let mut offset = base;
        let mut ids: Vec<TokenId> = Vec::new();
        let mut spans: Vec<parser_core::span::Span> = Vec::new();
        for row in &block.rows {
            let start = offset;
            let end = start + row.form.len();
            spans.push(map.span(start as u32, end as u32));
            offset = end;
            if row.space_after {
                offset += 1;
            }
            ids.push(TokenId(next_token));
            next_token += 1;
        }
        let sentence_span = map.span(
            base as u32,
            spans.last().map(|s| s.byte_end).unwrap_or(base as u32),
        );

        let sentence_support = SupportSet::new(
            RuleId::new(IMPORT_RULE),
            rule_pack.clone(),
            DerivationKind::Import,
            vec![SourceRef::Text(sentence_span)],
        );

        for (ordinal, (row, span)) in block.rows.iter().zip(spans.iter()).enumerate() {
            let token_id = ids[ordinal];
            analysis.add_token(Token {
                id: token_id,
                sentence: sentence_id,
                ordinal: ordinal as u32,
                span: *span,
                surface: row.form.clone(),
                normalized: english_rules::text::normalize(&row.form),
                space_after: row.space_after,
                support: SupportSet::new(
                    RuleId::new(IMPORT_RULE),
                    rule_pack.clone(),
                    DerivationKind::Import,
                    vec![SourceRef::Text(*span), SourceRef::Sentence(sentence_id)],
                ),
            });

            analysis.add_token_analysis(TokenAnalysis {
                token: token_id,
                pos: row.pos,
                upos: row.upos,
                lemma: row.lemma.clone(),
                morphology: row.morphology,
                unmapped_features: row.unmapped_features.clone(),
                unmapped_misc: row.unmapped_misc.clone(),
                support: SupportSet::new(
                    RuleId::new(IMPORT_RULE),
                    rule_pack.clone(),
                    DerivationKind::Import,
                    vec![SourceRef::Token(token_id)],
                ),
            });
        }

        for (ordinal, row) in block.rows.iter().enumerate() {
            let Some(head) = row.head else { continue };
            let dependent = ids[ordinal];
            let head_token = if head == 0 {
                None
            } else {
                Some(*ids.get(head - 1).ok_or(ConlluError::HeadOutOfRange {
                    line: row.line,
                    head,
                })?)
            };

            let mut sources = vec![SourceRef::TokenAnalysis(dependent)];
            if let Some(h) = head_token {
                sources.push(SourceRef::TokenAnalysis(h));
            }

            analysis.add_arc(DependencyArc {
                id: ArcId(next_arc),
                sentence: sentence_id,
                head: head_token,
                dependent,
                relation: row.relation,
                raw_label: row.raw_label.clone(),
                status: row.status.clone(),
                support: SupportSet::new(
                    RuleId::new(IMPORT_RULE),
                    rule_pack.clone(),
                    DerivationKind::Import,
                    sources,
                ),
            });
            next_arc += 1;
        }

        analysis.add_sentence(Sentence {
            id: sentence_id,
            ordinal: index as u32,
            span: sentence_span,
            tokens: ids,
            support: sentence_support,
        });
    }

    Ok(analysis)
}

fn render_sentence(block: &Block) -> String {
    let mut out = String::new();
    for (index, row) in block.rows.iter().enumerate() {
        out.push_str(&row.form);
        if row.space_after && index + 1 < block.rows.len() {
            out.push(' ');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Low level parsing
// ---------------------------------------------------------------------------

pub(crate) struct Block {
    pub line: usize,
    pub sent_id: Option<String>,
    pub text: Option<String>,
    pub rows: Vec<Row>,
}

pub(crate) struct Row {
    pub line: usize,
    pub form: String,
    pub lemma: String,
    pub upos: UPos,
    pub pos: Pos,
    pub morphology: Morphology,
    pub unmapped_features: Vec<String>,
    pub unmapped_misc: Vec<String>,
    pub head: Option<usize>,
    pub relation: Relation,
    pub raw_label: Option<String>,
    pub status: ArcStatus,
    pub space_after: bool,
}

fn parse_blocks(input: &str) -> Result<Vec<Block>, ConlluError> {
    let mut blocks = Vec::new();
    let mut current: Option<Block> = None;

    for (index, raw) in input.lines().enumerate() {
        let line = index + 1;
        let content = raw.trim_end_matches('\r');

        if content.trim().is_empty() {
            if let Some(block) = current.take() {
                blocks.push(finish(block, line)?);
            }
            continue;
        }

        if let Some(comment) = content.strip_prefix('#') {
            let block = current.get_or_insert(Block {
                line,
                sent_id: None,
                text: None,
                rows: Vec::new(),
            });
            if let Some((key, value)) = comment.split_once('=') {
                match key.trim() {
                    "sent_id" => block.sent_id = Some(value.trim().to_string()),
                    "text" => block.text = Some(value.trim().to_string()),
                    _ => {}
                }
            }
            continue;
        }

        let fields: Vec<&str> = content.split('\t').collect();
        if fields.len() != 10 {
            return Err(ConlluError::WrongColumnCount {
                line,
                found: fields.len(),
            });
        }

        let id = fields[0];
        if id.contains('-') {
            return Err(ConlluError::MultiwordToken {
                line,
                id: id.to_string(),
            });
        }
        if id.contains('.') {
            return Err(ConlluError::EmptyNode {
                line,
                id: id.to_string(),
            });
        }
        let ordinal: usize = id.parse().map_err(|_| ConlluError::BadId {
            line,
            id: id.to_string(),
        })?;

        let block = current.get_or_insert(Block {
            line,
            sent_id: None,
            text: None,
            rows: Vec::new(),
        });
        if ordinal != block.rows.len() + 1 {
            return Err(ConlluError::NonConsecutiveId {
                line,
                expected: block.rows.len() + 1,
                found: ordinal,
            });
        }

        let form = fields[1].to_string();
        if form.is_empty() || form == "_" {
            return Err(ConlluError::EmptyForm { line });
        }
        if form.chars().any(|c| c.is_whitespace()) {
            return Err(ConlluError::WhitespaceInForm { line });
        }

        let upos = if fields[3] == "_" {
            None
        } else {
            Some(
                UPos::parse(fields[3]).ok_or_else(|| ConlluError::UnknownUpos {
                    line,
                    value: fields[3].to_string(),
                })?,
            )
        };
        let pos = if fields[4] == "_" {
            upos.map(mapping::pos_for).unwrap_or(Pos::Unknown)
        } else {
            Pos::parse(fields[4]).ok_or_else(|| ConlluError::UnknownXpos {
                line,
                value: fields[4].to_string(),
            })?
        };
        let upos = upos.unwrap_or_else(|| mapping::upos_for(pos));

        let (morphology, unmapped_features) = Morphology::from_feats(fields[5])
            .map_err(|detail| ConlluError::BadFeats { line, detail })?;

        let head = if fields[6] == "_" {
            None
        } else {
            Some(fields[6].parse().map_err(|_| ConlluError::BadHead {
                line,
                value: fields[6].to_string(),
            })?)
        };

        let deprel = fields[7];
        let (relation, raw_label) = if deprel == "_" {
            if head.is_some() {
                return Err(ConlluError::HeadWithoutDeprel { line });
            }
            (Relation::Unsupported, None)
        } else {
            match Relation::parse(deprel) {
                Some(r) => (r, None),
                // Outside the first-gate set: recorded as unsupported with the
                // original label kept, never mapped onto a nearby relation.
                None => (Relation::Unsupported, Some(deprel.to_string())),
            }
        };

        let mut space_after = true;
        let mut unmapped_misc = Vec::new();
        let mut status = if relation == Relation::Unsupported && head.is_some() {
            ArcStatus::Unsupported
        } else {
            ArcStatus::Accepted
        };
        if fields[9] != "_" {
            for entry in fields[9].split('|') {
                match entry.split_once('=') {
                    Some(("SpaceAfter", "No")) => space_after = false,
                    Some(("ArcStatus", "unsupported")) => status = ArcStatus::Unsupported,
                    Some(("ArcStatus", "rejected")) => status = ArcStatus::Rejected,
                    Some(("ArcStatus", "accepted")) => status = ArcStatus::Accepted,
                    Some(("AltGroup", value)) => {
                        let id = value.trim_start_matches('x').parse().map_err(|_| {
                            ConlluError::BadMisc {
                                line,
                                entry: entry.to_string(),
                            }
                        })?;
                        status = ArcStatus::Alternative {
                            group: AlternativeGroupId(id),
                        };
                    }
                    _ => unmapped_misc.push(entry.to_string()),
                }
            }
        }

        block.rows.push(Row {
            line,
            form,
            lemma: fields[2].to_string(),
            upos,
            pos,
            morphology,
            unmapped_features,
            unmapped_misc,
            head,
            relation,
            raw_label,
            status,
            space_after,
        });
    }

    if let Some(block) = current.take() {
        let line = input.lines().count() + 1;
        blocks.push(finish(block, line)?);
    }
    Ok(blocks)
}

fn finish(block: Block, line: usize) -> Result<Block, ConlluError> {
    if block.rows.is_empty() {
        return Err(ConlluError::EmptySentence { line });
    }
    let roots = block
        .rows
        .iter()
        .filter(|r| r.head == Some(0) && r.status == ArcStatus::Accepted)
        .count();
    if roots > 1 {
        return Err(ConlluError::MultipleRoots {
            line: block.line,
            found: roots,
        });
    }
    Ok(block)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConlluError {
    WrongColumnCount {
        line: usize,
        found: usize,
    },
    MultiwordToken {
        line: usize,
        id: String,
    },
    EmptyNode {
        line: usize,
        id: String,
    },
    BadId {
        line: usize,
        id: String,
    },
    NonConsecutiveId {
        line: usize,
        expected: usize,
        found: usize,
    },
    EmptyForm {
        line: usize,
    },
    WhitespaceInForm {
        line: usize,
    },
    UnknownUpos {
        line: usize,
        value: String,
    },
    UnknownXpos {
        line: usize,
        value: String,
    },
    BadFeats {
        line: usize,
        detail: String,
    },
    BadHead {
        line: usize,
        value: String,
    },
    HeadOutOfRange {
        line: usize,
        head: usize,
    },
    HeadWithoutDeprel {
        line: usize,
    },
    BadMisc {
        line: usize,
        entry: String,
    },
    EmptySentence {
        line: usize,
    },
    MultipleRoots {
        line: usize,
        found: usize,
    },
    TextMismatch {
        line: usize,
        declared: String,
        rendered: String,
    },
}

impl fmt::Display for ConlluError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ConlluError::*;
        match self {
            WrongColumnCount { line, found } => {
                write!(f, "line {line}: expected 10 tab-separated columns, found {found}")
            }
            MultiwordToken { line, id } => write!(
                f,
                "line {line}: multi-word token `{id}` is not supported; the engine needs one span per token"
            ),
            EmptyNode { line, id } => write!(
                f,
                "line {line}: empty node `{id}` is not supported; the engine has no null tokens"
            ),
            BadId { line, id } => write!(f, "line {line}: unreadable ID `{id}`"),
            NonConsecutiveId { line, expected, found } => {
                write!(f, "line {line}: expected ID {expected}, found {found}")
            }
            EmptyForm { line } => write!(f, "line {line}: FORM must not be empty"),
            WhitespaceInForm { line } => write!(f, "line {line}: FORM must not contain whitespace"),
            UnknownUpos { line, value } => write!(f, "line {line}: unknown UPOS `{value}`"),
            UnknownXpos { line, value } => write!(f, "line {line}: unknown XPOS `{value}`"),
            BadFeats { line, detail } => write!(f, "line {line}: {detail}"),
            BadHead { line, value } => write!(f, "line {line}: unreadable HEAD `{value}`"),
            HeadOutOfRange { line, head } => {
                write!(f, "line {line}: HEAD {head} points past the end of the sentence")
            }
            HeadWithoutDeprel { line } => write!(f, "line {line}: HEAD given without a DEPREL"),
            BadMisc { line, entry } => write!(f, "line {line}: unreadable MISC entry `{entry}`"),
            EmptySentence { line } => write!(f, "line {line}: sentence block has no tokens"),
            MultipleRoots { line, found } => {
                write!(f, "line {line}: sentence has {found} accepted roots, expected one")
            }
            TextMismatch { line, declared, rendered } => write!(
                f,
                "line {line}: `# text` says `{declared}` but the tokens render as `{rendered}`"
            ),
        }
    }
}

impl std::error::Error for ConlluError {}
