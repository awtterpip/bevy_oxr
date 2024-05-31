use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use openxr::AnyGraphics;

use crate::error::OxrError;
use crate::graphics::*;
use crate::layer_builder::{CompositionLayer, LayerProvider};
use crate::types::*;

/// Wrapper around an [`Entry`](openxr::Entry) with some methods overridden to use bevy types.
///
/// See [`openxr::Entry`] for other available methods.
#[derive(Deref, Clone)]
pub struct OxrEntry(pub openxr::Entry);

impl OxrEntry {
    /// Enumerate available extensions for this OpenXR runtime.
    pub fn enumerate_extensions(&self) -> Result<OxrExtensions> {
        Ok(self.0.enumerate_extensions().map(Into::into)?)
    }

    /// Creates an [`OxrInstance`].
    ///
    /// Calls [`create_instance`](openxr::Entry::create_instance) internally.
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

    /// Returns a list of all of the backends the OpenXR runtime supports.
    pub fn available_backends(&self) -> Result<Vec<GraphicsBackend>> {
        Ok(GraphicsBackend::available_backends(
            &self.enumerate_extensions()?,
        ))
    }
}

/// Wrapper around [`openxr::Instance`] with additional data for safety and some methods overriden to use bevy types.
///
/// See [`openxr::Instance`] for other available methods.
#[derive(Resource, Deref, Clone)]
pub struct OxrInstance(
    #[deref] pub(crate) openxr::Instance,
    /// [`GraphicsBackend`] is stored here to let us know what graphics API the current instance wants to target.
    pub(crate) GraphicsBackend,
    pub(crate) AppInfo,
);

impl OxrInstance {
    /// Creates an [`OxrInstance`] from an [`openxr::Instance`] if needed.
    /// In the majority of cases, you should use [`create_instance`](OxrEntry::create_instance) instead.
    ///
    /// # Safety
    ///
    /// The OpenXR instance passed in *must* have support for the backend specified.
    pub unsafe fn from_inner(
        instance: openxr::Instance,
        backend: GraphicsBackend,
        info: AppInfo,
    ) -> Self {
        Self(instance, backend, info)
    }

    /// Consumes self and returns the inner [`openxr::Instance`]
    pub fn into_inner(self) -> openxr::Instance {
        self.0
    }

    /// Returns the current backend being used by this instance.
    pub fn backend(&self) -> GraphicsBackend {
        self.1
    }

    /// Returns the [`AppInfo`] being used by this instance.
    pub fn app_info(&self) -> &AppInfo {
        &self.2
    }

    /// Initialize graphics. This is used to create [WgpuGraphics] for the bevy app and to get the [SessionCreateInfo] needed to make an XR session.
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
    /// Calls [`create_session`](openxr::Instance::create_session) internally.
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

/// Graphics agnostic wrapper around [openxr::Session].
///
/// See [`openxr::Session`] for other available methods.
#[derive(Resource, Deref, Clone)]
pub struct OxrSession(
    /// A session handle with [`AnyGraphics`].
    /// Having this here allows the majority of [`Session`](openxr::Session)'s methods to work without having to rewrite them.
    #[deref]
    pub(crate) openxr::Session<AnyGraphics>,
    /// A [`GraphicsWrap`] with [`openxr::Session<G>`] as the inner type.
    /// This is so that we can still operate on functions that don't take [`AnyGraphics`] as the generic.
    pub(crate) GraphicsWrap<Self>,
);

impl GraphicsType for OxrSession {
    type Inner<G: GraphicsExt> = openxr::Session<G>;
}

impl<G: GraphicsExt> From<openxr::Session<G>> for OxrSession {
    fn from(session: openxr::Session<G>) -> Self {
        Self::from_inner(session)
    }
}

impl OxrSession {
    /// Creates a new [`OxrSession`] from an [`openxr::Session`].
    /// In the majority of cases, you should use [`create_session`](OxrInstance::create_session) instead.
    pub fn from_inner<G: GraphicsExt>(session: openxr::Session<G>) -> Self {
        Self(session.clone().into_any_graphics(), G::wrap(session))
    }

    /// Returns [`GraphicsWrap`] with [`openxr::Session<G>`] as the inner type.
    ///
    /// This can be useful if you need access to the original [`openxr::Session`] with the graphics API still specified.
    pub fn typed_session(&self) -> &GraphicsWrap<Self> {
        &self.1
    }

