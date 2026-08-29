//! Sentence segmentation.
//!
//! Every sentence records the rule that closed it, and every suppressed break
//! records the rule that suppressed it. The suppressions are kept because an
//! engine that cannot say *why* it declined to split is not explainable, and
//! because they are the first thing to look at when the corpus gate fails.

use crate::rulepack::RulePack;
use crate::text::CharMap;
use parser_core::ids::RuleId;
use parser_core::span::Span;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SegmentedSentence {
    pub span: Span,
    /// The rule that ended this sentence.
    pub rule: RuleId,
    /// Breaks considered and declined inside this sentence, in text order.
    pub suppressed: Vec<SuppressedBreak>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SuppressedBreak {
    pub span: Span,
    pub rule: RuleId,
    /// Abbreviation entry consulted, when the suppression came from the lexicon.
    pub lexicon_entry: Option<String>,
}

const TERMINALS: [char; 4] = ['.', '!', '?', '\u{2026}'];

fn is_closer(c: char) -> bool {
    matches!(
        c,
        ')' | ']' | '}' | '"' | '\'' | '\u{201D}' | '\u{2019}' | '\u{00BB}'
    )
}

fn can_open_sentence(c: char) -> bool {
    c.is_uppercase()
        || c.is_ascii_digit()
        || matches!(
            c,
            '(' | '['
                | '{'
                | '"'
                | '\''
                | '\u{201C}'
                | '\u{2018}'
                | '\u{00AB}'
                | '\u{00BF}'
                | '\u{00A1}'
        )
}

/// Split `text` into sentences. Whitespace between sentences belongs to no
/// sentence; sentence spans start and end on non-whitespace characters.
pub fn segment(text: &str, pack: &RulePack, map: &CharMap) -> Vec<SegmentedSentence> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut sentences = Vec::new();
    let mut start: Option<usize> = None; // index into `chars`
    let mut last_non_ws: Option<usize> = None;
    let mut suppressed: Vec<SuppressedBreak> = Vec::new();

    let mut i = 0usize;
    while i < chars.len() {
        let (byte, c) = chars[i];

        if c.is_whitespace() {
            // Paragraph break: whitespace run containing two or more newlines.
            let mut j = i;
            let mut newlines = 0usize;
            while j < chars.len() && chars[j].1.is_whitespace() {
                if chars[j].1 == '\n' {
                    newlines += 1;
                }
                j += 1;
            }
            if newlines >= 2 {
                if let (Some(s), Some(e)) = (start, last_non_ws) {
                    sentences.push(close(
                        &chars,
                        text,
                        map,
                        s,
                        e,
                        "SEG.BREAK.PARAGRAPH",
                        &mut suppressed,
                    ));
                    start = None;
                    last_non_ws = None;
                }
            }
            i = j;
            continue;
        }

        if start.is_none() {
            start = Some(i);
        }
        last_non_ws = Some(i);

        if TERMINALS.contains(&c) {
            let previous = if i > 0 { Some(chars[i - 1].1) } else { None };
            let next = chars.get(i + 1).map(|(_, c)| *c);

            // Numeric interior: 3.14, 1.2.3
            if c == '.'
                && previous.map(|p| p.is_ascii_digit()).unwrap_or(false)
                && next.map(|n| n.is_ascii_digit()).unwrap_or(false)
            {
                suppressed.push(SuppressedBreak {
                    span: map.span(byte as u32, (byte + c.len_utf8()) as u32),
                    rule: RuleId::new("SEG.NOBREAK.NUMERIC"),
                    lexicon_entry: None,
                });
                i += 1;
                continue;
            }

            if c == '.' {
                let word = word_ending_at(&chars, text, i);
                if pack.abbreviations.contains(word) {
                    suppressed.push(SuppressedBreak {
                        span: map.span(byte as u32, (byte + c.len_utf8()) as u32),
                        rule: RuleId::new("SEG.NOBREAK.ABBREVIATION"),
                        lexicon_entry: Some(word.to_string()),
                    });
                    i += 1;
                    continue;
                }
                // Single-letter initial: "J. R. R. Tolkien"
                if word.chars().count() == 2
                    && word
                        .chars()
                        .next()
                        .map(|c| c.is_alphabetic())
                        .unwrap_or(false)
                {
                    suppressed.push(SuppressedBreak {
                        span: map.span(byte as u32, (byte + c.len_utf8()) as u32),
                        rule: RuleId::new("SEG.NOBREAK.INITIAL"),
                        lexicon_entry: None,
                    });
                    i += 1;
                    continue;
                }
            }

            // Absorb the rest of the terminal run and any closing marks.
            let mut end = i;
            while end + 1 < chars.len()
                && (TERMINALS.contains(&chars[end + 1].1) || is_closer(chars[end + 1].1))
            {
                end += 1;
            }

            // Look ahead past whitespace for something that can open a sentence.
            let mut k = end + 1;
            let mut saw_space = false;
            while k < chars.len() && chars[k].1.is_whitespace() {
                saw_space = true;
                k += 1;
            }
            let opens = match chars.get(k) {
                None => true, // end of text always closes
                Some((_, c)) => saw_space && can_open_sentence(*c),
            };

            if opens {
                let s = start.unwrap_or(i);
                sentences.push(close(
                    &chars,
                    text,
                    map,
                    s,
                    end,
                    "SEG.BREAK.TERMINAL",
                    &mut suppressed,
                ));
                start = None;
                last_non_ws = None;
                i = end + 1;
                continue;
            }
            // Continuation: a lowercase word follows, so this was not a break.
            if chars.get(k).is_some() {
                suppressed.push(SuppressedBreak {
                    span: map.span(byte as u32, (byte + c.len_utf8()) as u32),
                    rule: RuleId::new("SEG.NOBREAK.LOWERCASE_FOLLOWER"),
                    lexicon_entry: None,
                });
            }
            last_non_ws = Some(end);
            i = end + 1;
            continue;
        }

        i += 1;
    }

    if let (Some(s), Some(e)) = (start, last_non_ws) {
        sentences.push(close(
            &chars,
            text,
            map,
            s,
            e,
            "SEG.BREAK.END_OF_TEXT",
            &mut suppressed,
        ));
    }
    sentences
}

