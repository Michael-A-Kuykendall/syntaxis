use syntaxis::conllu::{export, import_str, ConlluError};
use syntaxis::parser_core::ids::{RulePackId, TokenId};
use syntaxis::parser_core::model::{ArcStatus, Relation};

const FIXTURE: &str = include_str!("../fixtures/challenge_agreement.conllu");
const FIXTURE_DIGEST: &str = "084f30bcfd3743f57b554316e4acd75575bdd8d4c4371234359a25e367f1e7ec";

fn pack() -> RulePackId {
    RulePackId::new("en-core@0.1.0")
}

#[test]
fn fixture_roundtrips_byte_for_byte() {
    let analysis = import_str(FIXTURE, &pack()).unwrap();
    assert_eq!(export(&analysis), FIXTURE);
}

#[test]
fn imported_fixture_validates() {
    let analysis = import_str(FIXTURE, &pack()).unwrap();
    assert!(analysis.validate().is_empty());
    assert_eq!(analysis.sentences.len(), 3);
    assert_eq!(analysis.tokens.len(), 18);
    assert_eq!(analysis.arcs.len(), 18);
}

#[test]
fn imported_fixture_preserves_supported_and_unknown_details() {
    let analysis = import_str(FIXTURE, &pack()).unwrap();
    let there = analysis
        .arcs
        .values()
        .find(|a| a.dependent == TokenId(5))
        .unwrap();
    assert_eq!(there.relation, Relation::Expl);
    assert_eq!(there.status, ArcStatus::Accepted);
    let many = analysis
        .token_analyses
        .values()
        .find(|a| analysis.surface_of(a.token) == Some("many"))
        .unwrap();
    assert_eq!(many.unmapped_features, vec!["Degree=Pos"]);
}

#[test]
fn text_mismatch_and_malformed_rows_are_rejected() {
    let bad = "# text = mismatch\n1\tHi\thi\tINTJ\tUH\t_\t0\troot\t_\t_\n";
    assert!(matches!(
        import_str(bad, &pack()),
        Err(ConlluError::TextMismatch { .. })
    ));
    let short = "# sent_id = 1\n1\tHi\thi\tINTJ\n";
    assert!(matches!(
        import_str(short, &pack()),
        Err(ConlluError::WrongColumnCount { line: 2, found: 4 })
    ));
}

#[test]
fn unsupported_legacy_relation_is_explicit() {
    let input = "1\tEach\teach\tPRON\tDT\tNumber=Sing\t2\tnsubj\t_\t_\n2\thas\thave\tVERB\tVBZ\tNumber=Sing\t0\troot\t_\t_\n3\tof\tof\tADP\tIN\t_\t1\tprep\t_\t_\n";
    let graph = import_str(input, &pack()).unwrap();
    let prep = graph
        .arcs
        .values()
        .find(|arc| arc.raw_label.as_deref() == Some("prep"))
        .unwrap();
    assert_eq!(prep.relation, Relation::Unsupported);
    assert_eq!(prep.status, ArcStatus::Unsupported);
}

#[test]
fn import_is_deterministic_and_export_is_idempotent() {
    let a = import_str(FIXTURE, &pack()).unwrap();
    let b = import_str(FIXTURE, &pack()).unwrap();
    assert_eq!(a.digest(), b.digest());
    let once = export(&a);
    assert_eq!(export(&import_str(&once, &pack()).unwrap()), once);
}

#[test]
fn fixture_digest_matches_recorded_value() {
    let analysis = import_str(FIXTURE, &pack()).unwrap();
    assert_eq!(analysis.digest(), FIXTURE_DIGEST);
}
