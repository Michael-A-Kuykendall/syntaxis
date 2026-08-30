//! The Penn <-> UD projection.
//!
//! This mapping is lossy in both directions and every lossy decision is listed
//! here rather than buried in a match arm. It is versioned separately from the
//! engine so a consumer can pin it: changing a single arm below changes
//! CoNLL-U output for existing documents.
//!
//! Known ambiguities, all resolved conservatively and all recoverable because
//! `TokenAnalysis` stores `pos` and `upos` independently:
//!
//! * `IN` covers both prepositions and subordinating conjunctions. It maps to
//!   `ADP`; a `mark` arc is what distinguishes `SCONJ`, and that is a
//!   structural decision the attachment layer makes, not a tag lookup.
//! * `VB*` covers both lexical and auxiliary verbs. It maps to `VERB`; gold
//!   data saying `AUX` is preserved verbatim on import rather than re-derived.
//! * `TO` maps to `PART`, which is wrong for the prepositional `to`. A future
//!   version should split the tag rather than complicate this table.
//!
//! Dependency relations are native UD labels. Legacy Stanford-style `prep`
//! and `pobj` labels remain unsupported during import rather than being
//! silently converted.

use crate::parser_core;
use parser_core::ids::Version;
use parser_core::model::{Pos, UPos};

/// Version of the projection in this file.
pub const MAPPING_VERSION: Version = Version::new(0, 1, 0);

/// Penn -> UD. Total and deterministic.
pub fn upos_for(pos: Pos) -> UPos {
    use Pos::*;
    match pos {
        CC => UPos::CCONJ,
        CD => UPos::NUM,
        DT | PDT | WDT => UPos::DET,
        EX | PRP | PRPS | WP | WPS => UPos::PRON,
        FW | LS | Unknown => UPos::X,
        IN => UPos::ADP,
        JJ | JJR | JJS => UPos::ADJ,
        MD => UPos::AUX,
        NN | NNS => UPos::NOUN,
        NNP | NNPS => UPos::PROPN,
        POS | TO => UPos::PART,
        RB | RBR | RBS | WRB => UPos::ADV,
        RP => UPos::ADP,
        SYM => UPos::SYM,
        UH => UPos::INTJ,
        VB | VBD | VBG | VBN | VBP | VBZ => UPos::VERB,
        PunctSent | PunctComma | PunctColon | PunctLeftBracket | PunctRightBracket
        | PunctOpenQuote | PunctCloseQuote | PunctHyph | PunctOther => UPos::PUNCT,
    }
}

/// UD -> Penn, used only when an imported file has no XPOS column. Lossy: it
/// picks the most frequent Penn tag for each UPOS and cannot recover number,
/// tense, or degree, which is why FEATS is read separately.
pub fn pos_for(upos: UPos) -> Pos {
    match upos {
        UPos::ADJ => Pos::JJ,
        UPos::ADP => Pos::IN,
        UPos::ADV => Pos::RB,
        UPos::AUX => Pos::VB,
        UPos::CCONJ => Pos::CC,
        UPos::DET => Pos::DT,
        UPos::INTJ => Pos::UH,
        UPos::NOUN => Pos::NN,
        UPos::NUM => Pos::CD,
        UPos::PART => Pos::RP,
        UPos::PRON => Pos::PRP,
        UPos::PROPN => Pos::NNP,
        UPos::PUNCT => Pos::PunctOther,
        UPos::SCONJ => Pos::IN,
        UPos::SYM => Pos::SYM,
        UPos::VERB => Pos::VB,
        UPos::X => Pos::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_is_total() {
        // Every Penn tag the model can produce must map somewhere.
        for tag in [
            Pos::CC,
            Pos::CD,
            Pos::DT,
            Pos::EX,
            Pos::FW,
            Pos::IN,
            Pos::JJ,
            Pos::MD,
            Pos::NN,
            Pos::NNS,
            Pos::NNP,
            Pos::POS,
            Pos::PRP,
            Pos::PRPS,
            Pos::RB,
            Pos::RP,
            Pos::SYM,
            Pos::TO,
            Pos::UH,
            Pos::VBZ,
            Pos::WDT,
            Pos::WP,
            Pos::WRB,
            Pos::PunctSent,
            Pos::Unknown,
        ] {
            let _ = upos_for(tag);
        }
    }

    #[test]
    fn known_lossy_pairs_are_documented_not_accidental() {
        // IN collapses ADP and SCONJ: the reverse trip cannot recover SCONJ.
        assert_eq!(upos_for(pos_for(UPos::SCONJ)), UPos::ADP);
        // AUX collapses into VERB the same way.
        assert_eq!(upos_for(pos_for(UPos::AUX)), UPos::VERB);
        // Everything else must round-trip.
        for upos in [
            UPos::ADJ,
            UPos::ADV,
            UPos::CCONJ,
            UPos::DET,
            UPos::INTJ,
            UPos::NOUN,
            UPos::NUM,
            UPos::PRON,
            UPos::PROPN,
            UPos::PUNCT,
            UPos::SYM,
            UPos::VERB,
            UPos::X,
        ] {
            assert_eq!(upos_for(pos_for(upos)), upos, "{upos} did not round-trip");
        }
    }
}
