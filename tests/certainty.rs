use syntaxis::english_rules::{pipeline::analyze, rulepack::RulePack};
use syntaxis::parser_core::model::{Certainty, DiagnosticKind, GrammarDiagnostic};
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