    /// Enumerates all available swapchain formats and converts them to wgpu's [`TextureFormat`](wgpu::TextureFormat).
    ///
    /// Calls [`enumerate_swapchain_formats`](openxr::Session::enumerate_swapchain_formats) internally.
    pub fn enumerate_swapchain_formats(&self) -> Result<Vec<wgpu::TextureFormat>> {
        graphics_match!(
            &self.1;
            session => Ok(session.enumerate_swapchain_formats()?.into_iter().filter_map(Api::into_wgpu_format).collect())
        )
    }

    /// Creates an [OxrSwapchain].
    ///
    /// Calls [`create_swapchain`](openxr::Session::create_swapchain) internally.
    pub fn create_swapchain(&self, info: SwapchainCreateInfo) -> Result<OxrSwapchain> {
        Ok(OxrSwapchain(graphics_match!(
            &self.1;
            session => session.create_swapchain(&info.try_into()?)? => OxrSwapchain
        )))
    }

    /// Creates a passthrough.
    ///
    /// Requires [`XR_FB_passthrough`](https://www.khronos.org/registry/OpenXR/specs/1.0/html/xrspec.html#XR_FB_passthrough).
    ///
    /// Calls [`create_passthrough`](openxr::Session::create_passthrough) internally.
    pub fn create_passthrough(&self, flags: openxr::PassthroughFlagsFB) -> Result<OxrPassthrough> {
        Ok(OxrPassthrough(
            graphics_match! {
                &self.1;
                session => session.create_passthrough(flags)?
            },
            flags,
        ))
    }

    /// Creates a passthrough layer that can be used to make a [`CompositionLayerPassthrough`](crate::layer_builder::CompositionLayerPassthrough) for frame submission.
    ///
    /// Requires [`XR_FB_passthrough`](https://www.khronos.org/registry/OpenXR/specs/1.0/html/xrspec.html#XR_FB_passthrough).
    ///
    /// Calls [`create_passthrough_layer`](openxr::Session::create_passthrough_layer) internally.
    pub fn create_passthrough_layer(
        &self,
        passthrough: &OxrPassthrough,
        purpose: openxr::PassthroughLayerPurposeFB,
    ) -> Result<OxrPassthroughLayer> {
        Ok(OxrPassthroughLayer(graphics_match! {
            &self.1;
            session => session.create_passthrough_layer(&passthrough.0, passthrough.1, purpose)?
        }))
    }
}

/// Graphics agnostic wrapper around [openxr::FrameStream]
#[derive(Resource)]
pub struct OxrFrameStream(pub GraphicsWrap<Self>);

impl GraphicsType for OxrFrameStream {
    type Inner<G: GraphicsExt> = openxr::FrameStream<G>;
}

impl OxrFrameStream {
    /// Creates a new [`OxrFrameStream`] from an [`openxr::FrameStream`].
    /// In the majority of cases, you should use [`create_session`](OxrInstance::create_session) instead.
    pub fn from_inner<G: GraphicsExt>(frame_stream: openxr::FrameStream<G>) -> Self {
        Self(G::wrap(frame_stream))
    }

    /// Indicate that graphics device work is beginning.
    ///
    /// Calls [`begin`](openxr::FrameStream::begin) internally.
    pub fn begin(&mut self) -> openxr::Result<()> {
        graphics_match!(
            &mut self.0;
            stream => stream.begin()
        )
    }

    /// Indicate that all graphics work for the frame has been submitted
    ///
    /// `layers` is an array of references to any type of composition layer that implements [`CompositionLayer`],
    /// e.g. [`CompositionLayerProjection`](crate::layer_builder::CompositionLayerProjection)
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

/// Handle for waiting to render a frame.
///
/// See [`FrameWaiter`](openxr::FrameWaiter) for available methods.
#[derive(Resource, Deref, DerefMut)]
pub struct OxrFrameWaiter(pub openxr::FrameWaiter);

/// Graphics agnostic wrapper around [openxr::Swapchain]
#[derive(Resource)]
pub struct OxrSwapchain(pub GraphicsWrap<Self>);

impl GraphicsType for OxrSwapchain {
    type Inner<G: GraphicsExt> = openxr::Swapchain<G>;
}

impl OxrSwapchain {
    /// Creates a new [`OxrSwapchain`] from an [`openxr::Swapchain`].
    /// In the majority of cases, you should use [`create_swapchain`](OxrSession::create_swapchain) instead.
    pub fn from_inner<G: GraphicsExt>(swapchain: openxr::Swapchain<G>) -> Self {
        Self(G::wrap(swapchain))
    }

