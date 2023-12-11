use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{Object, Promise, Reflect},
    HtmlCanvasElement, WebGl2RenderingContext,
};

pub fn get_canvas(canvas_id: &str) -> Result<HtmlCanvasElement, JsValue> {
    let query = format!("#{}", canvas_id);
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .query_selector(&query)
        .unwrap()
        .expect("bevy_webxr - could not find canvas");
    let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;
    Ok(canvas)
}

pub fn create_webgl_context(
    xr_mode: bool,
    canvas: &str,
) -> Result<WebGl2RenderingContext, JsValue> {
    let canvas = get_canvas(canvas)?;

    let gl: WebGl2RenderingContext = if xr_mode {
        let gl_attribs = Object::new();
        Reflect::set(
            &gl_attribs,
            &JsValue::from_str("xrCompatible"),
            &JsValue::TRUE,
        )?;
        canvas
            .get_context_with_context_options("webgl2", &gl_attribs)?
            .unwrap()
            .dyn_into()?
    } else {
        canvas.get_context("webgl2")?.unwrap().dyn_into()?
    };

    Ok(gl)
}

pub trait PromiseRes {
    fn resolve<T: From<JsValue>>(self) -> Result<T, JsValue>;
}

impl PromiseRes for Promise {
    fn resolve<T: From<JsValue>>(self) -> Result<T, JsValue> {
        resolve_promise(self)
    }
}

pub fn resolve_promise<T: From<JsValue>>(promise: Promise) -> Result<T, JsValue> {
    bevy::tasks::block_on(async move { JsFuture::from(promise).await.map(Into::into) })
}
