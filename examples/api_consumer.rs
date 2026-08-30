use syntaxis::api::{
    analyze_conllu, analyze_json, digest, import_conllu_json, ApiError, MAX_INPUT_BYTES,
};

fn main() -> Result<(), ApiError> {
    let text = "The cat are sleeping.";

    let json = analyze_json(text)?;
    assert!(json.contains("GRAMMAR.AGREEMENT.SUBJECT_VERB"));
    let first_digest = digest(text)?;
    assert_eq!(first_digest, digest(text)?);
    println!("diagnostic JSON bytes: {}", json.len());
    println!("stable digest: {first_digest}");

    let conllu = analyze_conllu(text)?;
    assert!(conllu.contains("\troot\t"));
    println!("analyzed CoNLL-U rows: {}", conllu.lines().count());

    let fixture = include_str!("../fixtures/challenge_agreement.conllu");
    let imported = import_conllu_json(fixture)?;
    assert!(imported.contains("syntaxis/analysis"));
    println!("imported gold fixture JSON bytes: {}", imported.len());

    assert_eq!(analyze_json(""), Err(ApiError::EmptyInput));
    assert!(matches!(
        analyze_json(&"x".repeat(MAX_INPUT_BYTES + 1)),
        Err(ApiError::InputTooLarge { .. })
    ));
    assert!(analyze_json("Unicode café text.").is_ok());
    println!("boundary checks: empty, oversized, and Unicode inputs handled");

    Ok(())
}
