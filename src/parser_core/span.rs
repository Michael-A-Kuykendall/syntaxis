//! Source spans.
//!
//! A span carries **both** byte offsets (for Rust slicing and for LSP-style
//! consumers that speak UTF-8) and character offsets (for consumers that count
//! scalar values). Carrying both is deliberate: recomputing one from the other
//! requires the original text, and supports must be interpretable without it.

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Span {
    pub byte_start: u32,
    pub byte_end: u32,
    pub char_start: u32,
    pub char_end: u32,
}

impl Span {
    pub const fn new(byte_start: u32, byte_end: u32, char_start: u32, char_end: u32) -> Self {
        Span {
            byte_start,
            byte_end,
            char_start,
            char_end,
        }
    }

    pub const fn byte_len(&self) -> u32 {
        self.byte_end.saturating_sub(self.byte_start)
    }

    pub const fn char_len(&self) -> u32 {
        self.char_end.saturating_sub(self.char_start)
    }

    pub const fn is_empty(&self) -> bool {
        self.byte_start == self.byte_end
    }

    /// Smallest span covering both inputs. Used for sentence and phrase spans.
    pub fn cover(&self, other: &Span) -> Span {
        Span {
            byte_start: self.byte_start.min(other.byte_start),
            byte_end: self.byte_end.max(other.byte_end),
            char_start: self.char_start.min(other.char_start),
            char_end: self.char_end.max(other.char_end),
        }
    }

    pub fn contains(&self, other: &Span) -> bool {
        self.byte_start <= other.byte_start && other.byte_end <= self.byte_end
    }

    /// Slice the originating text. Returns `None` rather than panicking when
    /// the span does not land on character boundaries, so validators can report
    /// it as a data error instead of aborting.
    pub fn slice<'a>(&self, text: &'a str) -> Option<&'a str> {
        text.get(self.byte_start as usize..self.byte_end as usize)
    }

    /// Structural validity, independent of any particular text.
    pub fn is_well_formed(&self) -> bool {
        self.byte_start <= self.byte_end && self.char_start <= self.char_end
    }

    /// Validity against the text it claims to index, including the invariant
    /// that the char offsets really do describe the same region as the bytes.
    pub fn validate(&self, text: &str) -> Result<(), SpanError> {
        if !self.is_well_formed() {
            return Err(SpanError::Inverted(*self));
        }
        let s = self
            .slice(text)
            .ok_or(SpanError::NotOnCharBoundary(*self))?;
        if s.chars().count() as u32 != self.char_len() {
            return Err(SpanError::CharCountMismatch(*self));
        }
        let prefix = text
            .get(..self.byte_start as usize)
            .ok_or(SpanError::NotOnCharBoundary(*self))?;
        if prefix.chars().count() as u32 != self.char_start {
            return Err(SpanError::CharStartMismatch(*self));
        }
        Ok(())
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.byte_start, self.byte_end)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpanError {
    Inverted(Span),
    NotOnCharBoundary(Span),
    CharCountMismatch(Span),
    CharStartMismatch(Span),
}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpanError::Inverted(s) => write!(f, "span {s} has end before start"),
            SpanError::NotOnCharBoundary(s) => write!(f, "span {s} is not on a char boundary"),
            SpanError::CharCountMismatch(s) => {
                write!(f, "span {s} char length disagrees with byte length")
            }
            SpanError::CharStartMismatch(s) => {
                write!(f, "span {s} char start disagrees with byte start")
            }
        }
    }
}

impl std::error::Error for SpanError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multibyte_span_validates() {
        let text = "naïve café";
        // "café" -> bytes 7..12 (é is 2 bytes), chars 6..10
        let span = Span::new(7, 12, 6, 10);
        assert_eq!(span.slice(text), Some("café"));
        assert_eq!(span.validate(text), Ok(()));
    }

    #[test]
    fn wrong_char_offsets_are_rejected() {
        let text = "naïve café";
        let span = Span::new(7, 12, 7, 11);
        assert!(matches!(
            span.validate(text),
            Err(SpanError::CharStartMismatch(_))
        ));
    }

    #[test]
    fn cover_is_a_union() {
        let a = Span::new(0, 3, 0, 3);
        let b = Span::new(10, 14, 10, 14);
        assert_eq!(a.cover(&b), Span::new(0, 14, 0, 14));
    }
}
