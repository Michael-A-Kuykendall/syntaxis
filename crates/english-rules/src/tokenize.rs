//! Tokenization.
//!
//! Two invariants hold for every input, and both are tested:
//!
//! * concatenating token surfaces in order reproduces the input with
//!   whitespace removed — tokenization neither invents nor drops characters;
//! * every token span slices back to exactly its surface.
//!
//! Splitting decisions come from the versioned artifacts in the rule pack
//! rather than from literals in this file, so a change in behaviour is always
//! a change in a checksummed, licensed resource.

use crate::rulepack::RulePack;
use crate::text::{normalize, CharMap};
use parser_core::ids::RuleId;
use parser_core::span::Span;
use parser_core::support::LexiconRef;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RawToken {
    pub span: Span,
    pub surface: String,
    pub normalized: String,
    pub rule: RuleId,
    /// Reference artifact entry consulted, when one decided this token.
    pub lexicon: Option<LexiconRef>,
}

fn is_opener(c: char) -> bool {
    matches!(
        c,
        '(' | '['
            | '{'
            | '"'
            | '\''
            | '`'
            | '\u{201C}'
            | '\u{2018}'
            | '\u{00AB}'
            | '\u{00BF}'
            | '\u{00A1}'
    )
}

fn is_closer(c: char) -> bool {
    matches!(
        c,
        ')' | ']'
            | '}'
            | '"'
            | '\''
            | '.'
            | ','
            | ';'
            | ':'
            | '!'
            | '?'
            | '\u{201D}'
            | '\u{2019}'
            | '\u{00BB}'
            | '\u{2026}'
    )
}

fn is_hyphen(c: char) -> bool {
    matches!(c, '-' | '\u{2010}' | '\u{2011}' | '\u{2013}' | '\u{2014}')
}

struct Cx<'a> {
    text: &'a str,
    pack: &'a RulePack,
    map: &'a CharMap,
    out: Vec<RawToken>,
}

impl<'a> Cx<'a> {
    fn push(&mut self, start: usize, end: usize, rule: &str, lexicon: Option<LexiconRef>) {
        if start >= end {
            return;
        }
        let surface = &self.text[start..end];
        self.out.push(RawToken {
            span: self.map.span(start as u32, end as u32),
            surface: surface.to_string(),
            normalized: normalize(surface),
            rule: RuleId::new(rule),
            lexicon,
        });
    }

    fn abbreviation_ref(&self, entry: &str) -> LexiconRef {
        LexiconRef::new(
            "en.abbreviations",
            entry,
            self.pack.artifact_version("abbreviations"),
        )
    }
}

/// Tokenize one sentence span. Whitespace never becomes a token.
pub fn tokenize_sentence(
    text: &str,
    sentence: Span,
    pack: &RulePack,
    map: &CharMap,
) -> Vec<RawToken> {
    let mut cx = Cx {
        text,
        pack,
        map,
        out: Vec::new(),
    };
    let start = sentence.byte_start as usize;
    let end = sentence.byte_end as usize;
    let slice = &text[start..end];

    let mut chunk_start: Option<usize> = None;
    for (offset, c) in slice.char_indices() {
        let absolute = start + offset;
        if c.is_whitespace() {
            if let Some(s) = chunk_start.take() {
                chunk(&mut cx, s, absolute);
            }
        } else if chunk_start.is_none() {
            chunk_start = Some(absolute);
        }
    }
    if let Some(s) = chunk_start {
        chunk(&mut cx, s, end);
    }
    cx.out
}

