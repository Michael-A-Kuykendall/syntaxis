//! Strict Universal Dependencies import and export.

#![forbid(unsafe_code)]

pub mod export;
pub mod import;
pub mod mapping;

pub use export::{export, export_with, ExportOptions};
pub use import::{import_str, ConlluError};
pub use mapping::MAPPING_VERSION;
