use bevy::prelude::*;

use openxr::AnyGraphics;

use super::graphics::{graphics_match, GraphicsExt, GraphicsType, GraphicsWrap};
use super::types::*;

#[derive(Deref, Clone)]
pub struct XrEntry(openxr::Entry);

impl XrEntry {
    pub fn enumerate_extensions(&self) -> Result<XrExtensions> {
        Ok(self.0.enumerate_extensions().map(Into::into)?)
    }

    pub fn create_instance(
        &self,
        app_info: AppInfo,
        exts: XrExtensions,
        backend: GraphicsBackend,
    ) -> Result<XrInstance> {
        let available_exts = self.enumerate_extensions()?;

        if !backend.is_available(&available_exts) {
            return Err(XrError::UnavailableBackend(backend));
        }

        let required_exts = exts | backend.required_exts();

        let instance = self.0.create_instance(
            &openxr::ApplicationInfo {
                application_name: &app_info.name,
                application_version: app_info.version.to_u32(),
                engine_name: "Bevy",
                engine_version: Version::BEVY.to_u32(),
            },
            &required_exts.into(),
            &[],
        )?;

        Ok(XrInstance(instance, backend, app_info))
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
    pub(crate) AppInfo,
);

impl XrInstance {
    pub fn create_session(
        &self,
        system_id: openxr::SystemId,
    ) -> Result<(
        wgpu::Device,
        wgpu::Queue,
        wgpu::Adapter,
        wgpu::Instance,
        XrSession,
        XrFrameWaiter,
        XrFrameStream,
    )> {
        graphics_match!(
            self.1;
            _ => Api::create_session(&self.2, &self.0, system_id)
        )
    }
}

impl GraphicsType for XrSession {
    type Inner<G: GraphicsExt> = openxr::Session<G>;
}

#[derive(Resource, Deref, Clone)]
pub struct XrSession(
    #[deref] pub(crate) openxr::Session<AnyGraphics>,
    pub(crate) GraphicsWrap<Self>,
);

impl XrSession {
    pub fn enumerate_swapchain_formats(&self) -> Result<Vec<wgpu::TextureFormat>> {
        graphics_match!(
            &self.1;
            session => Ok(session.enumerate_swapchain_formats()?.into_iter().filter_map(Api::to_wgpu_format).collect())
        )
    }

    pub fn create_swapchain(&self, info: SwapchainCreateInfo) -> Result<XrSwapchain> {
        Ok(XrSwapchain(graphics_match!(
            &self.1;
            session => session.create_swapchain(&info.try_into()?)? => XrSwapchain
        )))
    }
}

pub struct XrFrameStream(pub(crate) GraphicsWrap<Self>);

impl GraphicsType for XrFrameStream {
    type Inner<G: GraphicsExt> = openxr::FrameStream<G>;
}

impl XrFrameStream {
    pub fn begin(&mut self) -> openxr::Result<()> {
        graphics_match!(
            &mut self.0;
            stream => stream.begin()
        )
    }

    pub fn end(
        &mut self,
        display_time: openxr::Time,
        environment_blend_mode: openxr::EnvironmentBlendMode,
        layers: &[&dyn CompositionLayer],
    ) -> Result<()> {
        graphics_match!(
            &mut self.0;
            stream => {
                let mut new_layers = vec![];

                for (i, layer) in layers.into_iter().enumerate() {
                    if let Some(swapchain) = layer.swapchain() {
                        if !swapchain.0.using_graphics::<Api>() {
                            warn!(
                                "composition layer {i} is using graphics api '{}', expected graphics api '{}'. Excluding layer from frame submission.",
                                swapchain.0.graphics_name(),
                                std::any::type_name::<Api>(),
                            );
                            continue;
                        }
                    }
                    new_layers.push(unsafe { std::mem::transmute(layer.header()) });
                }

                Ok(stream.end(display_time, environment_blend_mode, new_layers.as_slice())?)
            }
        )
    }
}

#[derive(Deref, DerefMut)]
pub struct XrFrameWaiter(pub openxr::FrameWaiter);

pub struct XrSwapchain(pub(crate) GraphicsWrap<Self>);

impl GraphicsType for XrSwapchain {
    type Inner<G: GraphicsExt> = openxr::Swapchain<G>;
}

impl XrSwapchain {
    pub fn acquire_image(&mut self) -> Result<u32> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.acquire_image()?)
        )
    }

    pub fn wait_image(&mut self, timeout: openxr::Duration) -> Result<()> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.wait_image(timeout)?)
        )
    }

    pub fn release_image(&mut self) -> Result<()> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.release_image()?)
        )
    }

    pub fn enumerate_images(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<Vec<wgpu::Texture>> {
        graphics_match!(
            &mut self.0;
            swap => {
                let mut images = vec![];
                for image in swap.enumerate_images()? {
                    unsafe {
                        images.push(Api::to_wgpu_img(image, device, format, resolution)?);
                    }
                }
                Ok(images)
            }
        )
    }
}
