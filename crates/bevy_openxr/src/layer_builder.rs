use std::mem;

use openxr::{sys, CompositionLayerFlags, Fovf, Posef, Rect2Di, Space};

use crate::graphics::graphics_match;
use crate::resources::XrSwapchain;

#[derive(Copy, Clone)]
pub struct SwapchainSubImage<'a> {
    inner: sys::SwapchainSubImage,
    swapchain: Option<&'a XrSwapchain>,
}

impl<'a> SwapchainSubImage<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: sys::SwapchainSubImage {
                ..unsafe { mem::zeroed() }
            },
            swapchain: None,
        }
    }
    #[inline]
    pub fn into_raw(self) -> sys::SwapchainSubImage {
        self.inner
    }
    #[inline]
    pub fn as_raw(&self) -> &sys::SwapchainSubImage {
        &self.inner
    }
    #[inline]
    pub fn swapchain(mut self, value: &'a XrSwapchain) -> Self {
        graphics_match!(
            &value.0;
            swap => self.inner.swapchain = swap.lock().unwrap().as_raw()
        );
        self.swapchain = Some(value);
        self
    }
    #[inline]
    pub fn image_rect(mut self, value: Rect2Di) -> Self {
        self.inner.image_rect = value;
        self
    }
    #[inline]
    pub fn image_array_index(mut self, value: u32) -> Self {
        self.inner.image_array_index = value;
        self
    }
}

impl<'a> Default for SwapchainSubImage<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone)]
pub struct CompositionLayerProjectionView<'a> {
    inner: sys::CompositionLayerProjectionView,
    swapchain: Option<&'a XrSwapchain>,
}

impl<'a> CompositionLayerProjectionView<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: sys::CompositionLayerProjectionView {
                ty: sys::StructureType::COMPOSITION_LAYER_PROJECTION_VIEW,
                ..unsafe { mem::zeroed() }
            },
            swapchain: None,
        }
    }
    #[inline]
    pub fn into_raw(self) -> sys::CompositionLayerProjectionView {
        self.inner
    }
    #[inline]
    pub fn as_raw(&self) -> &sys::CompositionLayerProjectionView {
        &self.inner
    }
    #[inline]
    pub fn pose(mut self, value: Posef) -> Self {
        self.inner.pose = value;
        self
    }
    #[inline]
    pub fn fov(mut self, value: Fovf) -> Self {
        self.inner.fov = value;
        self
    }
    #[inline]
    pub fn sub_image(mut self, value: SwapchainSubImage<'a>) -> Self {
        self.inner.sub_image = value.inner;
        self.swapchain = value.swapchain;
        self
    }
}
impl<'a> Default for CompositionLayerProjectionView<'a> {
    fn default() -> Self {
        Self::new()
    }
}
pub unsafe trait CompositionLayer<'a> {
    fn swapchain(&self) -> Option<&'a XrSwapchain>;
    fn header(&self) -> &'a sys::CompositionLayerBaseHeader;
}
#[derive(Clone)]
pub struct CompositionLayerProjection<'a> {
    inner: sys::CompositionLayerProjection,
    swapchain: Option<&'a XrSwapchain>,
    views: Vec<sys::CompositionLayerProjectionView>,
}
impl<'a> CompositionLayerProjection<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: sys::CompositionLayerProjection {
                ty: sys::StructureType::COMPOSITION_LAYER_PROJECTION,
                ..unsafe { mem::zeroed() }
            },
            swapchain: None,
            views: Vec::new(),
        }
    }
    #[inline]
    pub fn into_raw(self) -> sys::CompositionLayerProjection {
        self.inner
    }
    #[inline]
    pub fn as_raw(&self) -> &sys::CompositionLayerProjection {
        &self.inner
    }
    #[inline]
    pub fn layer_flags(mut self, value: CompositionLayerFlags) -> Self {
        self.inner.layer_flags = value;
        self
    }
    #[inline]
    pub fn space(mut self, value: &'a Space) -> Self {
        self.inner.space = value.as_raw();
        self
    }
    #[inline]
    pub fn views(mut self, value: &'a [CompositionLayerProjectionView<'a>]) -> Self {
        for view in value {
            self.views.push(view.inner.clone());
        }
        self.inner.views = self.views.as_slice().as_ptr() as *const _ as _;
        self.inner.view_count = value.len() as u32;
        self
    }
}
unsafe impl<'a> CompositionLayer<'a> for CompositionLayerProjection<'a> {
    fn swapchain(&self) -> Option<&'a XrSwapchain> {
        self.swapchain
    }

    fn header(&self) -> &'a sys::CompositionLayerBaseHeader {
        unsafe { std::mem::transmute(&self.inner) }
    }
}
impl<'a> Default for CompositionLayerProjection<'a> {
    fn default() -> Self {
        Self::new()
    }
}
