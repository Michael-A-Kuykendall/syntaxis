//! JavaScript bindings for the I/O-free consumer API.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn analyze_json(text: &str) -> Result<String, JsValue> {
    crate::api::analyze_json(text).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn digest(text: &str) -> Result<String, JsValue> {
    crate::api::digest(text).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn import_conllu_json(input: &str) -> Result<String, JsValue> {
    crate::api::import_conllu_json(input).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn browser_api_returns_canonical_json_and_digest() {
        let json = analyze_json("The cat are sleeping.").unwrap();
        assert!(json.contains("The cat are sleeping."));
        assert_eq!(digest("The cat are sleeping.").unwrap().len(), 64);
    }

    #[wasm_bindgen_test]
    fn browser_api_returns_errors_without_panicking() {
        assert!(analyze_json("").is_err());
        assert!(import_conllu_json("1\tbroken\n").is_err());
    }
}
