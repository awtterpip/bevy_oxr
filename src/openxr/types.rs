use std::borrow::Cow;

use super::graphics::{graphics_match, GraphicsExt, GraphicsWrap};

pub use super::extensions::XrExtensions;
pub use openxr::{
    EnvironmentBlendMode, Extent2Di, FormFactor, Graphics, Offset2Di, Rect2Di,
    SwapchainCreateFlags, SwapchainUsageFlags,
};
pub type Result<T> = std::result::Result<T, XrError>;

pub struct WgpuGraphics(
    pub wgpu::Device,
    pub wgpu::Queue,
    pub wgpu::AdapterInfo,
    pub wgpu::Adapter,
    pub wgpu::Instance,
);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Version(pub u8, pub u8, pub u16);

impl Version {
    pub const BEVY: Self = Self(0, 12, 1);

    pub const fn to_u32(self) -> u32 {
        let major = (self.0 as u32) << 24;
        let minor = (self.1 as u32) << 16;
        self.2 as u32 | major | minor
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppInfo {
    pub name: Cow<'static, str>,
    pub version: Version,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: "Bevy".into(),
            version: Version::BEVY,
        }
    }
}

pub type GraphicsBackend = GraphicsWrap<()>;

impl GraphicsBackend {
    const ALL: &'static [Self] = &[Self::Vulkan(())];

    pub fn available_backends(exts: &XrExtensions) -> Vec<Self> {
        Self::ALL
            .iter()
            .copied()
            .filter(|backend| backend.is_available(exts))
            .collect()
    }

    pub fn is_available(&self, exts: &XrExtensions) -> bool {
        self.required_exts().is_available(exts)
    }

    pub fn required_exts(&self) -> XrExtensions {
        graphics_match!(
            self;
            _ => Api::required_exts()
        )
    }
}

mod error {
    use super::GraphicsBackend;
    use std::borrow::Cow;
    use std::fmt;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum XrError {
        #[error("OpenXR error: {0}")]
        OpenXrError(#[from] openxr::sys::Result),
        #[error("OpenXR loading error: {0}")]
        OpenXrLoadingError(#[from] openxr::LoadError),
        #[error("WGPU instance error: {0}")]
        WgpuInstanceError(#[from] wgpu_hal::InstanceError),
        #[error("WGPU device error: {0}")]
        WgpuDeviceError(#[from] wgpu_hal::DeviceError),
        #[error("WGPU request device error: {0}")]
        WgpuRequestDeviceError(#[from] wgpu::RequestDeviceError),
        #[error("Unsupported texture format: {0:?}")]
        UnsupportedTextureFormat(wgpu::TextureFormat),
        #[error("Vulkan error: {0}")]
        VulkanError(#[from] ash::vk::Result),
        #[error("Vulkan loading error: {0}")]
        VulkanLoadingError(#[from] ash::LoadingError),
        #[error("Graphics backend '{0:?}' is not available")]
        UnavailableBackend(GraphicsBackend),
        #[error("No compatible backend available")]
        NoAvailableBackend,
        #[error("No compatible view configuration available")]
        NoAvailableViewConfiguration,
        #[error("No compatible blend mode available")]
        NoAvailableBlendMode,
        #[error("No compatible format available")]
        NoAvailableFormat,
        #[error("OpenXR runtime does not support these extensions: {0}")]
        UnavailableExtensions(UnavailableExts),
        #[error("Could not meet graphics requirements for platform. See console for details")]
        FailedGraphicsRequirements,
        #[error(
            "Tried to use item {item} with backend {backend}. Expected backend {expected_backend}"
        )]
        GraphicsBackendMismatch {
            item: &'static str,
            backend: &'static str,
            expected_backend: &'static str,
        },
        #[error("Failed to create CString: {0}")]
        NulError(#[from] std::ffi::NulError),
    }

    impl From<Vec<Cow<'static, str>>> for XrError {
        fn from(value: Vec<Cow<'static, str>>) -> Self {
            Self::UnavailableExtensions(UnavailableExts(value))
        }
    }

    #[derive(Debug)]
    pub struct UnavailableExts(Vec<Cow<'static, str>>);

    impl fmt::Display for UnavailableExts {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for s in &self.0 {
                write!(f, "\t{s}")?;
            }
            Ok(())
        }
    }
}

pub use error::XrError;

#[derive(Debug, Copy, Clone)]
pub struct SwapchainCreateInfo {
    pub create_flags: SwapchainCreateFlags,
    pub usage_flags: SwapchainUsageFlags,
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub width: u32,
    pub height: u32,
    pub face_count: u32,
    pub array_size: u32,
    pub mip_count: u32,
}

impl<G: GraphicsExt> TryFrom<SwapchainCreateInfo> for openxr::SwapchainCreateInfo<G> {
    type Error = XrError;

    fn try_from(value: SwapchainCreateInfo) -> Result<Self> {
        Ok(openxr::SwapchainCreateInfo {
            create_flags: value.create_flags,
            usage_flags: value.usage_flags,
            format: G::from_wgpu_format(value.format)
                .ok_or(XrError::UnsupportedTextureFormat(value.format))?,
            sample_count: value.sample_count,
            width: value.width,
            height: value.height,
            face_count: value.face_count,
            array_size: value.array_size,
            mip_count: value.mip_count,
        })
    }
}

pub use builder::*;

/// Copied with modification from the openxr crate to allow for a safe, graphics agnostic api to work with Bevy.
mod builder {
    use std::mem;

    use openxr::{sys, CompositionLayerFlags, Fovf, Posef, Rect2Di, Space};

    use crate::openxr::{graphics::graphics_match, XrSwapchain};

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
}
