//! Rule packs.
//!
//! A rule pack is a manifest plus a set of reference artifacts. Both are
//! embedded at build time and both are checksum-verified at load, so a build
//! either has the exact resources it claims or refuses to start. Nothing is
//! read from disk or the network at analysis time.
//!
//! The manifest also carries, per rule, the declarations: supported
//! constructions, known blind spots, and a precision target. Those
//! fields are deliberately allowed to say "not yet measured" — inventing a
//! threshold before the baseline exists is forbidden, and a manifest that lies
//! about it would be worse than one that admits it.

use crate::parser_core;
use parser_core::hash::sha256_hex;
use parser_core::ids::{RuleId, RulePackId, Version};
use std::collections::BTreeMap;
use std::fmt;

const MANIFEST_TEXT: &str = include_str!("../../resources/en/rulepack.manifest");
const ABBREVIATIONS_TEXT: &str = include_str!("../../resources/en/abbreviations.txt");
const CLITICS_TEXT: &str = include_str!("../../resources/en/clitics.txt");
const FUSED_TEXT: &str = include_str!("../../resources/en/fused.txt");

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ArtifactMeta {
    pub name: String,
    pub path: String,
    pub version: Version,
    pub license: String,
    pub source: String,
    pub normalization: String,
    pub generation: String,
    pub sha256: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RuleMeta {
    pub id: RuleId,
    pub stage: String,
    pub description: String,
    pub supports: String,
    pub blind_spots: String,
    pub precision_target: String,
}

#[derive(Clone, Debug)]
pub struct RulePack {
    pub id: RulePackId,
    pub name: String,
    pub version: Version,
    pub dialect: String,
    pub analysis_version: Version,
    pub conllu_mapping_version: Version,
    pub description: String,
    pub artifacts: BTreeMap<String, ArtifactMeta>,
    pub rules: BTreeMap<RuleId, RuleMeta>,
    pub abbreviations: Abbreviations,
    pub clitics: Clitics,
    pub fused: FusedForms,
}

impl RulePack {
    /// The built-in `en-core` pack. Fails loudly if an embedded artifact does
    /// not match the checksum recorded in the manifest.
    pub fn builtin() -> Result<RulePack, RulePackError> {
        let manifest = Manifest::parse(MANIFEST_TEXT)?;
        let pack = manifest.section("pack")?;

        let mut artifacts = BTreeMap::new();
        for (section, fields) in manifest.sections_with_prefix("artifact.") {
            let meta = ArtifactMeta {
                name: section.to_string(),
                path: field(fields, "path", section)?.to_string(),
                version: version_field(fields, "version", section)?,
                license: field(fields, "license", section)?.to_string(),
                source: field(fields, "source", section)?.to_string(),
                normalization: field(fields, "normalization", section)?.to_string(),
                generation: field(fields, "generation", section)?.to_string(),
                sha256: field(fields, "sha256", section)?.to_string(),
            };
            artifacts.insert(section.to_string(), meta);
        }

        let mut rules = BTreeMap::new();
        for (section, fields) in manifest.sections_with_prefix("rule.") {
            let id = RuleId::new(section);
            let meta = RuleMeta {
                id: id.clone(),
                stage: field(fields, "stage", section)?.to_string(),
                description: field(fields, "description", section)?.to_string(),
                supports: field(fields, "supports", section)?.to_string(),
                blind_spots: field(fields, "blind_spots", section)?.to_string(),
                precision_target: field(fields, "precision_target", section)?.to_string(),
            };
            rules.insert(id, meta);
        }

        for (name, text) in [
            ("abbreviations", ABBREVIATIONS_TEXT),
            ("clitics", CLITICS_TEXT),
            ("fused", FUSED_TEXT),
        ] {
            let meta = artifacts
                .get(name)
                .ok_or_else(|| RulePackError::MissingArtifact(name.to_string()))?;
            let actual = sha256_hex(text.as_bytes());
            if actual != meta.sha256 {
                return Err(RulePackError::ChecksumMismatch {
                    artifact: name.to_string(),
                    expected: meta.sha256.clone(),
                    actual,
                });
            }
        }

        let name = field(pack, "id", "pack")?.to_string();
        let version = version_field(pack, "version", "pack")?;
        Ok(RulePack {
            id: RulePackId::new(&format!("{name}@{version}")),
            name,
            version,
            dialect: field(pack, "dialect", "pack")?.to_string(),
            analysis_version: version_field(pack, "analysis_version", "pack")?,
            conllu_mapping_version: version_field(pack, "conllu_mapping_version", "pack")?,
            description: field(pack, "description", "pack")?.to_string(),
            abbreviations: Abbreviations::parse(ABBREVIATIONS_TEXT),
            clitics: Clitics::parse(CLITICS_TEXT),
            fused: FusedForms::parse(FUSED_TEXT)?,
            artifacts,
            rules,
        })
    }

    /// Look up a rule, returning an error rather than silently proceeding: a
    /// rule that fires without a manifest entry has no declared blind spots,
    /// which is exactly the situation the manifest exists to prevent.
    pub fn rule(&self, id: &str) -> Result<&RuleMeta, RulePackError> {
        self.rules
            .get(&RuleId::new(id))
            .ok_or_else(|| RulePackError::UndeclaredRule(id.to_string()))
    }

    pub fn artifact_version(&self, name: &str) -> Version {
        self.artifacts
            .get(name)
            .map(|a| a.version)
            .unwrap_or(Version::new(0, 0, 0))
    }
}

// ---------------------------------------------------------------------------
// Artifacts
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Abbreviations {
    entries: BTreeMap<String, ()>,
}

impl Abbreviations {
    fn parse(text: &str) -> Abbreviations {
        Abbreviations {
            entries: data_lines(text).map(|l| (l.to_string(), ())).collect(),
        }
    }
    pub fn contains(&self, surface: &str) -> bool {
        self.entries.contains_key(surface)
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Clitics {
    /// Longest-first so `n't` wins over `'t`-style shorter matches.
    suffixes: Vec<String>,
}

impl Clitics {
    fn parse(text: &str) -> Clitics {
        let mut suffixes: Vec<String> = data_lines(text).map(|l| l.to_lowercase()).collect();
        suffixes.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
        Clitics { suffixes }
    }

    /// Longest matching clitic suffix of `word_lower`, never the whole word.
    pub fn match_suffix(&self, word_lower: &str) -> Option<&str> {
        self.suffixes
            .iter()
            .find(|s| word_lower.len() > s.len() && word_lower.ends_with(s.as_str()))
            .map(|s| s.as_str())
    }

    /// Exact membership, used by the tokenizer when it compares a normalized
    /// tail against the artifact. Normalization means a curly apostrophe in the
    /// source still matches the ASCII entry here.
    pub fn contains(&self, normalized_tail: &str) -> bool {
        self.suffixes.iter().any(|s| s == normalized_tail)
    }

    /// Longest suffix length in characters, so the tokenizer knows how far back
    /// to look instead of guessing a constant.
    pub fn max_chars(&self) -> usize {
        self.suffixes
            .iter()
            .map(|s| s.chars().count())
            .max()
            .unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.suffixes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.suffixes.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct FusedForms {
    entries: BTreeMap<String, Vec<String>>,
}

impl FusedForms {
    fn parse(text: &str) -> Result<FusedForms, RulePackError> {
        let mut entries = BTreeMap::new();
        for line in data_lines(text) {
            let (surface, parts) =
                line.split_once('\t')
                    .ok_or_else(|| RulePackError::MalformedArtifact {
                        artifact: "fused".to_string(),
                        detail: format!("missing tab in `{line}`"),
                    })?;
            let parts: Vec<String> = parts.split('|').map(|p| p.to_string()).collect();
            let joined: String = parts.concat();
            // Without this check a fused entry could silently corrupt spans.
            if joined != surface {
                return Err(RulePackError::MalformedArtifact {
                    artifact: "fused".to_string(),
                    detail: format!("parts of `{surface}` concatenate to `{joined}`"),
                });
            }
            entries.insert(surface.to_lowercase(), parts);
        }
        Ok(FusedForms { entries })
    }

    pub fn split(&self, word_lower: &str) -> Option<&[String]> {
        self.entries.get(word_lower).map(|v| v.as_slice())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn data_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
}

// ---------------------------------------------------------------------------
// Manifest parsing
// ---------------------------------------------------------------------------

struct Manifest {
    sections: Vec<(String, BTreeMap<String, String>)>,
}

impl Manifest {
    fn parse(text: &str) -> Result<Manifest, RulePackError> {
        let mut sections: Vec<(String, BTreeMap<String, String>)> = Vec::new();
        let mut current: Option<(String, BTreeMap<String, String>)> = None;

        for (number, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                if let Some(section) = current.take() {
                    sections.push(section);
                }
                current = Some((name.trim().to_string(), BTreeMap::new()));
                continue;
            }
            let (key, value) = line.split_once('=').ok_or(RulePackError::ManifestSyntax {
                line: number + 1,
                detail: "expected `key = value`".to_string(),
            })?;
            let section = current.as_mut().ok_or(RulePackError::ManifestSyntax {
                line: number + 1,
                detail: "field outside any section".to_string(),
            })?;
            if section
                .1
                .insert(key.trim().to_string(), value.trim().to_string())
                .is_some()
            {
                return Err(RulePackError::ManifestSyntax {
                    line: number + 1,
                    detail: format!("duplicate key `{}`", key.trim()),
                });
            }
        }
        if let Some(section) = current.take() {
            sections.push(section);
        }
        Ok(Manifest { sections })
    }

    fn section(&self, name: &str) -> Result<&BTreeMap<String, String>, RulePackError> {
        self.sections
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, f)| f)
            .ok_or_else(|| RulePackError::MissingSection(name.to_string()))
    }

    fn sections_with_prefix<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = (&'a str, &'a BTreeMap<String, String>)> {
        self.sections
            .iter()
            .filter_map(move |(name, fields)| Some((name.strip_prefix(prefix)?, fields)))
    }
}

fn field<'a>(
    fields: &'a BTreeMap<String, String>,
    key: &str,
    section: &str,
) -> Result<&'a str, RulePackError> {
    fields
        .get(key)
        .map(|s| s.as_str())
        .ok_or_else(|| RulePackError::MissingField {
            section: section.to_string(),
            field: key.to_string(),
        })
}

fn version_field(
    fields: &BTreeMap<String, String>,
    key: &str,
    section: &str,
) -> Result<Version, RulePackError> {
    let raw = field(fields, key, section)?;
    Version::parse(raw).ok_or_else(|| RulePackError::BadVersion {
        section: section.to_string(),
        value: raw.to_string(),
    })
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RulePackError {
    ManifestSyntax {
        line: usize,
        detail: String,
    },
    MissingSection(String),
    MissingField {
        section: String,
        field: String,
    },
    BadVersion {
        section: String,
        value: String,
    },
    MissingArtifact(String),
    ChecksumMismatch {
        artifact: String,
        expected: String,
        actual: String,
    },
    MalformedArtifact {
        artifact: String,
        detail: String,
    },
    UndeclaredRule(String),
}

impl fmt::Display for RulePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RulePackError::*;
        match self {
            ManifestSyntax { line, detail } => write!(f, "manifest line {line}: {detail}"),
            MissingSection(s) => write!(f, "manifest is missing section [{s}]"),
            MissingField { section, field } => {
                write!(f, "manifest section [{section}] is missing `{field}`")
            }
            BadVersion { section, value } => {
                write!(f, "manifest section [{section}] has bad version `{value}`")
            }
            MissingArtifact(a) => write!(f, "artifact `{a}` is embedded but not declared"),
            ChecksumMismatch {
                artifact,
                expected,
                actual,
            } => write!(
                f,
                "artifact `{artifact}` checksum mismatch: manifest says {expected}, content is {actual}"
            ),
            MalformedArtifact { artifact, detail } => {
                write!(f, "artifact `{artifact}` is malformed: {detail}")
            }
            UndeclaredRule(r) => write!(f, "rule `{r}` fired but is not declared in the manifest"),
        }
    }
}

