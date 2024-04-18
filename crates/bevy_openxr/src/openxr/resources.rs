use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use openxr::AnyGraphics;

use crate::error::OxrError;
use crate::graphics::*;
use crate::layer_builder::CompositionLayer;
use crate::types::*;

/// Wrapper around the entry point to the OpenXR API
#[derive(Deref, Clone)]
pub struct OxrEntry(pub openxr::Entry);

impl OxrEntry {
    /// Enumerate available extensions for this OpenXR runtime.
    pub fn enumerate_extensions(&self) -> Result<OxrExtensions> {
        Ok(self.0.enumerate_extensions().map(Into::into)?)
    }

    pub fn create_instance(
        &self,
        app_info: AppInfo,
        exts: OxrExtensions,
        layers: &[&str],
        backend: GraphicsBackend,
    ) -> Result<OxrInstance> {
        let available_exts = self.enumerate_extensions()?;

        if !backend.is_available(&available_exts) {
            return Err(OxrError::UnavailableBackend(backend));
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
            layers,
        )?;

        Ok(OxrInstance(instance, backend, app_info))
    }

    pub fn available_backends(&self) -> Result<Vec<GraphicsBackend>> {
        Ok(GraphicsBackend::available_backends(
            &self.enumerate_extensions()?,
        ))
    }
}

/// Wrapper around [openxr::Instance] with additional data for safety.
#[derive(Resource, Deref, Clone)]
pub struct OxrInstance(
    #[deref] pub openxr::Instance,
    pub(crate) GraphicsBackend,
    pub(crate) AppInfo,
);

impl OxrInstance {
    pub fn into_inner(self) -> openxr::Instance {
        self.0
    }

    /// Initialize graphics. This is used to create [WgpuGraphics] for the bevy app and to get the [SessionCreateInfo] to make an XR session.
    pub fn init_graphics(
        &self,
        system_id: openxr::SystemId,
    ) -> Result<(WgpuGraphics, SessionCreateInfo)> {
        graphics_match!(
            self.1;
            _ => {
                let (graphics, session_info) = Api::init_graphics(&self.2, &self, system_id)?;

                Ok((graphics, SessionCreateInfo(Api::wrap(session_info))))
            }
        )
    }

    /// Creates an [OxrSession]
    ///
    /// # Safety
    ///
    /// `info` must contain valid handles for the graphics api
    pub unsafe fn create_session(
        &self,
        system_id: openxr::SystemId,
        info: SessionCreateInfo,
    ) -> Result<(OxrSession, OxrFrameWaiter, OxrFrameStream)> {
        if !info.0.using_graphics_of_val(&self.1) {
            return Err(OxrError::GraphicsBackendMismatch {
                item: std::any::type_name::<SessionCreateInfo>(),
                backend: info.0.graphics_name(),
                expected_backend: self.1.graphics_name(),
            });
        }
        graphics_match!(
            info.0;
            info => {
                let (session, frame_waiter, frame_stream) = self.0.create_session::<Api>(system_id, &info)?;
                Ok((session.into(), OxrFrameWaiter(frame_waiter), OxrFrameStream(Api::wrap(frame_stream))))
            }
        )
    }
}

/// Graphics agnostic wrapper around [openxr::Session]
#[derive(Resource, Deref, Clone)]
pub struct OxrSession(
    #[deref] pub openxr::Session<AnyGraphics>,
    pub GraphicsWrap<Self>,
);

impl GraphicsType for OxrSession {
    type Inner<G: GraphicsExt> = openxr::Session<G>;
}

impl<G: GraphicsExt> From<openxr::Session<G>> for OxrSession {
    fn from(session: openxr::Session<G>) -> Self {
        Self::new(session)
    }
}

impl OxrSession {
    pub fn new<G: GraphicsExt>(session: openxr::Session<G>) -> Self {
        Self(session.clone().into_any_graphics(), G::wrap(session))
    }

    /// Enumerate all available swapchain formats.
    pub fn enumerate_swapchain_formats(&self) -> Result<Vec<wgpu::TextureFormat>> {
        graphics_match!(
            &self.1;
            session => Ok(session.enumerate_swapchain_formats()?.into_iter().filter_map(Api::into_wgpu_format).collect())
        )
    }

    /// Creates an [OxrSwapchain].
    pub fn create_swapchain(&self, info: SwapchainCreateInfo) -> Result<OxrSwapchain> {
        Ok(OxrSwapchain(graphics_match!(
            &self.1;
            session => session.create_swapchain(&info.try_into()?)? => OxrSwapchain
        )))
    }
}

/// Graphics agnostic wrapper around [openxr::FrameStream]
#[derive(Resource)]
pub struct OxrFrameStream(pub GraphicsWrap<Self>);

impl GraphicsType for OxrFrameStream {
    type Inner<G: GraphicsExt> = openxr::FrameStream<G>;
}

