//! Stable identifiers.
//!
//! Every id in this module is *positional*, not allocated from a counter that
//! depends on execution order. Given the same input bytes and the same engine
//! version, the same id denotes the same thing. This is what makes
//! [`crate::support::SupportSet`] portable across runs and across processes.

use std::fmt;

macro_rules! numeric_id {
    ($name:ident, $prefix:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
        pub struct $name(pub u32);

        impl $name {
            pub const fn index(self) -> usize {
                self.0 as usize
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", $prefix, self.0)
            }
        }
    };
}

numeric_id!(DocumentId, "d", "Identifies one analysed document.");
numeric_id!(
    SentenceId,
    "s",
    "Zero-based sentence ordinal inside a document."
);
numeric_id!(
    TokenId,
    "t",
    "Zero-based token ordinal inside a document (not inside a sentence)."
);
numeric_id!(ArcId, "a", "Dependency arc identity inside an analysis.");
numeric_id!(DiagnosticId, "g", "Grammar diagnostic identity.");
numeric_id!(AlternativeGroupId, "x", "Alternative group identity.");

/// Identifier of a single rule, e.g. `TOK.CONTRACTION.NT` or `AGR.SV.NUMBER`.
///
/// Rule ids are part of the public contract: they appear in serialized output,
/// in supports, and in message keys. Renaming one is a breaking change.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RuleId(pub String);

impl RuleId {
    pub fn new(s: &str) -> Self {
        RuleId(s.to_string())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifier of the rule pack a derivation came from, e.g. `en-core@0.1.0`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RulePackId(pub String);

impl RulePackId {
    pub fn new(s: &str) -> Self {
        RulePackId(s.to_string())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RulePackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Stable, locale-independent key for a diagnostic message.
///
/// The engine never emits prose. Rendering lives in the consumer so that
/// message wording can change without changing engine output bytes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MessageKey(pub String);

impl MessageKey {
    pub fn new(s: &str) -> Self {
        MessageKey(s.to_string())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Semantic version of an engine component or resource artifact.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Option<Version> {
        let mut parts = s.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Version {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_roundtrip() {
        let v = Version::new(1, 2, 3);
        assert_eq!(Version::parse(&v.to_string()), Some(v));
        assert_eq!(Version::parse("1.2"), None);
        assert_eq!(Version::parse("1.2.3.4"), None);
    }

    #[test]
    fn ids_display_with_prefix() {
        assert_eq!(TokenId(7).to_string(), "t7");
        assert_eq!(SentenceId(0).to_string(), "s0");
    }
}
