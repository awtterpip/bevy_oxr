use wasm_bindgen::JsValue;
use wasm_bindgen_futures::js_sys::Promise;
use wasm_bindgen_futures::JsFuture;

pub trait PromiseRes {
    fn resolve<T: From<JsValue>>(self) -> Result<T, JsValue>;
}

impl PromiseRes for Promise {
    fn resolve<T: From<JsValue>>(self) -> Result<T, JsValue> {
        resolve_promise(self)
    }
}

pub fn resolve_promise<T: From<JsValue>>(promise: Promise) -> Result<T, JsValue> {
    futures::executor::block_on(async move { JsFuture::from(promise).await.map(Into::into) })
}