/// One whitespace-delimited chunk: peel openers, peel closers, then split the
/// core.
fn chunk(cx: &mut Cx<'_>, mut start: usize, mut end: usize) {
    // Leading openers.
    while start < end {
        let c = cx.text[start..].chars().next().unwrap();
        if !is_opener(c) {
            break;
        }
        let next = start + c.len_utf8();
        cx.push(start, next, "TOK.PUNCT.LEADING", None);
        start = next;
    }

    // Trailing closers, right to left; emitted after the core.
    let mut trailing: Vec<(usize, usize, &'static str)> = Vec::new();
    while start < end {
        let core = &cx.text[start..end];
        if cx.pack.abbreviations.contains(core) {
            break; // TOK.NOSPLIT.ABBREVIATION
        }
        let last = core.chars().next_back().unwrap();
        if !is_closer(last) {
            break;
        }
        if last == '.' {
            let run: usize = core.chars().rev().take_while(|c| *c == '.').count();
            if run >= 2 {
                let cut = end - run;
                trailing.push((cut, end, "TOK.PUNCT.ELLIPSIS"));
                end = cut;
                continue;
            }
        }
        if last == '\u{2026}' {
            let cut = end - last.len_utf8();
            trailing.push((cut, end, "TOK.PUNCT.ELLIPSIS"));
            end = cut;
            continue;
        }
        let cut = end - last.len_utf8();
        trailing.push((cut, end, "TOK.PUNCT.TRAILING"));
        end = cut;
    }

    core(cx, start, end);

    for (s, e, rule) in trailing.into_iter().rev() {
        cx.push(s, e, rule, None);
    }
}

fn core(cx: &mut Cx<'_>, start: usize, end: usize) {
    if start >= end {
        return;
    }
    let core = &cx.text[start..end];

    // A known abbreviation is emitted whole, and records the entry that saved
    // its period so the token can be retracted if the artifact changes.
    if cx.pack.abbreviations.contains(core) {
        let entry = cx.abbreviation_ref(core);
        cx.push(start, end, "TOK.NOSPLIT.ABBREVIATION", Some(entry));
        return;
    }

    // Nothing word-like: emit runs of identical symbols as single tokens.
    if !core.chars().any(|c| c.is_alphanumeric()) {
        let mut offset = start;
        while offset < end {
            let c = cx.text[offset..].chars().next().unwrap();
            let mut run_end = offset;
            while run_end < end && cx.text[run_end..].starts_with(c) {
                run_end += c.len_utf8();
            }
            let rule = if c == '.' && run_end - offset >= 2 * c.len_utf8() {
                "TOK.PUNCT.ELLIPSIS"
            } else {
                "TOK.SYMBOL"
            };
            cx.push(offset, run_end, rule, None);
            offset = run_end;
        }
        return;
    }

    // Word-internal hyphens split, matching the UD English convention.
    let mut segment_start = start;
    let mut offset = start;
    while offset < end {
        let c = cx.text[offset..].chars().next().unwrap();
        let next = offset + c.len_utf8();
        if is_hyphen(c) && offset > segment_start && next < end {
            word(cx, segment_start, offset);
            cx.push(offset, next, "TOK.HYPHEN", None);
            segment_start = next;
        }
        offset = next;
    }
    word(cx, segment_start, end);
}

/// A hyphen-free word core: fused form, then clitics, then plain word.
fn word(cx: &mut Cx<'_>, start: usize, end: usize) {
    if start >= end {
        return;
    }
    let surface = &cx.text[start..end];
    let lower = normalize(surface);

    if let Some(parts) = cx.pack.fused.split(&lower) {
        let total: usize = parts.iter().map(|p| p.len()).sum();
        // Guard: lowercasing can change byte length, and a split whose parts do
        // not line up with the surface would corrupt spans. Fall through to the
        // plain-word rule instead of emitting a wrong span.
        if total == surface.len() {
            let parts: Vec<usize> = parts.iter().map(|p| p.len()).collect();
            let mut offset = start;
            for length in parts {
                cx.push(offset, offset + length, "TOK.FUSED", None);
                offset += length;
            }
            return;
        }
    }

    // Peel clitic suffixes right to left, recording each as a span.
    //
    // The match is made on the *normalized* tail rather than on raw bytes, so
    // `don't` and `don\u{2019}t` split identically while the emitted surface
    // keeps the original character. Byte arithmetic on the surface would land
    // mid-character for the curly form and silently fail to split.
    let lookback = cx.pack.clitics.max_chars();
    let mut cut = end;
    let mut clitics: Vec<(usize, usize)> = Vec::new();
    loop {
        let current = &cx.text[start..cut];
        let mut boundary = None;
        for (offset, _) in current.char_indices().rev().take(lookback) {
            let absolute = start + offset;
            if absolute <= start {
                break;
            }
            if cx
                .pack
                .clitics
                .contains(&normalize(&cx.text[absolute..cut]))
            {
                // Keep scanning leftwards: longest match wins (`n't` over `'t`).
                boundary = Some(absolute);
            }
        }
        let Some(boundary) = boundary else { break };
        clitics.push((boundary, cut));
        cut = boundary;
    }

    cx.push(start, cut, "TOK.WORD", None);
    for (s, e) in clitics.into_iter().rev() {
        cx.push(s, e, "TOK.CLITIC", None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(text: &str) -> Vec<String> {
        let pack = RulePack::builtin().unwrap();
        let map = CharMap::new(text);
        let sentence = map.span(0, text.len() as u32);
        tokenize_sentence(text, sentence, &pack, &map)
            .into_iter()
            .map(|t| t.surface)
            .collect()
    }

    fn raw(text: &str) -> Vec<RawToken> {
        let pack = RulePack::builtin().unwrap();
        let map = CharMap::new(text);
        let sentence = map.span(0, text.len() as u32);
        tokenize_sentence(text, sentence, &pack, &map)
    }

    #[test]
    fn splits_final_punctuation() {
        assert_eq!(
            tokens("The cat are sleeping."),
            ["The", "cat", "are", "sleeping", "."]
        );
    }

    #[test]
    fn motivating_cases_tokenize_cleanly() {
        assert_eq!(
            tokens("Each of the students have a book."),
            ["Each", "of", "the", "students", "have", "a", "book", "."]
        );
        assert_eq!(
            tokens("There is many reasons."),
            ["There", "is", "many", "reasons", "."]
        );
    }

    #[test]
    fn splits_contractions_penn_style() {
        assert_eq!(tokens("Don't."), ["Do", "n't", "."]);
        assert_eq!(tokens("They're here"), ["They", "'re", "here"]);
        assert_eq!(tokens("the cat's toy"), ["the", "cat", "'s", "toy"]);
        assert_eq!(tokens("cannot"), ["can", "not"]);
    }

    /// A curly apostrophe must split exactly like an ASCII one, while the
    /// emitted surface keeps the original character.
    #[test]
    fn curly_apostrophes_split_like_ascii_ones() {
        assert_eq!(tokens("Don\u{2019}t"), ["Do", "n\u{2019}t"]);
        assert_eq!(
            tokens("the cat\u{2019}s toy"),
            ["the", "cat", "\u{2019}s", "toy"]
        );
    }

    #[test]
    fn splits_hyphenated_compounds() {
        assert_eq!(
            tokens("a well-known case"),
            ["a", "well", "-", "known", "case"]
        );
        assert_eq!(tokens("--"), ["--"]);
    }

    #[test]
    fn brackets_and_decimals() {
        assert_eq!(tokens("(3.14)"), ["(", "3.14", ")"]);
        assert_eq!(tokens("\"Stop!\""), ["\"", "Stop", "!", "\""]);
    }

    #[test]
    fn abbreviations_keep_their_period_and_record_the_entry() {
        let ts = raw("Dr. Smith left.");
        let surfaces: Vec<&str> = ts.iter().map(|t| t.surface.as_str()).collect();
        assert_eq!(surfaces, ["Dr.", "Smith", "left", "."]);
        assert_eq!(ts[0].rule.as_str(), "TOK.NOSPLIT.ABBREVIATION");
        assert_eq!(
            ts[0].lexicon.as_ref().map(|l| l.entry.as_str()),
            Some("Dr.")
        );
        assert!(ts[1].lexicon.is_none());
    }

    #[test]
    fn ellipsis_is_one_token() {
        assert_eq!(tokens("wait... then go"), ["wait", "...", "then", "go"]);
        assert_eq!(tokens("wait\u{2026}"), ["wait", "\u{2026}"]);
    }

    #[test]
    fn trailing_punctuation_peels_in_order() {
        assert_eq!(tokens("(really?),"), ["(", "really", "?", ")", ","]);
        assert_eq!(tokens("e.g., this"), ["e.g.", ",", "this"]);
    }

    /// The invariant that matters most: no character is invented or lost.
    #[test]
    fn surfaces_reconstruct_the_input() {
        for text in [
            "The cat are sleeping.",
            "Each of the students have a book.",
            "Dr. J. R. R. Tolkien didn't write \"Cannot-Be\" (3.14)...",
            "naïve café — well-known, e.g. this.",
            "They're the students' books.",
        ] {
            let joined: String = raw(text).iter().map(|t| t.surface.clone()).collect();
            let expected: String = text.chars().filter(|c| !c.is_whitespace()).collect();
            assert_eq!(joined, expected, "input: {text}");
        }
    }

    #[test]
    fn every_span_slices_back_to_its_surface() {
        let text = "naïve café didn't — e.g. \"quoted\" (3.14)...";
        for token in raw(text) {
            assert_eq!(token.span.validate(text), Ok(()), "{token:?}");
            assert_eq!(token.span.slice(text), Some(token.surface.as_str()));
        }
    }

    #[test]
    fn tokens_are_ordered_and_non_overlapping() {
        let text = "Dr. Smith didn't go to the well-known café.";
        let mut previous = 0u32;
        for token in raw(text) {
            assert!(token.span.byte_start >= previous, "{token:?}");
            previous = token.span.byte_end;
        }
    }

    #[test]
    fn tokenization_is_repeatable() {
        let text = "Each of the students have a book; the cat are sleeping.";
        assert_eq!(raw(text), raw(text));
    }
}
