//! Small, I/O-free API boundary for native and browser consumers.

use crate::conllu::{export, import_str};
use crate::english_rules::{pipeline::analyze, rulepack::RulePack};
use std::fmt;

/// Maximum UTF-8 input accepted by the consumer API.
pub const MAX_INPUT_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    EmptyInput,
    InputTooLarge { bytes: usize, maximum: usize },
    RulePack(String),
    Conllu(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::EmptyInput => f.write_str("input must not be empty"),
            ApiError::InputTooLarge { bytes, maximum } => {
                write!(f, "input is {bytes} bytes; maximum is {maximum} bytes")
            }
            ApiError::RulePack(error) => write!(f, "rule pack: {error}"),
            ApiError::Conllu(error) => write!(f, "CoNLL-U: {error}"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Analyze plain English text and return canonical JSON.
///
/// The input is bounded by [`MAX_INPUT_BYTES`]. This function performs the
/// engine analysis; it does not read files or import an external annotation.
pub fn analyze_json(text: &str) -> Result<String, ApiError> {
    let pack = builtin_pack()?;
    Ok(analyze_checked(text, &pack)?.to_canonical_json())
}

/// Analyze plain English text and return its canonical-output digest.
pub fn digest(text: &str) -> Result<String, ApiError> {
    let pack = builtin_pack()?;
    Ok(analyze_checked(text, &pack)?.digest())
}

/// Import strict CoNLL-U and return the imported analysis as canonical JSON.
///
/// Imported annotations are preserved and marked as imported evidence; grammar
/// diagnostics are not inferred from the imported rows.
pub fn import_conllu_json(input: &str) -> Result<String, ApiError> {
    check_input(input)?;
    let pack = builtin_pack()?;
    import_str(input, &pack.id)
        .map(|analysis| analysis.to_canonical_json())
        .map_err(|error| ApiError::Conllu(error.to_string()))
}

/// Analyze plain English text and export the resulting snapshot as CoNLL-U.
pub fn analyze_conllu(text: &str) -> Result<String, ApiError> {
    let pack = builtin_pack()?;
    Ok(export(&analyze_checked(text, &pack)?))
}

fn builtin_pack() -> Result<RulePack, ApiError> {
    RulePack::builtin().map_err(|error| ApiError::RulePack(error.to_string()))
}

fn analyze_checked(text: &str, pack: &RulePack) -> Result<crate::Analysis, ApiError> {
    check_input(text)?;
    Ok(analyze(text, pack))
}

fn check_input(input: &str) -> Result<(), ApiError> {
    if input.is_empty() {
        return Err(ApiError::EmptyInput);
    }
    if input.len() > MAX_INPUT_BYTES {
        return Err(ApiError::InputTooLarge {
            bytes: input.len(),
            maximum: MAX_INPUT_BYTES,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_rejects_empty_and_oversized_input() {
        assert_eq!(analyze_json(""), Err(ApiError::EmptyInput));
        assert!(matches!(
            analyze_json(&"a".repeat(MAX_INPUT_BYTES + 1)),
            Err(ApiError::InputTooLarge { .. })
        ));
    }

    #[test]
    fn api_output_and_digest_are_repeatable() {
        let text = "The cat are sleeping.";
        assert_eq!(analyze_json(text), analyze_json(text));
        assert_eq!(digest(text), digest(text));
        assert!(analyze_conllu(text).unwrap().contains("\troot\t"));
    }

    #[test]
    fn conllu_api_reports_malformed_input_without_panicking() {
        let result = std::panic::catch_unwind(|| import_conllu_json("1\tbroken\n"));
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Err(ApiError::Conllu(_))));
    }
}
