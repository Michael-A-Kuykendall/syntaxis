//! `conllu` — strict Universal Dependencies import and export.
//!
//! This crate is the interoperability boundary. It exists so the engine
//! can be measured against gold annotations and so its output can be read by
//! standard tooling, not so that UD becomes the internal model.

#![forbid(unsafe_code)]

pub mod export;
pub mod import;
pub mod mapping;

pub use export::{export, export_with, ExportOptions};
pub use import::{import_str, ConlluError};
pub use mapping::MAPPING_VERSION;
