use std::borrow::Cow;
use std::fmt;

use super::graphics::GraphicsBackend;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OxrError {
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
    #[error("Graphics init error: {0}")]
    InitError(InitError),
}

pub use init_error::InitError;

/// This module is needed because thiserror does not allow conditional compilation within enums for some reason,
/// so graphics api specific errors are implemented here.
mod init_error {
    use super::OxrError;
    use std::fmt;

    #[derive(Debug)]
    pub enum InitError {
        #[cfg(feature = "vulkan")]
        VulkanError(ash::vk::Result),
        #[cfg(feature = "vulkan")]
        VulkanLoadingError(ash::LoadingError),
    }

    impl fmt::Display for InitError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                #[cfg(feature = "vulkan")]
                InitError::VulkanError(error) => write!(f, "Vulkan error: {}", error),
                #[cfg(feature = "vulkan")]
                InitError::VulkanLoadingError(error) => {
                    write!(f, "Vulkan loading error: {}", error)
                }
            }
        }
    }

    #[cfg(feature = "vulkan")]
    impl From<ash::vk::Result> for OxrError {
        fn from(value: ash::vk::Result) -> Self {
            Self::InitError(InitError::VulkanError(value))
        }
    }

    #[cfg(feature = "vulkan")]
    impl From<ash::LoadingError> for OxrError {
        fn from(value: ash::LoadingError) -> Self {
            Self::InitError(InitError::VulkanLoadingError(value))
        }
    }
}

impl From<Vec<Cow<'static, str>>> for OxrError {
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