fn close(
    chars: &[(usize, char)],
    _text: &str,
    map: &CharMap,
    start: usize,
    end: usize,
    rule: &str,
    suppressed: &mut Vec<SuppressedBreak>,
) -> SegmentedSentence {
    let byte_start = chars[start].0 as u32;
    let (last_byte, last_char) = chars[end];
    let byte_end = (last_byte + last_char.len_utf8()) as u32;
    let span = map.span(byte_start, byte_end);
    let mine: Vec<SuppressedBreak> = suppressed
        .iter()
        .filter(|s| span.contains(&s.span))
        .cloned()
        .collect();
    suppressed.retain(|s| !span.contains(&s.span));
    SegmentedSentence {
        span,
        rule: RuleId::new(rule),
        suppressed: mine,
    }
}

/// The whitespace-delimited word ending at `index` (inclusive), used for
/// abbreviation and initial checks.
fn word_ending_at<'a>(chars: &[(usize, char)], text: &'a str, index: usize) -> &'a str {
    let mut start = index;
    while start > 0 && !chars[start - 1].1.is_whitespace() {
        start -= 1;
    }
    let byte_start = chars[start].0;
    let (last_byte, last_char) = chars[index];
    &text[byte_start..last_byte + last_char.len_utf8()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> Vec<String> {
        let pack = RulePack::builtin().unwrap();
        let map = CharMap::new(text);
        segment(text, &pack, &map)
            .iter()
            .map(|s| s.span.slice(text).unwrap().to_string())
            .collect()
    }

    #[test]
    fn splits_on_terminal_punctuation() {
        assert_eq!(
            run("The cat are sleeping. There is many reasons."),
            vec!["The cat are sleeping.", "There is many reasons."]
        );
    }

    #[test]
    fn does_not_split_known_abbreviations() {
        assert_eq!(
            run("Dr. Smith arrived. He was late."),
            vec!["Dr. Smith arrived.", "He was late."]
        );
    }

    #[test]
    fn does_not_split_initials_or_decimals() {
        assert_eq!(run("J. R. R. Tolkien wrote it.").len(), 1);
        assert_eq!(run("It costs 3.50 today.").len(), 1);
    }

    #[test]
    fn splits_on_blank_line_without_punctuation() {
        assert_eq!(
            run("Chapter one\n\nIt was late"),
            vec!["Chapter one", "It was late"]
        );
    }

    #[test]
    fn keeps_closing_quote_with_the_sentence() {
        assert_eq!(
            run("\"Stop!\" She turned away."),
            vec!["\"Stop!\"", "She turned away."]
        );
    }

    #[test]
    fn suppressions_are_recorded_not_silent() {
        let pack = RulePack::builtin().unwrap();
        let text = "Dr. Smith arrived.";
        let map = CharMap::new(text);
        let sentences = segment(text, &pack, &map);
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].suppressed.len(), 1);
        assert_eq!(
            sentences[0].suppressed[0].rule.as_str(),
            "SEG.NOBREAK.ABBREVIATION"
        );
        assert_eq!(
            sentences[0].suppressed[0].lexicon_entry.as_deref(),
            Some("Dr.")
        );
    }

    #[test]
    fn spans_are_valid_against_the_source() {
        let text = "  Leading space. Then café, naïvely.  ";
        let pack = RulePack::builtin().unwrap();
        let map = CharMap::new(text);
        for sentence in segment(text, &pack, &map) {
            assert_eq!(sentence.span.validate(text), Ok(()));
        }
    }
}
