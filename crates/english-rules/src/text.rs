//! Text handling primitives: the normalization policy and the byte/char map.
//!
//! The document text is never rewritten. Normalization produces a *separate*
//! lookup form stored alongside the untouched surface, so spans always index
//! the original bytes: source offsets are never discarded.

use parser_core::span::Span;

/// Version of the normalization policy below. Changing the policy changes
/// lookup behaviour and therefore requires a bump plus a rule-pack bump.
pub const NORMALIZATION_POLICY_VERSION: &str = "0.1.0";

/// Produce the lookup form of a surface string.
///
/// Policy, in order:
/// 1. fold typographic punctuation to its ASCII equivalent;
/// 2. fold non-breaking and thin spaces to a normal space;
/// 3. lowercase using Unicode simple lowercasing.
///
/// Not done, deliberately: NFC/NFD normalization (would need a dependency or a
/// large embedded table; recorded as a known gap in docs/RESOURCES.md),
/// diacritic stripping, and any spelling change.
pub fn normalize(surface: &str) -> String {
    let mut out = String::with_capacity(surface.len());
    for c in surface.chars() {
        match c {
            '\u{2018}' | '\u{2019}' | '\u{201B}' | '\u{02BC}' => out.push('\''),
            '\u{201C}' | '\u{201D}' | '\u{201F}' => out.push('"'),
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}' => {
                out.push('-')
            }
            '\u{2026}' => out.push_str("..."),
            '\u{00A0}' | '\u{2007}' | '\u{202F}' | '\u{2009}' => out.push(' '),
            c => {
                for lowered in c.to_lowercase() {
                    out.push(lowered);
                }
            }
        }
    }
    out
}

/// Byte offset -> character offset lookup, built once per document.
///
/// Spans must carry both, and recomputing character offsets by rescanning the
/// prefix for every token would be quadratic.
#[derive(Clone, Debug)]
pub struct CharMap {
    /// Byte offset of each character, plus a terminating text length.
    offsets: Vec<u32>,
}

impl CharMap {
    pub fn new(text: &str) -> CharMap {
        let mut offsets: Vec<u32> = text.char_indices().map(|(i, _)| i as u32).collect();
        offsets.push(text.len() as u32);
        CharMap { offsets }
    }

    /// Character index of a byte offset that lies on a character boundary.
    pub fn char_index(&self, byte: u32) -> u32 {
        match self.offsets.binary_search(&byte) {
            Ok(index) => index as u32,
            // Interior of a multi-byte character: report the character it is
            // inside, which keeps the result monotone rather than panicking.
            Err(index) => index.saturating_sub(1) as u32,
        }
    }

    pub fn span(&self, byte_start: u32, byte_end: u32) -> Span {
        Span::new(
            byte_start,
            byte_end,
            self.char_index(byte_start),
            self.char_index(byte_end),
        )
    }

    pub fn char_count(&self) -> usize {
        self.offsets.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_folds_typography_but_not_letters() {
        assert_eq!(normalize("Don\u{2019}t"), "don't");
        assert_eq!(normalize("\u{201C}Hi\u{201D}"), "\"hi\"");
        assert_eq!(normalize("well\u{2014}known"), "well-known");
        assert_eq!(normalize("caf\u{00E9}"), "café");
    }

    #[test]
    fn charmap_handles_multibyte() {
        let text = "naïve café";
        let map = CharMap::new(text);
        assert_eq!(map.char_count(), 10);
        let span = map.span(7, 12);
        assert_eq!(span.slice(text), Some("café"));
        assert_eq!(span.validate(text), Ok(()));
    }
}
