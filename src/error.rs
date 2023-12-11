pub struct XrError {}

#[cfg(target_family = "wasm")]
impl From<wasm_bindgen::JsValue> for XrError {
    fn from(_: wasm_bindgen::JsValue) -> Self {
        Self {}
    }
}

#[cfg(not(target_family = "wasm"))]
impl From<openxr::sys::Result> for XrError {
    fn from(_: openxr::sys::Result) -> Self {
        Self {}
    }
}
