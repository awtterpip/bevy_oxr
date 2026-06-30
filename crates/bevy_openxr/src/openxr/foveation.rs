use bevy_app::{App, Plugin};
use bevy_ecs::resource::Resource;
use openxr::{FoveationDynamicFB, FoveationLevelFB};

use crate::exts::OxrExtensions;

/// Fixed foveated rendering settings for OpenXR runtimes that expose the
/// `XR_FB_foveation` extension family.
///
/// Insert this resource before [`OxrInitPlugin`](crate::init::OxrInitPlugin)
/// is built so the OpenXR instance and graphics device can request the
/// required extensions.
#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct OxrFoveationConfig {
    pub level: FoveationLevelFB,
    pub dynamic: FoveationDynamicFB,
    pub vertical_offset: f32,
    pub use_fragment_density_map: bool,
    pub use_subsampled_layout: bool,
}

impl OxrFoveationConfig {
    pub const fn new(level: FoveationLevelFB) -> Self {
        Self {
            level,
            dynamic: FoveationDynamicFB::DISABLED,
            vertical_offset: 0.0,
            use_fragment_density_map: true,
            use_subsampled_layout: true,
        }
    }

    pub const fn low() -> Self {
        Self::new(FoveationLevelFB::LOW)
    }

    pub const fn medium() -> Self {
        Self::new(FoveationLevelFB::MEDIUM)
    }

    pub const fn high() -> Self {
        Self::new(FoveationLevelFB::HIGH)
    }

    pub const fn dynamic(mut self, dynamic: bool) -> Self {
        self.dynamic = if dynamic {
            FoveationDynamicFB::LEVEL_ENABLED
        } else {
            FoveationDynamicFB::DISABLED
        };
        self
    }

    pub const fn vertical_offset(mut self, vertical_offset: f32) -> Self {
        self.vertical_offset = vertical_offset;
        self
    }

    pub const fn without_subsampled_layout(mut self) -> Self {
        self.use_subsampled_layout = false;
        self
    }

    pub(crate) fn required_extensions(self) -> OxrExtensions {
        let mut exts = OxrExtensions::default();
        exts.raw_mut().fb_swapchain_update_state = true;
        exts.raw_mut().fb_foveation = true;
        exts.raw_mut().fb_foveation_configuration = true;
        if self.use_fragment_density_map {
            exts.raw_mut().fb_foveation_vulkan = true;
        }
        if self.use_subsampled_layout {
            exts.raw_mut().meta_vulkan_swapchain_create_info = true;
        }
        exts
    }
}

impl Default for OxrFoveationConfig {
    fn default() -> Self {
        Self::medium()
    }
}

pub struct OxrFoveationPlugin {
    pub config: OxrFoveationConfig,
}

impl OxrFoveationPlugin {
    pub const fn new(config: OxrFoveationConfig) -> Self {
        Self { config }
    }
}

impl Plugin for OxrFoveationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub(crate) struct OxrEnabledFoveationConfig(pub OxrFoveationConfig);

#[derive(Clone, Resource)]
pub struct OxrFoveationProfile(pub openxr::FoveationProfileFB);
