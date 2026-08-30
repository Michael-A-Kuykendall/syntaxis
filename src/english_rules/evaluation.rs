//! Frozen structural evaluation gate.
//!
//! The first gate is an original, generated corpus. It is Apache-2.0 licensed
//! by this repository and measures the declared constructions, not general
//! English quality. A natural corpus must not be substituted without its own
//! provenance and annotation review.

use crate::pipeline::analyze;
use crate::rulepack::RulePack;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    Clean,
    AgreementError,
}

pub struct EvaluationCase {
    pub text: String,
    pub label: Label,
    pub expected_kinds: Vec<crate::DiagnosticKind>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvaluationReport {
    pub total: usize,
    pub clean: usize,
    pub errors: usize,
    pub true_positive: usize,
    pub false_positive: usize,
    pub false_negative: usize,
    pub deterministic: bool,
}

impl EvaluationReport {
    pub fn precision_micros(&self) -> u64 {
        let denominator = self.true_positive + self.false_positive;
        if denominator == 0 {
            1_000_000
        } else {
            self.true_positive as u64 * 1_000_000 / denominator as u64
        }
    }

    pub fn recall_micros(&self) -> u64 {
        let denominator = self.true_positive + self.false_negative;
        if denominator == 0 {
            1_000_000
        } else {
            self.true_positive as u64 * 1_000_000 / denominator as u64
        }
    }
}

pub fn frozen_corpus() -> Vec<EvaluationCase> {
    let singular = [
        "cat", "dog", "book", "reason", "student", "example", "home", "case", "person", "teacher",
    ];
    let plural = [
        "cats", "dogs", "books", "reasons", "students", "examples", "homes", "cases", "people",
        "teachers",
    ];
    let mut cases = Vec::with_capacity(500);
    for index in 0..100 {
        let s = singular[index % singular.len()];
        let p = plural[index % plural.len()];
        cases.push(EvaluationCase {
            text: format!("The {s} is sleeping."),
            label: Label::Clean,
            expected_kinds: Vec::new(),
        });
        cases.push(EvaluationCase {
            text: format!("The {p} is sleeping."),
            label: Label::AgreementError,
            expected_kinds: Vec::new(),
        });
        cases.push(EvaluationCase {
            text: format!("There is many {p}."),
            label: Label::AgreementError,
            expected_kinds: Vec::new(),
        });
        cases.push(EvaluationCase {
            text: format!("There are many {p}."),
            label: Label::Clean,
            expected_kinds: Vec::new(),
        });
        cases.push(EvaluationCase {
            text: format!("Each of the {p} have a book."),
            label: Label::AgreementError,
            expected_kinds: Vec::new(),
        });
    }
    for case in &mut cases {
        case.expected_kinds = match case.label {
            Label::Clean => Vec::new(),
            Label::AgreementError => vec![crate::DiagnosticKind::Agreement],
        };
    }
    cases
}

pub fn evaluate(pack: &RulePack) -> EvaluationReport {
    let corpus = frozen_corpus();
    let mut true_positive = 0;
    let mut false_positive = 0;
    let mut false_negative = 0;
    for case in &corpus {
        let analysis = analyze(&case.text, pack);
        let mut actual_kinds: Vec<_> = analysis.diagnostics.values().map(|d| d.kind).collect();
        actual_kinds.sort();
        let mut expected_kinds = case.expected_kinds.clone();
        expected_kinds.sort();
        match (case.label, actual_kinds == expected_kinds) {
            (Label::AgreementError, true) => true_positive += 1,
            (Label::AgreementError, false) => false_negative += 1,
            (Label::Clean, true) => {}
            (Label::Clean, false) => false_positive += 1,
        }
    }
    let deterministic = corpus
        .iter()
        .all(|case| analyze(&case.text, pack).digest() == analyze(&case.text, pack).digest());
    EvaluationReport {
        total: corpus.len(),
        clean: corpus
            .iter()
            .filter(|case| case.label == Label::Clean)
            .count(),
        errors: corpus
            .iter()
            .filter(|case| case.label == Label::AgreementError)
            .count(),
        true_positive,
        false_positive,
        false_negative,
        deterministic,
    }
}

/// Small, checked-in adversarial corpus used by the release safety gate.
pub fn hostile_corpus() -> Vec<String> {
    vec![
        "a".repeat(64 * 1024),
        "(".repeat(4096) + &")".repeat(4096),
        "word... ... -- -- \"quoted\"".repeat(2048),
        "é漢字🙂".repeat(8192),
        "The\0cat\u{7f}are\u{2028}sleeping.".to_string(),
        "\n\n\n\t \n".repeat(4096),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conllu::import_str;

    #[test]
    fn frozen_gate_has_five_hundred_cases_and_is_repeatable() {
        let pack = RulePack::builtin().unwrap();
        let report = evaluate(&pack);
        assert_eq!(report.total, 500);
        assert_eq!(report.clean + report.errors, 500);
        assert!(report.deterministic);
        assert_eq!(report.false_positive, 0);
        assert_eq!(report.false_negative, 0);
        assert_eq!(report.precision_micros(), 1_000_000);
        assert_eq!(report.recall_micros(), 1_000_000);
    }

    #[test]
    fn hostile_inputs_are_bounded_and_never_panic() {
        let pack = RulePack::builtin().unwrap();
        for input in hostile_corpus() {
            assert!(input.len() <= 128 * 1024);
            let result = std::panic::catch_unwind(|| analyze(&input, &pack));
            assert!(
                result.is_ok(),
                "analysis panicked for {} bytes",
                input.len()
            );
        }

        let malformed = "1\tbroken\t\t\t\t\t\t\t\ttoo\tmany\tfields\n";
        let result = std::panic::catch_unwind(|| import_str(malformed, &pack.id));
        assert!(result.is_ok(), "CoNLL-U import panicked on malformed input");
    }
}
