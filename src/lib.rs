use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

mod fragment;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}


#[derive(Serialize, Deserialize, Clone, Debug)]
struct CombinationRequest {
    max_fragments: u8,
    frag_weights: HashMap<String, i64>,
    set_effect_weights: Vec<(String, u8, i64)>,
    max_results: usize,
}

pub fn compute_combinations_impl(request: &str) -> anyhow::Result<String> {
    let decoded_request: CombinationRequest = serde_json::from_str(request)?;
    let result = fragment::best_combinations(
        decoded_request.max_fragments,
        decoded_request.frag_weights,
        decoded_request.set_effect_weights,
        decoded_request.max_results,
    )?;
    Ok(serde_json::to_string(&result)?)
}

#[wasm_bindgen]
pub fn compute_combinations(request: &str) -> String {
    match compute_combinations_impl(request) {
        Ok(result) => format!(r#"{{"status": "ok", "result": {}}}"#, result),
        Err(e) => format!(r#"{{"status": "err", "result": {}}}"#, e.to_string()).into(),
    }
}
