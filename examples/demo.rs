use syntaxis::conllu::{export, import_str};
use syntaxis::english_rules::{pipeline::analyze, rulepack::RulePack};
use syntaxis::parser_core::support::{FactId, SourceRef};

fn main() {
    let pack = RulePack::builtin().unwrap();

    println!("== tokenization with spans ==");
    let a = analyze(
        "Dr. Smith didn't think each of the students have a book.",
        &pack,
    );
    for t in a.tokens.values() {
        println!(
            "  {:>3} {:<10} {:<10} {}",
            t.id.to_string(),
            t.surface,
            t.span.to_string(),
            t.support.rule
        );
    }
    println!("  validation issues: {:?}", a.validate());
    println!("  digest: {}", &a.digest()[..16]);

    println!("\n== gold fixture import ==");
    let fixture = include_str!("../fixtures/challenge_agreement.conllu");
    let mut g = import_str(fixture, &pack.id).unwrap();
    println!(
        "  sentences {} tokens {} arcs {}",
        g.sentences.len(),
        g.tokens.len(),
        g.arcs.len()
    );
    println!("  byte-identical re-export: {}", export(&g) == fixture);

    println!("\n== retraction cascade ==");
    let arc = *g.arcs.keys().next().unwrap();
    let before = (g.arcs.len(), g.token_analyses.len());
    let r = g.retract(&SourceRef::TokenAnalysis(
        syntaxis::parser_core::ids::TokenId(1),
    ));
    println!(
        "  retracted analysis of token t1 ({:?})",
        g.surface_of(syntaxis::parser_core::ids::TokenId(1))
    );
    println!(
        "  removed: {:?}",
        r.removed.iter().map(|f| f.to_string()).collect::<Vec<_>>()
    );
    println!(
        "  arcs {} -> {}, analyses {} -> {}",
        before.0,
        g.arcs.len(),
        before.1,
        g.token_analyses.len()
    );
    println!(
        "  first arc still present: {}",
        g.graph.contains(FactId::Arc(arc))
    );
}