    /// Determine the index of the next image to render to in the swapchain image array.
    ///
    /// Calls [`acquire_image`](openxr::Swapchain::acquire_image) internally.
    pub fn acquire_image(&mut self) -> Result<u32> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.acquire_image()?)
        )
    }

    /// Wait for the compositor to finish reading from the oldest unwaited acquired image.
    ///
    /// Calls [`wait_image`](openxr::Swapchain::wait_image) internally.
    pub fn wait_image(&mut self, timeout: openxr::Duration) -> Result<()> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.wait_image(timeout)?)
        )
    }

    /// Release the oldest acquired image.
    ///
    /// Calls [`release_image`](openxr::Swapchain::release_image) internally.
    pub fn release_image(&mut self) -> Result<()> {
        graphics_match!(
            &mut self.0;
            swap => Ok(swap.release_image()?)
        )
    }

    /// Enumerates swapchain images and converts them to wgpu [`Texture`](wgpu::Texture)s.
    ///
    /// Calls [`enumerate_images`](openxr::Swapchain::enumerate_images) internally.
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
// #[derive(Deref, Clone, Resource)]
// pub struct OxrStage(pub Arc<openxr::Space>);

/// Stores the latest generated [OxrViews]
#[derive(Clone, Resource, Deref, DerefMut)]
pub struct OxrViews(pub Vec<openxr::View>);

/// Wrapper around [openxr::SystemId] to allow it to be stored as a resource.
#[derive(Debug, Copy, Clone, Deref, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Resource)]
pub struct OxrSystemId(pub openxr::SystemId);

/// Wrapper around [`openxr::Passthrough`].
///
/// Used to [`start`](openxr::Passthrough::start) or [`pause`](openxr::Passthrough::pause) passthrough on the physical device.
///
/// See [`openxr::Passthrough`] for available methods.
#[derive(Resource, Deref, DerefMut)]
pub struct OxrPassthrough(
    #[deref] pub openxr::Passthrough,
    /// The flags are stored here so that they don't need to be passed in again when creating an [`OxrPassthroughLayer`].
    openxr::PassthroughFlagsFB,
);

impl OxrPassthrough {
    /// This function can create an [`OxrPassthrough`] from raw openxr types if needed.
    /// In the majority of cases, you should use [`create_passthrough`](OxrSession::create_passthrough) instead.
    pub fn from_inner(passthrough: openxr::Passthrough, flags: openxr::PassthroughFlagsFB) -> Self {
        Self(passthrough, flags)
    }
}

/// Wrapper around [`openxr::Passthrough`].
///
/// Used to create a [`CompositionLayerPassthrough`](crate::layer_builder::CompositionLayerPassthrough), and to [`pause`](openxr::PassthroughLayer::pause) or [`resume`](openxr::PassthroughLayer::resume) rendering of the passthrough layer.
///
/// See [`openxr::PassthroughLayer`] for available methods.
#[derive(Resource, Deref, DerefMut)]
pub struct OxrPassthroughLayer(pub openxr::PassthroughLayer);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct OxrRenderLayers(pub Vec<Box<dyn LayerProvider + Send + Sync>>);

/// Resource storing graphics info for the currently running session.
#[derive(Clone, Copy, Resource)]
pub struct OxrGraphicsInfo {
    pub blend_mode: EnvironmentBlendMode,
    pub resolution: UVec2,
    pub format: wgpu::TextureFormat,
}

#[derive(Clone)]
/// This is used to store information from startup that is needed to create the session after the instance has been created.
pub struct OxrSessionConfigInfo {
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

#[derive(Clone, Resource)]
pub struct OxrFrameState(pub Arc<Mutex<Option<openxr::FrameState>>>);

/// The root transform's global position for late latching in the render world.
#[derive(ExtractResource, Resource, Clone, Copy, Default)]
pub struct OxrRootTransform(pub GlobalTransform);

#[derive(Resource, Clone, Default)]
/// This is inserted into the world to signify if the session should be cleaned up.
pub struct OxrCleanupSession(Arc<AtomicBool>);

impl OxrCleanupSession {
    pub fn set(&self, val: bool) {
        self.0.store(val, Ordering::SeqCst);
    }

    pub fn get(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
