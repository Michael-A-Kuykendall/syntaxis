//! Deterministic English normalization, segmentation, tokenization, and rules.

#![forbid(unsafe_code)]

pub mod evaluation;
pub mod grammar;
pub mod parser;
pub mod pipeline;
pub mod pos;
pub mod rulepack;
pub mod segment;
pub mod text;
pub mod tokenize;
