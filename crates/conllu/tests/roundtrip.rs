use conllu::{export, import_str, ConlluError};
use parser_core::ids::{RulePackId, TokenId};
use parser_core::model::{ArcStatus, Relation};

const FIXTURE: &str = include_str!("../../../fixtures/challenge_agreement.conllu");

fn pack() -> RulePackId {
    RulePackId::new("en-core@0.1.0")
}

#[test]
fn fixture_roundtrips_byte_for_byte() {
    let analysis = import_str(FIXTURE, &pack()).expect("fixture must import");
    assert_eq!(export(&analysis), FIXTURE);
}

#[test]
fn imported_fixture_validates() {
    let analysis = import_str(FIXTURE, &pack()).unwrap();
    let issues = analysis.validate();
    assert!(issues.is_empty(), "{issues:?}");
    assert_eq!(analysis.sentences.len(), 3);
    assert_eq!(analysis.tokens.len(), 5 + 5 + 8);
    assert_eq!(analysis.arcs.len(), 5 + 5 + 8);
}

#[test]
fn document_text_is_reconstructed_from_the_tokens() {
    let analysis = import_str(FIXTURE, &pack()).unwrap();
    assert_eq!(
        analysis.document.text,
        "The cat are sleeping.\nThere is many reasons.\nEach of the students have a book."
    );
    // Every span must slice back to its own surface.
    for token in analysis.tokens.values() {
        assert_eq!(
            token.span.slice(&analysis.document.text),
            Some(token.surface.as_str()),
            "{token:?}"
        );
    }
}

/// The frozen relation set has no `expl`. The importer must say so rather than
/// bending it into `nsubj`.
#[test]
fn out_of_set_relations_are_marked_unsupported_not_guessed() {
    let analysis = import_str(FIXTURE, &pack()).unwrap();
    let there = analysis
        .arcs
        .values()
        .find(|a| a.dependent == TokenId(5))
        .expect("the existential `There` must have an arc");
    assert_eq!(there.relation, Relation::Unsupported);
    assert_eq!(there.raw_label.as_deref(), Some("expl"));
    assert_eq!(there.status, ArcStatus::Unsupported);
}

#[test]
fn unrepresented_features_survive_the_round_trip() {
    let analysis = import_str(FIXTURE, &pack()).unwrap();
    let many = analysis
        .token_analyses
        .values()
        .find(|a| analysis.surface_of(a.token) == Some("many"))
        .unwrap();
    assert_eq!(many.unmapped_features, vec!["Degree=Pos".to_string()]);
}

#[test]
fn export_is_idempotent() {
    let once = export(&import_str(FIXTURE, &pack()).unwrap());
    let twice = export(&import_str(&once, &pack()).unwrap());
    assert_eq!(once, twice);
}

#[test]
fn declared_text_must_match_the_tokens() {
    let bad = "# text = Not what the tokens say\n1\tHi\thi\tINTJ\tUH\t_\t0\troot\t_\t_\n";
    assert!(matches!(
        import_str(bad, &pack()),
        Err(ConlluError::TextMismatch { .. })
    ));
}

#[test]
fn multiword_tokens_and_empty_nodes_are_rejected() {
    let multiword = "1-2\tdon't\t_\t_\t_\t_\t_\t_\t_\t_\n";
    assert!(matches!(
        import_str(multiword, &pack()),
        Err(ConlluError::MultiwordToken { .. })
    ));
    let empty_node = "1.1\tsomething\t_\t_\t_\t_\t_\t_\t_\t_\n";
    assert!(matches!(
        import_str(empty_node, &pack()),
        Err(ConlluError::EmptyNode { .. })
    ));
}

#[test]
fn malformed_rows_report_their_line() {
    let short = "# sent_id = 1\n1\tHi\thi\tINTJ\n";
    match import_str(short, &pack()) {
        Err(ConlluError::WrongColumnCount { line, found }) => {
            assert_eq!((line, found), (2, 4));
        }
        other => panic!("expected a column count error, got {other:?}"),
    }
}

#[test]
fn ids_must_be_consecutive() {
    let gap =
        "1\tHi\thi\tINTJ\tUH\t_\t0\troot\t_\t_\n3\tthere\tthere\tADV\tRB\t_\t1\tadvmod\t_\t_\n";
    assert!(matches!(
        import_str(gap, &pack()),
        Err(ConlluError::NonConsecutiveId { .. })
    ));
}

#[test]
fn two_accepted_roots_are_rejected() {
    let two = "1\tHi\thi\tINTJ\tUH\t_\t0\troot\t_\t_\n2\tthere\tthere\tADV\tRB\t_\t0\troot\t_\t_\n";
    assert!(matches!(
        import_str(two, &pack()),
        Err(ConlluError::MultipleRoots { .. })
    ));
}

#[test]
fn import_is_deterministic() {
    let a = import_str(FIXTURE, &pack()).unwrap();
    let b = import_str(FIXTURE, &pack()).unwrap();
    assert_eq!(a.digest(), b.digest());
    assert_eq!(a.to_canonical_json(), b.to_canonical_json());
}
