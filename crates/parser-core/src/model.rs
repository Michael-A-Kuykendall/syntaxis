//! The frozen data model.
//!
//! M0 freezes these shapes. Adding a variant to a closed enum below is a
//! breaking change to serialized output and requires a bump of
//! `analysis_version`.

use crate::ids::*;
use crate::span::Span;
use crate::support::SupportSet;
use std::fmt;

// ---------------------------------------------------------------------------
// Part of speech
// ---------------------------------------------------------------------------

/// Penn-style tag set, which is the engine's internal working tag set.
///
/// The mapping to UD is versioned separately (`conllu_mapping_version`) because
/// it is a lossy, contestable projection and consumers must be able to pin it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Pos {
    CC,
    CD,
    DT,
    EX,
    FW,
    IN,
    JJ,
    JJR,
    JJS,
    LS,
    MD,
    NN,
    NNS,
    NNP,
    NNPS,
    PDT,
    POS,
    PRP,
    PRPS,
    RB,
    RBR,
    RBS,
    RP,
    SYM,
    TO,
    UH,
    VB,
    VBD,
    VBG,
    VBN,
    VBP,
    VBZ,
    WDT,
    WP,
    WPS,
    WRB,
    /// Sentence-final punctuation.
    PunctSent,
    /// Comma.
    PunctComma,
    /// Colon, semicolon, dash used as separator.
    PunctColon,
    PunctLeftBracket,
    PunctRightBracket,
    PunctOpenQuote,
    PunctCloseQuote,
    PunctHyph,
    PunctOther,
    /// Deliberately unresolved. Never a silent default: a token carrying
    /// `Unknown` must still carry the supports that failed to resolve it.
    Unknown,
}

