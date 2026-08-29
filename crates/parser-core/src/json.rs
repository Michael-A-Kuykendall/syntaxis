//! Canonical JSON.
//!
//! §4.2 requires byte-for-byte stable serialization. That rules out any writer
//! whose object order depends on a hash map. This one keeps object members in
//! the order the producer declared, and the producers in `serialize.rs` declare
//! a fixed order. There are no floats in the model, so there is no float
//! formatting ambiguity to worry about.

use std::fmt::Write as _;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Json {
    Null,
    Bool(bool),
    Uint(u64),
    Int(i64),
    Str(String),
    Array(Vec<Json>),
    /// Members in declared order. Duplicate keys are the producer's bug; the
    /// writer does not reorder or deduplicate.
    Object(Vec<(String, Json)>),
}

impl Json {
    pub fn str(s: impl Into<String>) -> Json {
        Json::Str(s.into())
    }

    pub fn object(members: Vec<(&str, Json)>) -> Json {
        Json::Object(
            members
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn array(items: impl IntoIterator<Item = Json>) -> Json {
        Json::Array(items.into_iter().collect())
    }

    /// Pretty form: two-space indent, LF line endings, no trailing whitespace,
    /// terminating newline. This is the form hashed in regression fixtures.
    pub fn to_canonical_string(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    /// Compact form: no insignificant whitespace. Same member order.
    pub fn to_compact_string(&self) -> String {
        let mut out = String::new();
        self.write_compact(&mut out);
        out
    }

    fn write(&self, out: &mut String, indent: usize) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Uint(n) => {
                let _ = write!(out, "{n}");
            }
            Json::Int(n) => {
                let _ = write!(out, "{n}");
            }
            Json::Str(s) => write_escaped(out, s),
            Json::Array(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    push_indent(out, indent + 1);
                    item.write(out, indent + 1);
                    if i + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                push_indent(out, indent);
                out.push(']');
            }
            Json::Object(members) => {
                if members.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push_str("{\n");
                for (i, (key, value)) in members.iter().enumerate() {
                    push_indent(out, indent + 1);
                    write_escaped(out, key);
                    out.push_str(": ");
                    value.write(out, indent + 1);
                    if i + 1 < members.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                push_indent(out, indent);
                out.push('}');
            }
        }
    }

    fn write_compact(&self, out: &mut String) {
        match self {
            Json::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write_compact(out);
                }
                out.push(']');
            }
            Json::Object(members) => {
                out.push('{');
                for (i, (key, value)) in members.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_escaped(out, key);
                    out.push(':');
                    value.write_compact(out);
                }
                out.push('}');
            }
            other => other.write(out, 0),
        }
    }
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

/// Escapes exactly what JSON requires and nothing more. Non-ASCII is emitted as
/// UTF-8 rather than `\u` escapes; this is stable and avoids surrogate-pair
/// ambiguity, and is recorded as part of the serialization contract.
fn write_escaped(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_order_is_preserved_not_sorted() {
        let j = Json::object(vec![("z", Json::Uint(1)), ("a", Json::Uint(2))]);
        assert_eq!(j.to_compact_string(), r#"{"z":1,"a":2}"#);
    }

    #[test]
    fn escapes_control_characters() {
        let j = Json::str("tab\there\u{1}\"quote\"");
        assert_eq!(j.to_compact_string(), r#""tab\there\u0001\"quote\"""#);
    }

    #[test]
    fn non_ascii_is_literal_utf8() {
        assert_eq!(Json::str("café").to_compact_string(), "\"café\"");
    }

    #[test]
    fn pretty_output_is_stable_and_newline_terminated() {
        let j = Json::object(vec![
            ("tokens", Json::array([Json::Uint(1), Json::Uint(2)])),
            ("empty", Json::Array(vec![])),
        ]);
        assert_eq!(
            j.to_canonical_string(),
            "{\n  \"tokens\": [\n    1,\n    2\n  ],\n  \"empty\": []\n}\n"
        );
    }
}
