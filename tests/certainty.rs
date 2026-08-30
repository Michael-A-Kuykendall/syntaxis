use syntaxis::english_rules::{pipeline::analyze, rulepack::RulePack};
use syntaxis::parser_core::model::{ArcStatus, Certainty, DiagnosticKind, GrammarDiagnostic};
use syntaxis::parser_core::support::{DerivationKind, SourceRef, SupportSet};
use syntaxis::parser_core::{MessageKey, RuleId, RulePackId};

#[test]
fn diagnostic_certainty_is_clamped_to_support() {
    let pack = RulePack::builtin().unwrap();
    let mut analysis = analyze("The cat are sleeping.", &pack);
    let arc = analysis.arcs.values().next().unwrap().clone();
    let token = analysis.tokens.values().next().unwrap();
    let support = SupportSet::new(
        RuleId::new("test"),
        RulePackId::new("test@0.1.0"),
        DerivationKind::GrammarRule,
        vec![SourceRef::Arc(arc.id)],
    );
    analysis.add_diagnostic(GrammarDiagnostic {
        id: analysis.next_diagnostic_id(),
        sentence: token.sentence,
        kind: DiagnosticKind::Agreement,
        span: token.span,
        message_key: MessageKey::new("test.diagnostic"),
        certainty: Certainty::Definite,
        replacements: Vec::new(),
        support,
    });
    assert_eq!(
        analysis.diagnostics.values().next().unwrap().certainty,
        analysis.certainty_for(&[SourceRef::Arc(arc.id)])
    );
}

#[test]
fn unresolved_arcs_lower_certainty_but_token_support_is_definite() {
    let pack = RulePack::builtin().unwrap();
    let mut unsupported = analyze("The cat are sleeping.", &pack);
    let unsupported_id = unsupported.arcs.keys().next().copied().unwrap();
    unsupported.arcs.get_mut(&unsupported_id).unwrap().status = ArcStatus::Unsupported;
    assert_eq!(
        unsupported.certainty_for(&[SourceRef::Arc(unsupported_id)]),
        Certainty::Conditional
    );

    let mut alternative = analyze("The cat are sleeping.", &pack);
    let alternative_id = alternative.arcs.keys().next().copied().unwrap();
    alternative.arcs.get_mut(&alternative_id).unwrap().status = ArcStatus::Alternative {
        group: syntaxis::parser_core::AlternativeGroupId(99),
    };
    assert_eq!(
        alternative.certainty_for(&[SourceRef::Arc(alternative_id)]),
        Certainty::Conditional
    );

    assert_eq!(
        alternative.certainty_for(&[SourceRef::Token(syntaxis::parser_core::TokenId(0))]),
        Certainty::Definite
    );
}