impl Pos {
    pub fn as_str(self) -> &'static str {
        use Pos::*;
        match self {
            CC => "CC",
            CD => "CD",
            DT => "DT",
            EX => "EX",
            FW => "FW",
            IN => "IN",
            JJ => "JJ",
            JJR => "JJR",
            JJS => "JJS",
            LS => "LS",
            MD => "MD",
            NN => "NN",
            NNS => "NNS",
            NNP => "NNP",
            NNPS => "NNPS",
            PDT => "PDT",
            POS => "POS",
            PRP => "PRP",
            PRPS => "PRP$",
            RB => "RB",
            RBR => "RBR",
            RBS => "RBS",
            RP => "RP",
            SYM => "SYM",
            TO => "TO",
            UH => "UH",
            VB => "VB",
            VBD => "VBD",
            VBG => "VBG",
            VBN => "VBN",
            VBP => "VBP",
            VBZ => "VBZ",
            WDT => "WDT",
            WP => "WP",
            WPS => "WP$",
            WRB => "WRB",
            PunctSent => ".",
            PunctComma => ",",
            PunctColon => ":",
            PunctLeftBracket => "-LRB-",
            PunctRightBracket => "-RRB-",
            PunctOpenQuote => "``",
            PunctCloseQuote => "''",
            PunctHyph => "HYPH",
            PunctOther => "NFP",
            Unknown => "X",
        }
    }

    pub fn parse(s: &str) -> Option<Pos> {
        use Pos::*;
        Some(match s {
            "CC" => CC,
            "CD" => CD,
            "DT" => DT,
            "EX" => EX,
            "FW" => FW,
            "IN" => IN,
            "JJ" => JJ,
            "JJR" => JJR,
            "JJS" => JJS,
            "LS" => LS,
            "MD" => MD,
            "NN" => NN,
            "NNS" => NNS,
            "NNP" => NNP,
            "NNPS" => NNPS,
            "PDT" => PDT,
            "POS" => POS,
            "PRP" => PRP,
            "PRP$" => PRPS,
            "RB" => RB,
            "RBR" => RBR,
            "RBS" => RBS,
            "RP" => RP,
            "SYM" => SYM,
            "TO" => TO,
            "UH" => UH,
            "VB" => VB,
            "VBD" => VBD,
            "VBG" => VBG,
            "VBN" => VBN,
            "VBP" => VBP,
            "VBZ" => VBZ,
            "WDT" => WDT,
            "WP" => WP,
            "WP$" => WPS,
            "WRB" => WRB,
            "." => PunctSent,
            "," => PunctComma,
            ":" => PunctColon,
            "-LRB-" => PunctLeftBracket,
            "-RRB-" => PunctRightBracket,
            "``" => PunctOpenQuote,
            "''" => PunctCloseQuote,
            "HYPH" => PunctHyph,
            "NFP" => PunctOther,
            "X" => Unknown,
            _ => return None,
        })
    }

    pub fn is_verbal(self) -> bool {
        matches!(
            self,
            Pos::VB | Pos::VBD | Pos::VBG | Pos::VBN | Pos::VBP | Pos::VBZ | Pos::MD
        )
    }

    pub fn is_nominal(self) -> bool {
        matches!(
            self,
            Pos::NN | Pos::NNS | Pos::NNP | Pos::NNPS | Pos::PRP | Pos::WP
        )
    }

    pub fn is_punct(self) -> bool {
        matches!(
            self,
            Pos::PunctSent
                | Pos::PunctComma
                | Pos::PunctColon
                | Pos::PunctLeftBracket
                | Pos::PunctRightBracket
                | Pos::PunctOpenQuote
                | Pos::PunctCloseQuote
                | Pos::PunctHyph
                | Pos::PunctOther
        )
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Universal POS, produced only by the versioned projection in `conllu`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum UPos {
    ADJ,
    ADP,
    ADV,
    AUX,
    CCONJ,
    DET,
    INTJ,
    NOUN,
    NUM,
    PART,
    PRON,
    PROPN,
    PUNCT,
    SCONJ,
    SYM,
    VERB,
    X,
}

impl UPos {
    pub fn as_str(self) -> &'static str {
        use UPos::*;
        match self {
            ADJ => "ADJ",
            ADP => "ADP",
            ADV => "ADV",
            AUX => "AUX",
            CCONJ => "CCONJ",
            DET => "DET",
            INTJ => "INTJ",
            NOUN => "NOUN",
            NUM => "NUM",
            PART => "PART",
            PRON => "PRON",
            PROPN => "PROPN",
            PUNCT => "PUNCT",
            SCONJ => "SCONJ",
            SYM => "SYM",
            VERB => "VERB",
            X => "X",
        }
    }

    pub fn parse(s: &str) -> Option<UPos> {
        use UPos::*;
        Some(match s {
            "ADJ" => ADJ,
            "ADP" => ADP,
            "ADV" => ADV,
            "AUX" => AUX,
            "CCONJ" => CCONJ,
            "DET" => DET,
            "INTJ" => INTJ,
            "NOUN" => NOUN,
            "NUM" => NUM,
            "PART" => PART,
            "PRON" => PRON,
            "PROPN" => PROPN,
            "PUNCT" => PUNCT,
            "SCONJ" => SCONJ,
            "SYM" => SYM,
            "VERB" => VERB,
            "X" => X,
            _ => return None,
        })
    }
}

impl fmt::Display for UPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Morphology
// ---------------------------------------------------------------------------

macro_rules! simple_feature {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
        pub enum $name {
            #[default]
            Unknown,
            $($variant),+
        }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self {
                    $name::Unknown => "Unknown",
                    $($name::$variant => $text),+
                }
            }
            pub fn parse(s: &str) -> Option<$name> {
                Some(match s {
                    "Unknown" => $name::Unknown,
                    $($text => $name::$variant,)+
                    _ => return None,
                })
            }
            pub fn is_known(self) -> bool { self != $name::Unknown }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

simple_feature!(Number { Sing => "Sing", Plur => "Plur" });
simple_feature!(Person { First => "1", Second => "2", Third => "3" });
simple_feature!(Tense { Pres => "Pres", Past => "Past", Fut => "Fut" });
simple_feature!(VerbForm {
    Fin => "Fin",
    Inf => "Inf",
    Part => "Part",
    Ger => "Ger",
});
simple_feature!(DetKind {
    Article => "Art",
    Demonstrative => "Dem",
    Quantifier => "Ind",
    Total => "Tot",
    Negative => "Neg",
    Interrogative => "Int",
    Relative => "Rel",
    Possessive => "Prs",
});
simple_feature!(PronKind {
    Personal => "Prs",
    Demonstrative => "Dem",
    Indefinite => "Ind",
    Total => "Tot",
    Negative => "Neg",
    Interrogative => "Int",
    Relative => "Rel",
    Reciprocal => "Rcp",
    Expletive => "Exp",
});

