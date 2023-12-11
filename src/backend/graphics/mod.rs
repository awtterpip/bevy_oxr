#[cfg(feature = "vulkan")]
pub mod vulkan;

use bevy::window::RawHandleWrapper;
use openxr::{FrameStream, FrameWaiter, Instance, Swapchain, ViewConfigurationType};

use crate::{error::XrError, types::GraphicsFeatures};

use super::OXrSession;

const VIEW_TYPE: ViewConfigurationType = ViewConfigurationType::PRIMARY_STEREO;

pub enum OXrGraphics {
    #[cfg(feature = "vulkan")]
    Vulkan {
        swapchain: Swapchain<openxr::Vulkan>,
        frame_stream: FrameStream<openxr::Vulkan>,
        frame_waiter: FrameWaiter,
    },
}

pub fn init_oxr_graphics(
    instance: Instance,
    graphics: GraphicsFeatures,
    window: Option<RawHandleWrapper>,
) -> Result<OXrSession, XrError> {
    #[cfg(feature = "vulkan")]
    if graphics.vulkan {
        if let Ok(session) = vulkan::init_oxr_graphics(instance, window) {
            return Ok(session);
        }
    }

    Err(XrError {})
}
