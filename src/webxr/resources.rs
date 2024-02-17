use bevy::prelude::{Deref, DerefMut};
use web_sys::WebGlFramebuffer;

#[derive(Deref, DerefMut, Clone)]
pub struct XrSession(#[deref] pub(crate) web_sys::XrSession);

#[derive(Deref, DerefMut, Clone)]
pub struct XrReferenceSpace(#[deref] pub(crate) web_sys::XrReferenceSpace);

#[derive(Deref, DerefMut, Clone)]
pub struct XrFrame(#[deref] pub(crate) web_sys::XrFrame);

#[derive(Deref, DerefMut, Clone)]
pub struct XrWebGlLayer(#[deref] pub(crate) web_sys::XrWebGlLayer);

impl XrWebGlLayer {
    pub(crate) fn framebuffer_unwrapped(&self) -> WebGlFramebuffer {
        js_sys::Reflect::get(&self, &"framebuffer".into())
            .unwrap()
            .into()
    }
}

#[derive(Clone)]
pub struct XrView(pub(crate) web_sys::XrView);