/// Observable morphology. Absent information is `Unknown`, never guessed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Morphology {
    pub number: Number,
    pub person: Person,
    pub tense: Tense,
    pub verb_form: VerbForm,
    pub det_kind: DetKind,
    pub pron_kind: PronKind,
}

impl Morphology {
    pub fn is_empty(&self) -> bool {
        *self == Morphology::default()
    }

    /// UD FEATS string in canonical (alphabetical) feature order.
    ///
    /// `PronType` is projected from `pron_kind` when known, otherwise from
    /// `det_kind`. That precedence is part of the CoNLL-U mapping version.
    pub fn to_feats(&self) -> String {
        let mut feats: Vec<String> = Vec::new();
        if self.number.is_known() {
            feats.push(format!("Number={}", self.number));
        }
        if self.person.is_known() {
            feats.push(format!("Person={}", self.person));
        }
        let pron_type = if self.pron_kind.is_known() {
            Some(self.pron_kind.as_str())
        } else if self.det_kind.is_known() {
            Some(self.det_kind.as_str())
        } else {
            None
        };
        if let Some(pt) = pron_type {
            feats.push(format!("PronType={pt}"));
        }
        if self.tense.is_known() {
            feats.push(format!("Tense={}", self.tense));
        }
        if self.verb_form.is_known() {
            feats.push(format!("VerbForm={}", self.verb_form));
        }
        if feats.is_empty() {
            "_".to_string()
        } else {
            feats.join("|")
        }
    }

    /// Parse a UD FEATS string. Unknown features are reported, not dropped
    /// silently, so import stays strict.
    pub fn from_feats(s: &str) -> Result<(Morphology, Vec<String>), String> {
        let mut m = Morphology::default();
        let mut unmapped = Vec::new();
        if s == "_" || s.is_empty() {
            return Ok((m, unmapped));
        }
        for feat in s.split('|') {
            let (key, value) = feat
                .split_once('=')
                .ok_or_else(|| format!("malformed feature `{feat}`"))?;
            match key {
                "Number" => {
                    m.number = Number::parse(value).ok_or_else(|| format!("bad Number={value}"))?
                }
                "Person" => {
                    m.person = Person::parse(value).ok_or_else(|| format!("bad Person={value}"))?
                }
                "Tense" => {
                    m.tense = Tense::parse(value).ok_or_else(|| format!("bad Tense={value}"))?
                }
                "VerbForm" => {
                    m.verb_form =
                        VerbForm::parse(value).ok_or_else(|| format!("bad VerbForm={value}"))?
                }
                "PronType" => {
                    // Ambiguous target; recorded on both when it parses for both.
                    if let Some(p) = PronKind::parse(value) {
                        m.pron_kind = p;
                    }
                    if let Some(d) = DetKind::parse(value) {
                        m.det_kind = d;
                    }
                    if !m.pron_kind.is_known() && !m.det_kind.is_known() {
                        return Err(format!("bad PronType={value}"));
                    }
                }
                _ => unmapped.push(feat.to_string()),
            }
        }
        Ok((m, unmapped))
    }
}

// ---------------------------------------------------------------------------
// Relations
// ---------------------------------------------------------------------------

/// The first-gate relation set. Deliberately smaller than UD.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Relation {
    Root,
    Nsubj,
    NsubjPass,
    Obj,
    Iobj,
    Aux,
    AuxPass,
    Cop,
    Neg,
    Det,
    Amod,
    Advmod,
    Prep,
    Pobj,
    Compound,
    Conj,
    Cc,
    Mark,
    Advcl,
    Acl,
    Ccomp,
    Xcomp,
    Punct,
    /// A relation outside the supported set. The original label, if any, is
    /// preserved on the arc in `raw_label`. Never guessed into a supported one.
    Unsupported,
}