impl OxrFrameStream {
    /// Indicate that graphics device work is beginning.
    pub fn begin(&mut self) -> openxr::Result<()> {
        graphics_match!(
            &mut self.0;
            stream => stream.begin()
        )
    }

    /// Indicate that all graphics work for the frame has been submitted
    ///
    /// `layers` is an array of references to any type of composition layer,
    /// e.g. [`CompositionLayerProjection`](crate::oxr::layer_builder::CompositionLayerProjection)
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
                            error!(
                                "Composition layer {i} is using graphics api '{}', expected graphics api '{}'. Excluding layer from frame submission.",
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

/// Handle for waiting to render a frame. Check [`FrameWaiter`](openxr::FrameWaiter) for available methods.
#[derive(Resource, Deref, DerefMut)]
pub struct OxrFrameWaiter(pub openxr::FrameWaiter);

/// Graphics agnostic wrapper around [openxr::Swapchain]
#[derive(Resource)]
pub struct OxrSwapchain(pub GraphicsWrap<Self>);

impl GraphicsType for OxrSwapchain {
    type Inner<G: GraphicsExt> = openxr::Swapchain<G>;
}

impl OxrSwapchain {
    /// Determine the index of the next image to render to in the swapchain image array
    pub fn acquire_image(&mut self) -> Result<u32> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.acquire_image()?)
        )
    }

    /// Wait for the compositor to finish reading from the oldest unwaited acquired image
    pub fn wait_image(&mut self, timeout: openxr::Duration) -> Result<()> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.wait_image(timeout)?)
        )
    }

    /// Release the oldest acquired image
    pub fn release_image(&mut self) -> Result<()> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.release_image()?)
        )
    }

    /// Enumerates swapchain images and converts them to wgpu [`Texture`](wgpu::Texture)s.
    pub fn enumerate_images(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<OxrSwapchainImages> {
        graphics_match!(
            &self.0;
            swap => {
                let mut images = vec![];
                for image in swap.enumerate_images()? {
                    unsafe {
                        images.push(Api::to_wgpu_img(image, device, format, resolution)?);
                    }
                }
                Ok(OxrSwapchainImages(images.into()))
            }
        )
    }
}

/// Stores the generated swapchain images.
#[derive(Debug, Deref, Resource, Clone)]
pub struct OxrSwapchainImages(pub Arc<Vec<wgpu::Texture>>);

/// Thread safe wrapper around [openxr::Space] representing the stage.
#[derive(Deref, Clone, Resource)]
pub struct OxrStage(pub Arc<openxr::Space>);

/// Stores the latest generated [OxrViews]
#[derive(Clone, Resource, ExtractResource, Deref, DerefMut)]
pub struct OxrViews(pub Vec<openxr::View>);

/// Wrapper around [openxr::SystemId] to allow it to be stored as a resource.
#[derive(Debug, Copy, Clone, Deref, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Resource)]
pub struct OxrSystemId(pub openxr::SystemId);

/// Resource storing graphics info for the currently running session.
#[derive(Clone, Copy, Resource)]
pub struct OxrGraphicsInfo {
    pub blend_mode: EnvironmentBlendMode,
    pub resolution: UVec2,
    pub format: wgpu::TextureFormat,
}

#[derive(Clone)]
/// This is used to store information from startup that is needed to create the session after the instance has been created.
pub struct SessionConfigInfo {
    /// List of blend modes the openxr session can use. If [None], pick the first available blend mode.
    pub blend_modes: Option<Vec<EnvironmentBlendMode>>,
    /// List of formats the openxr session can use. If [None], pick the first available format
    pub formats: Option<Vec<wgpu::TextureFormat>>,
    /// List of resolutions that the openxr swapchain can use. If [None] pick the first available resolution.
    pub resolutions: Option<Vec<UVec2>>,
    /// Graphics info used to create a session.
    pub graphics_info: SessionCreateInfo,
}

#[derive(Resource, Clone, Default)]
pub struct OxrSessionStarted(Arc<AtomicBool>);

impl OxrSessionStarted {
    pub fn set(&self, val: bool) {
        self.0.store(val, Ordering::SeqCst);
    }

    pub fn get(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// The calculated display time for the app. Passed through the pipeline.
#[derive(Copy, Clone, Eq, PartialEq, Deref, DerefMut, Resource, ExtractResource)]
pub struct OxrTime(pub openxr::Time);

/// The root transform's global position for late latching in the render world.
#[derive(ExtractResource, Resource, Clone, Copy, Default)]
pub struct OxrRootTransform(pub GlobalTransform);

#[derive(ExtractResource, Resource, Clone, Copy, Default, Deref, DerefMut, PartialEq)]
/// This is inserted into the world to signify if the session should be cleaned up.
pub struct OxrCleanupSession(pub bool);
