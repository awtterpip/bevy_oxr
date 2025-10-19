use std::mem;

use bevy_ecs::world::World;
use bevy_mod_xr::spaces::{XrPrimaryReferenceSpace, XrSpace};
use openxr::{sys, CompositionLayerFlags, Fovf, Posef, Rect2Di};

use crate::graphics::graphics_match;
use crate::resources::*;
use crate::spaces::OxrSpaceExt as _;

pub trait LayerProvider {
    fn get<'a>(&'a self, world: &'a World) -> Option<Box<dyn CompositionLayer<'a> + 'a>>;
}

pub struct ProjectionLayer;

pub struct PassthroughLayer;

impl LayerProvider for ProjectionLayer {
    fn get<'a>(&self, world: &'a World) -> Option<Box<dyn CompositionLayer<'a> + 'a>> {
        let stage = world.get_resource::<XrPrimaryReferenceSpace>()?;
        let openxr_views = world.get_resource::<OxrViews>()?;
        let swapchain = world.get_resource::<OxrSwapchain>()?;
        let graphics_info = world.get_resource::<OxrCurrentSessionConfig>()?;
        let rect = openxr::Rect2Di {
            offset: openxr::Offset2Di { x: 0, y: 0 },
            extent: openxr::Extent2Di {
                width: graphics_info.resolution.x as _,
                height: graphics_info.resolution.y as _,
            },
        };

        if openxr_views.len() < 2 {
            return None;
        }

        Some(Box::new(
            CompositionLayerProjection::new()
                .layer_flags(CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA)
                .space(stage)
                .views(&[
                    CompositionLayerProjectionView::new()
                        .pose(openxr_views.0[0].pose)
                        .fov(openxr_views.0[0].fov)
                        .sub_image(
                            SwapchainSubImage::new()
                                .swapchain(swapchain)
                                .image_array_index(0)
                                .image_rect(rect),
                        ),
                    CompositionLayerProjectionView::new()
                        .pose(openxr_views.0[1].pose)
                        .fov(openxr_views.0[1].fov)
                        .sub_image(
                            SwapchainSubImage::new()
                                .swapchain(swapchain)
                                .image_array_index(1)
                                .image_rect(rect),
                        ),
                ]),
        ))
    }
}

impl LayerProvider for PassthroughLayer {
    fn get(&self, world: &World) -> Option<Box<dyn CompositionLayer<'_>>> {
        Some(Box::new(
            CompositionLayerPassthrough::new()
                .layer_handle(world.get_resource::<OxrPassthroughLayer>()?)
                .layer_flags(CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA),
        ))
    }
}

#[derive(Copy, Clone)]
pub struct SwapchainSubImage<'a> {
    inner: sys::SwapchainSubImage,
    swapchain: Option<&'a OxrSwapchain>,
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
    pub fn swapchain(mut self, value: &'a OxrSwapchain) -> Self {
        graphics_match!(
            &value.0;
            swap => self.inner.swapchain = swap.as_raw()
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

impl Default for SwapchainSubImage<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone)]
pub struct CompositionLayerProjectionView<'a> {
    inner: sys::CompositionLayerProjectionView,
    swapchain: Option<&'a OxrSwapchain>,
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
impl Default for CompositionLayerProjectionView<'_> {
    fn default() -> Self {
        Self::new()
    }
}
/// # Safety
/// the header function must return a ref to a valid Composition Layer struct.
/// it has to use `repr(C)` and it has to follow the shape of a Composition Layer struct from the
/// OpenXR specification
pub unsafe trait CompositionLayer<'a> {
    fn swapchain(&self) -> Option<&'a OxrSwapchain>;
    fn header(&self) -> &sys::CompositionLayerBaseHeader;
}
#[derive(Clone)]
pub struct CompositionLayerProjection<'a> {
    inner: sys::CompositionLayerProjection,
    swapchain: Option<&'a OxrSwapchain>,
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
    pub fn space(mut self, value: &XrSpace) -> Self {
        self.inner.space = value.as_raw_openxr_space();
        self
    }
    #[inline]
    pub fn views(mut self, value: &[CompositionLayerProjectionView<'a>]) -> Self {
        self.views = value.iter().map(|view| view.inner).collect();
        self.inner.views = self.views.as_slice().as_ptr() as *const _ as _;
        self.inner.view_count = self.views.len() as u32;
        self
    }
}
unsafe impl<'a> CompositionLayer<'a> for CompositionLayerProjection<'a> {
    fn swapchain(&self) -> Option<&'a OxrSwapchain> {
        self.swapchain
    }

    fn header(&self) -> &sys::CompositionLayerBaseHeader {
        unsafe { mem::transmute(&self.inner) }
    }
}
impl Default for CompositionLayerProjection<'_> {
    fn default() -> Self {
        Self::new()
    }
}
pub struct CompositionLayerPassthrough {
    inner: sys::CompositionLayerPassthroughFB,
}
impl Default for CompositionLayerPassthrough {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositionLayerPassthrough {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: openxr::sys::CompositionLayerPassthroughFB {
                ty: openxr::sys::CompositionLayerPassthroughFB::TYPE,
                ..unsafe { mem::zeroed() }
            },
        }
    }
    #[inline]
    pub fn layer_handle(mut self, layer_handle: &OxrPassthroughLayer) -> Self {
        self.inner.layer_handle = *layer_handle.inner();
        self
    }
    #[inline]
    pub fn layer_flags(mut self, value: CompositionLayerFlags) -> Self {
        self.inner.flags = value;
        self
    }
}
unsafe impl<'a> CompositionLayer<'a> for CompositionLayerPassthrough {
    fn swapchain(&self) -> Option<&'a OxrSwapchain> {
        None
    }

    fn header(&self) -> &sys::CompositionLayerBaseHeader {
        unsafe { mem::transmute(&self.inner) }
    }
}
