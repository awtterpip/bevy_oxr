use std::sync::{Arc, Mutex};

use ash::vk::Handle;
use bevy::prelude::*;

use openxr::{AnyGraphics, Vulkan};

use crate::openxr::init::Version;

use super::extensions::XrExtensions;
use super::init::{self, AppInfo, GraphicsBackend, GraphicsExt, XrInitError};
type Result<T> = std::result::Result<T, XrInitError>;
use super::types::*;

#[derive(Deref, Clone)]
pub struct XrEntry(openxr::Entry);

impl XrEntry {
    pub fn enumerate_extensions(&self) -> Result<XrExtensions> {
        Ok(self.0.enumerate_extensions().map(Into::into)?)
    }

    pub fn create_instance(
        entry: XrEntry,
        app_info: AppInfo,
        exts: XrExtensions,
        backend: GraphicsBackend,
    ) -> Result<XrInstance> {
        let available_exts = entry.enumerate_extensions()?;

        if !backend.is_available(&available_exts) {
            return Err(XrInitError::UnavailableBackend(backend));
        }

        let required_exts = exts | backend.required_exts();

        let instance = entry.create_instance(
            &openxr::ApplicationInfo {
                application_name: app_info.name,
                application_version: app_info.version.to_u32(),
                engine_name: "Bevy",
                engine_version: Version::BEVY.to_u32(),
            },
            &required_exts.into(),
            &[],
        )?;

        Ok(XrInstance(instance, backend))
    }

    pub fn available_backends(&self) -> Result<Vec<GraphicsBackend>> {
        Ok(GraphicsBackend::available_backends(
            &self.enumerate_extensions()?,
        ))
    }
}

impl From<openxr::Entry> for XrEntry {
    fn from(value: openxr::Entry) -> Self {
        Self(value)
    }
}

#[derive(Resource, Deref, Clone)]
pub struct XrInstance(
    #[deref] pub(crate) openxr::Instance,
    pub(crate) GraphicsBackend,
);

impl XrInstance {
    pub fn create_session(
        &self,
        app_info: AppInfo,
        system_id: openxr::SystemId,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<(
        wgpu::Device,
        wgpu::Queue,
        wgpu::Adapter,
        wgpu::Instance,
        openxr::Session<openxr::AnyGraphics>,
        openxr::FrameWaiter,
        FrameStreamInner,
    )> {
        match self.1 {
            GraphicsBackend::Vulkan => {
                openxr::Vulkan::create_session(app_info, &self.0, system_id, format, resolution)
            }
        }
    }
}

#[derive(Resource, Deref, Clone)]
pub struct XrSession(
    #[deref] pub(crate) openxr::Session<AnyGraphics>,
    pub(crate) TypedSession,
);

impl XrSession {
    pub fn enumerate_swapchain_formats(&self) -> Result<Vec<wgpu::TextureFormat>> {
        self.1.enumerate_swapchain_formats()
    }

    pub fn create_swapchain(&self, info: SwapchainCreateInfo) -> Result<XrSwapchain> {
        self.1.create_swapchain(info)
    }
}

#[derive(Clone)]
pub enum TypedSession {
    Vulkan(openxr::Session<Vulkan>),
}

impl TypedSession {
    pub fn into_any_graphics(&self) -> openxr::Session<AnyGraphics> {
        match self {
            TypedSession::Vulkan(session) => session.clone().into_any_graphics(),
        }
    }

    pub fn enumerate_swapchain_formats(&self) -> Result<Vec<wgpu::TextureFormat>> {
        Ok(match self {
            TypedSession::Vulkan(session) => init::vulkan::enumerate_swapchain_formats(session),
        }?)
    }

    pub fn create_swapchain(&self, info: SwapchainCreateInfo) -> Result<XrSwapchain> {
        Ok(match self {
            TypedSession::Vulkan(session) => {
                XrSwapchain::Vulkan(session.create_swapchain(&info.try_into()?)?)
            }
        })
    }
}

#[derive(Resource, Default, Deref)]
pub struct Framebuffers(pub Vec<wgpu::Texture>);

#[derive(Clone)]
pub struct Swapchain {
    pub inner: Arc<Mutex<XrSwapchain>>,
    pub format: wgpu::TextureFormat,
    pub resolution: UVec2,
}

pub enum FrameStreamInner {
    Vulkan(openxr::FrameStream<openxr::Vulkan>),
}

pub enum XrSwapchain {
    Vulkan(openxr::Swapchain<openxr::Vulkan>),
}

impl XrSwapchain {
    pub fn acquire_image(&mut self) -> Result<u32> {
        Ok(match self {
            XrSwapchain::Vulkan(swap) => swap.acquire_image()?,
        })
    }

    pub fn wait_image(&mut self, timeout: openxr::Duration) -> Result<()> {
        Ok(match self {
            XrSwapchain::Vulkan(swap) => swap.wait_image(timeout)?,
        })
    }

    pub fn release_image(&mut self) -> Result<()> {
        Ok(match self {
            XrSwapchain::Vulkan(swap) => swap.release_image()?,
        })
    }

    pub fn enumerate_images(
        &mut self,
        device: wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<Vec<wgpu::Texture>> {
        match self {
            XrSwapchain::Vulkan(swap) => swap.enumerate_imgs(device, format, resolution),
        }
    }
}

trait EnumerateImages {
    fn enumerate_imgs(
        &mut self,
        device: wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<Vec<wgpu::Texture>>;
}

impl EnumerateImages for openxr::Swapchain<openxr::Vulkan> {
    fn enumerate_imgs(
        &mut self,
        device: wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<Vec<wgpu::Texture>> {
        let images = self.enumerate_images()?;
        let images = images.into_iter().map(|color_image| {
            let color_image = ash::vk::Image::from_raw(color_image);
            let wgpu_hal_texture = unsafe {
                <wgpu_hal::vulkan::Api as wgpu_hal::Api>::Device::texture_from_raw(
                    color_image,
                    &wgpu_hal::TextureDescriptor {
                        label: Some("VR Swapchain"),
                        size: wgpu::Extent3d {
                            width: resolution.x,
                            height: resolution.y,
                            depth_or_array_layers: 2,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: format,
                        usage: wgpu_hal::TextureUses::COLOR_TARGET
                            | wgpu_hal::TextureUses::COPY_DST,
                        memory_flags: wgpu_hal::MemoryFlags::empty(),
                        view_formats: vec![],
                    },
                    None,
                )
            };
            let texture = unsafe {
                device.create_texture_from_hal::<wgpu_hal::vulkan::Api>(
                    wgpu_hal_texture,
                    &wgpu::TextureDescriptor {
                        label: Some("VR Swapchain"),
                        size: wgpu::Extent3d {
                            width: resolution.x,
                            height: resolution.y,
                            depth_or_array_layers: 2,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    },
                )
            };
            texture
        });
        Ok(images.collect())
    }
}
