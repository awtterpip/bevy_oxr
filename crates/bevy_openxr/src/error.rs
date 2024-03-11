use crate::graphics::GraphicsBackend;
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
