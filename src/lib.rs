#![feature(box_patterns)]
#![feature(let_chains)]
#![feature(assert_matches)]

use syntax::{abst, concrete, Exp};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
mod bootstrapping;
mod semantic;
mod syntax;

#[wasm_bindgen]
pub fn parse(code: &str) -> JsValue {
    let expr = concrete::parse(code).unwrap_or_else(|_| {
        abst::parse(code).unwrap()
    });
    serde_wasm_bindgen::to_value(&expr).unwrap()
}

#[wasm_bindgen]
pub fn format_abstract(exp: JsValue) -> String {
    let exp: Exp = serde_wasm_bindgen::from_value(exp).unwrap();
    abst::format(&exp)
}

#[wasm_bindgen]
pub fn format_concrete(exp: JsValue) -> String {
    let exp: Exp = serde_wasm_bindgen::from_value(exp).unwrap();
    concrete::format(&exp)
}

#[wasm_bindgen]
pub fn substitute(exp: JsValue, from_variable: &str, to_exp: JsValue) -> JsValue {
    let exp: Exp = serde_wasm_bindgen::from_value(exp).unwrap();
    let to_exp: Exp = serde_wasm_bindgen::from_value(to_exp).unwrap();
    let result = semantic::substitute(&exp, from_variable, &to_exp);
    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn eval(exp: JsValue) -> JsValue {
    let exp: Exp = serde_wasm_bindgen::from_value(exp).unwrap();
    let result = semantic::eval(&exp);
    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn standard_form(exp: JsValue) -> JsValue {
    let exp: Exp = serde_wasm_bindgen::from_value(exp).unwrap();
    let mut context = bootstrapping::Context::default();
    let result = bootstrapping::decompile(&exp, &mut context);
    serde_wasm_bindgen::to_value(&result).unwrap()
}