impl std::error::Error for RulePackError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_pack_loads_and_verifies() {
        let pack = RulePack::builtin().expect("builtin rule pack must load");
        assert_eq!(pack.id.as_str(), "en-core@0.1.0");
        assert_eq!(pack.dialect, "en-US");
        assert!(pack.abbreviations.contains("Dr."));
        assert!(!pack.abbreviations.contains("Zzz."));
        assert!(pack.clitics.len() >= 7);
        assert_eq!(
            pack.fused.split("cannot").map(|p| p.to_vec()),
            Some(vec!["can".to_string(), "not".to_string()])
        );
    }

    #[test]
    fn every_declared_rule_has_blind_spots_recorded() {
        let pack = RulePack::builtin().unwrap();
        assert!(!pack.rules.is_empty());
        for (id, rule) in &pack.rules {
            assert!(
                !rule.supports.trim().is_empty(),
                "{id} declares no supported constructions"
            );
            assert!(
                !rule.blind_spots.trim().is_empty(),
                "{id} declares no blind spots"
            );
            assert!(
                !rule.precision_target.trim().is_empty(),
                "{id} declares no precision target"
            );
        }
    }

    #[test]
    fn clitics_prefer_longest_match() {
        let pack = RulePack::builtin().unwrap();
        assert_eq!(pack.clitics.match_suffix("don't"), Some("n't"));
        assert_eq!(pack.clitics.match_suffix("cat's"), Some("'s"));
        // A bare clitic is not split off itself.
        assert_eq!(pack.clitics.match_suffix("'s"), None);
    }

    #[test]
    fn undeclared_rule_is_an_error() {
        let pack = RulePack::builtin().unwrap();
        assert!(pack.rule("TOK.WORD").is_ok());
        assert!(matches!(
            pack.rule("TOK.MADE.UP"),
            Err(RulePackError::UndeclaredRule(_))
        ));
    }

    #[test]
    fn fused_parts_must_reconstruct_the_surface() {
        let bad = "# header\nfoo\tf|zz\n";
        assert!(matches!(
            FusedForms::parse(bad),
            Err(RulePackError::MalformedArtifact { .. })
        ));
    }

    #[test]
    fn duplicate_manifest_keys_are_rejected() {
        let text = "[pack]\nid = a\nid = b\n";
        assert!(matches!(
            Manifest::parse(text),
            Err(RulePackError::ManifestSyntax { .. })
        ));
    }
}