impl Relation {
    pub fn as_str(self) -> &'static str {
        use Relation::*;
        match self {
            Root => "root",
            Nsubj => "nsubj",
            NsubjPass => "nsubj:pass",
            Obj => "obj",
            Iobj => "iobj",
            Aux => "aux",
            AuxPass => "aux:pass",
            Cop => "cop",
            Neg => "neg",
            Det => "det",
            Amod => "amod",
            Advmod => "advmod",
            Prep => "prep",
            Pobj => "pobj",
            Compound => "compound",
            Conj => "conj",
            Cc => "cc",
            Mark => "mark",
            Advcl => "advcl",
            Acl => "acl",
            Ccomp => "ccomp",
            Xcomp => "xcomp",
            Punct => "punct",
            Unsupported => "dep:unsupported",
        }
    }

    pub fn parse(s: &str) -> Option<Relation> {
        use Relation::*;
        Some(match s {
            "root" => Root,
            "nsubj" => Nsubj,
            "nsubj:pass" => NsubjPass,
            "obj" => Obj,
            "iobj" => Iobj,
            "aux" => Aux,
            "aux:pass" => AuxPass,
            "cop" => Cop,
            "neg" => Neg,
            "det" => Det,
            "amod" => Amod,
            "advmod" => Advmod,
            "prep" => Prep,
            "pobj" => Pobj,
            "compound" => Compound,
            "conj" => Conj,
            "cc" => Cc,
            "mark" => Mark,
            "advcl" => Advcl,
            "acl" => Acl,
            "ccomp" => Ccomp,
            "xcomp" => Xcomp,
            "punct" => Punct,
            "dep:unsupported" => Unsupported,
            _ => return None,
        })
    }

    pub const ALL_SUPPORTED: [Relation; 23] = [
        Relation::Root,
        Relation::Nsubj,
        Relation::NsubjPass,
        Relation::Obj,
        Relation::Iobj,
        Relation::Aux,
        Relation::AuxPass,
        Relation::Cop,
        Relation::Neg,
        Relation::Det,
        Relation::Amod,
        Relation::Advmod,
        Relation::Prep,
        Relation::Pobj,
        Relation::Compound,
        Relation::Conj,
        Relation::Cc,
        Relation::Mark,
        Relation::Advcl,
        Relation::Acl,
        Relation::Ccomp,
        Relation::Xcomp,
        Relation::Punct,
    ];
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Structural items
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Sentence {
    pub id: SentenceId,
    pub ordinal: u32,
    pub span: Span,
    pub tokens: Vec<TokenId>,
    pub support: SupportSet,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    pub id: TokenId,
    pub sentence: SentenceId,
    /// Position within the sentence, zero-based.
    pub ordinal: u32,
    pub span: Span,
    /// Exact input substring. Never modified.
    pub surface: String,
    /// Lookup form. The normalization policy is versioned; see
    /// `docs/RESOURCES.md`.
    pub normalized: String,
    /// False when the next token starts immediately at this token's byte end.
    /// Recorded because it is needed to rebuild text and to export CoNLL-U.
    pub space_after: bool,
    pub support: SupportSet,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TokenAnalysis {
    pub token: TokenId,
    pub pos: Pos,
    /// Universal POS. Derived from `pos` by the versioned projection when the
    /// engine assigns it, or taken verbatim from an imported annotation. Held
    /// explicitly rather than re-derived on demand because the Penn tag does
    /// not determine it (VB* is AUX or VERB depending on the clause).
    pub upos: UPos,
    pub lemma: String,
    pub morphology: Morphology,
    /// Features seen on import that this model does not represent. Kept so
    /// round-tripping does not silently lose data.
    pub unmapped_features: Vec<String>,
    /// MISC entries the model does not represent, preserved verbatim in order.
    pub unmapped_misc: Vec<String>,
    pub support: SupportSet,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ArcStatus {
    Accepted,
    Alternative { group: AlternativeGroupId },
    Rejected,
    Unsupported,
}

impl ArcStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArcStatus::Accepted => "accepted",
            ArcStatus::Alternative { .. } => "alternative",
            ArcStatus::Rejected => "rejected",
            ArcStatus::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DependencyArc {
    pub id: ArcId,
    pub sentence: SentenceId,
    /// `None` only for the artificial root attachment (UD HEAD = 0).
    pub head: Option<TokenId>,
    pub dependent: TokenId,
    pub relation: Relation,
    /// Original label when `relation` is `Unsupported`.
    pub raw_label: Option<String>,
    pub status: ArcStatus,
    pub support: SupportSet,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Resolution {
    /// Exactly one candidate survived hard constraints.
    Unique,
    /// Several survive; ordered by declared deterministic precedence.
    Ranked,
    /// Several survive and precedence does not separate them.
    Ambiguous,
    /// No rule in the pack safely covers this construction.
    Unsupported,
}

impl Resolution {
    pub fn as_str(self) -> &'static str {
        match self {
            Resolution::Unique => "unique",
            Resolution::Ranked => "ranked",
            Resolution::Ambiguous => "ambiguous",
            Resolution::Unsupported => "unsupported",
        }
    }

    /// True when a diagnostic resting on this group must inherit uncertainty
    /// (§7: a diagnostic may not be promoted to definite on unresolved support).
    pub fn is_uncertain(self) -> bool {
        matches!(self, Resolution::Ambiguous | Resolution::Unsupported)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlternativeGroup {
    pub id: AlternativeGroupId,
    pub sentence: SentenceId,
    /// Ordered: for `Ranked`, index 0 is the preferred candidate.
    pub members: Vec<ArcId>,
    pub resolution: Resolution,
    pub support: SupportSet,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DiagnosticKind {
    /// Subject-verb or coordination number/person agreement.
    Agreement,
    /// Auxiliary/participle or complement form compatibility.
    VerbForm,
    /// Determiner/article compatibility.
    Determiner,
    /// Negation or auxiliary placement.
    Placement,
}

impl DiagnosticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticKind::Agreement => "agreement",
            DiagnosticKind::VerbForm => "verb_form",
            DiagnosticKind::Determiner => "determiner",
            DiagnosticKind::Placement => "placement",
        }
    }
}

/// Confidence carried by a diagnostic, propagated from the arcs it rests on.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Certainty {
    /// All supporting arcs are `Accepted` in `Unique` groups.
    Definite,
    /// At least one supporting arc sits in a `Ranked` group.
    Preferred,
    /// At least one supporting arc sits in an `Ambiguous` or `Unsupported`
    /// group.
    Conditional,
}

impl Certainty {
    pub fn as_str(self) -> &'static str {
        match self {
            Certainty::Definite => "definite",
            Certainty::Preferred => "preferred",
            Certainty::Conditional => "conditional",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Replacement {
    pub span: Span,
    pub text: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GrammarDiagnostic {
    pub id: DiagnosticId,
    pub sentence: SentenceId,
    pub kind: DiagnosticKind,
    pub span: Span,
    pub message_key: MessageKey,
    pub certainty: Certainty,
    pub replacements: Vec<Replacement>,
    pub support: SupportSet,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Document {
    pub id: DocumentId,
    pub text: String,
    pub sentences: Vec<SentenceId>,
    pub analysis_version: Version,
    pub rule_pack: RulePackId,
    /// Version of the Penn/internal -> UD projection used for CoNLL-U output.
    pub conllu_mapping_version: Version,
    /// Dialect configuration name, e.g. `en-US`.
    pub dialect: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feats_roundtrip_alphabetical() {
        let m = Morphology {
            number: Number::Sing,
            person: Person::Third,
            tense: Tense::Pres,
            verb_form: VerbForm::Fin,
            ..Morphology::default()
        };
        assert_eq!(m.to_feats(), "Number=Sing|Person=3|Tense=Pres|VerbForm=Fin");
        let (back, unmapped) = Morphology::from_feats(&m.to_feats()).unwrap();
        assert_eq!(back, m);
        assert!(unmapped.is_empty());
    }

    #[test]
    fn empty_feats_is_underscore() {
        assert_eq!(Morphology::default().to_feats(), "_");
        assert_eq!(
            Morphology::from_feats("_").unwrap().0,
            Morphology::default()
        );
    }

    #[test]
    fn unknown_features_are_retained_not_dropped() {
        let (m, unmapped) = Morphology::from_feats("Number=Plur|Gender=Fem").unwrap();
        assert_eq!(m.number, Number::Plur);
        assert_eq!(unmapped, vec!["Gender=Fem".to_string()]);
    }

    #[test]
    fn all_supported_relations_roundtrip() {
        for r in Relation::ALL_SUPPORTED {
            assert_eq!(Relation::parse(r.as_str()), Some(r));
        }
    }

    #[test]
    fn all_pos_tags_roundtrip() {
        for tag in [
            "CC", "CD", "DT", "EX", "IN", "JJ", "MD", "NN", "NNS", "PRP$", "VBZ", "WP$", ".", ",",
            "-LRB-", "HYPH", "X",
        ] {
            assert_eq!(Pos::parse(tag).map(|p| p.as_str()), Some(tag));
        }
    }
}
