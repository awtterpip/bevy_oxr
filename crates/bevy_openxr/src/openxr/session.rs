use std::ffi::c_void;
use std::ptr;

use crate::foveation::{OxrFoveationConfig, OxrFoveationProfile};
use crate::next_chain::{OxrNextChain, OxrNextChainStructBase, OxrNextChainStructProvider};
use crate::resources::{OxrPassthrough, OxrPassthroughLayerFB, OxrSwapchain};
use crate::types::{Result, SwapchainCreateInfo};
use bevy_derive::Deref;
use bevy_ecs::resource::Resource;
use openxr::AnyGraphics;
use openxr::sys::Handle as _;

use crate::graphics::{GraphicsExt, GraphicsType, GraphicsWrap, graphics_match};

const VK_IMAGE_CREATE_SUBSAMPLED_BIT_EXT: openxr::sys::platform::VkImageCreateFlags = 0x0000_4000;

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

    /// Creates an [`OxrSwapchain`] with `XR_FB_foveation` enabled and applies
    /// a foveation profile to it.
    ///
    /// On Vulkan runtimes this requests a fragment-density-map swapchain. The
    /// returned profile is kept alive by the caller for as long as the
    /// swapchain may use it.
    pub fn create_foveated_swapchain(
        &self,
        info: SwapchainCreateInfo,
        config: OxrFoveationConfig,
    ) -> Result<(OxrSwapchain, Option<OxrFoveationProfile>)> {
        Ok(graphics_match!(
            &self.1;
            session => {
                let (swapchain, profile) = create_foveated_swapchain(session, info, config)?;
                (OxrSwapchain(Api::wrap(swapchain)), profile)
            }
        ))
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
    ) -> Result<OxrPassthroughLayerFB> {
        Ok(OxrPassthroughLayerFB(graphics_match! {
            &self.1;
            session => session.create_passthrough_layer(&passthrough.0, passthrough.1, purpose)?
        }))
    }
}

fn create_foveated_swapchain<G: GraphicsExt>(
    session: &openxr::Session<G>,
    info: SwapchainCreateInfo,
    config: OxrFoveationConfig,
) -> Result<(openxr::Swapchain<G>, Option<OxrFoveationProfile>)> {
    let Some(format) = G::from_wgpu_format(info.format) else {
        return Err(crate::error::OxrError::UnsupportedTextureFormat(
            info.format,
        ));
    };

    let mut vulkan_create_info = openxr::sys::VulkanSwapchainCreateInfoMETA {
        ty: openxr::sys::VulkanSwapchainCreateInfoMETA::TYPE,
        next: ptr::null(),
        additional_create_flags: 0,
        additional_usage_flags: 0,
    };
    if config.use_subsampled_layout {
        vulkan_create_info.additional_create_flags = VK_IMAGE_CREATE_SUBSAMPLED_BIT_EXT;
    }

    let mut foveation_create_info = openxr::sys::SwapchainCreateInfoFoveationFB {
        ty: openxr::sys::SwapchainCreateInfoFoveationFB::TYPE,
        next: if config.use_subsampled_layout {
            &vulkan_create_info as *const _ as *mut _
        } else {
            ptr::null_mut()
        },
        flags: if config.use_fragment_density_map {
            openxr::sys::SwapchainCreateFoveationFlagsFB::FRAGMENT_DENSITY_MAP
        } else {
            openxr::sys::SwapchainCreateFoveationFlagsFB::SCALED_BIN
        },
    };

    let create_info = openxr::sys::SwapchainCreateInfo {
        ty: openxr::sys::SwapchainCreateInfo::TYPE,
        next: &mut foveation_create_info as *mut _ as *const _,
        create_flags: info.create_flags,
        usage_flags: info.usage_flags,
        format: G::lower_format(format),
        sample_count: info.sample_count,
        width: info.width,
        height: info.height,
        face_count: info.face_count,
        array_size: info.array_size,
        mip_count: info.mip_count,
    };

    let mut swapchain = openxr::sys::Swapchain::NULL;
    let result = unsafe {
        (session.instance().fp().create_swapchain)(session.as_raw(), &create_info, &mut swapchain)
    };
    if result.into_raw() < 0 {
        return Err(result.into());
    }

    let swapchain = unsafe { openxr::Swapchain::from_raw(session.clone(), swapchain) };
    let profile = session.create_foveation_profile(Some(openxr::FoveationLevelProfile {
        level: config.level,
        vertical_offset: config.vertical_offset,
        dynamic: config.dynamic,
    }))?;
    let state = openxr::sys::SwapchainStateFoveationFB {
        ty: openxr::sys::SwapchainStateFoveationFB::TYPE,
        next: ptr::null_mut(),
        flags: openxr::sys::SwapchainStateFoveationFlagsFB::EMPTY,
        profile: profile.as_raw(),
    };
    let Some(update_swapchain) = session.instance().exts().fb_swapchain_update_state.as_ref()
    else {
        return Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT.into());
    };
    let result = unsafe {
        (update_swapchain.update_swapchain)(
            swapchain.as_raw(),
            &state as *const _ as *const openxr::sys::SwapchainStateBaseHeaderFB,
        )
    };
    if result.into_raw() < 0 {
        return Err(result.into());
    }

    Ok((swapchain, Some(OxrFoveationProfile(profile))))
}

pub trait OxrSessionCreateNextProvider: OxrNextChainStructProvider {}

/// NonSend Resource
#[derive(Default)]
pub struct OxrSessionCreateNextChain(OxrNextChain);

impl OxrSessionCreateNextChain {
    pub fn push<T: OxrSessionCreateNextProvider>(&mut self, info_struct: T) {
        self.0.push(info_struct)
    }
    pub fn chain(&self) -> Option<&OxrNextChainStructBase> {
        self.0.chain()
    }
    pub fn chain_pointer(&self) -> *const c_void {
        self.0.chain_pointer()
    }
}
