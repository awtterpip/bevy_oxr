pub mod vulkan;

use bevy::math::UVec2;

use crate::openxr::resources::*;
use crate::openxr::types::{AppInfo, Result, XrError};
use crate::types::BlendMode;

pub trait GraphicsExt: openxr::Graphics {
    fn from_wgpu_format(format: wgpu::TextureFormat) -> Option<Self::Format>;
    fn to_wgpu_format(format: Self::Format) -> Option<wgpu::TextureFormat>;
    fn create_session(
        app_info: &AppInfo,
        instance: &openxr::Instance,
        system_id: openxr::SystemId,
    ) -> Result<(
        wgpu::Device,
        wgpu::Queue,
        wgpu::Adapter,
        wgpu::Instance,
        XrSession,
        XrFrameWaiter,
        XrFrameStream,
    )>;
    unsafe fn to_wgpu_img(
        image: Self::SwapchainImage,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<wgpu::Texture>;
    fn required_exts() -> XrExtensions;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsWrap<T: GraphicsType> {
    Vulkan(T::Inner<openxr::Vulkan>),
}

impl<T: GraphicsType> GraphicsWrap<T> {
    /// Returns the name of the graphics api this struct is using.
    pub fn graphics_name(&self) -> &'static str {
        graphics_match!(
            self;
            _ => std::any::type_name::<Api>()
        )
    }

    fn graphics_type(&self) -> std::any::TypeId {
        graphics_match!(
            self;
            _ => std::any::TypeId::of::<Api>()
        )
    }

    /// Checks if this struct is using the wanted graphics api.
    pub fn using_graphics<G: GraphicsExt + 'static>(&self) -> bool {
        self.graphics_type() == std::any::TypeId::of::<G>()
    }
}

pub trait GraphicsType {
    type Inner<G: GraphicsExt>;
}

impl GraphicsType for () {
    type Inner<G: GraphicsExt> = ();
}

macro_rules! graphics_match {
    (
        $field:expr;
        $var:pat => $expr:expr $(=> $($return:tt)*)?
    ) => {
        match $field {
            $crate::openxr::graphics::GraphicsWrap::Vulkan($var) => {
                #[allow(unused)]
                type Api = openxr::Vulkan;
                graphics_match!(@arm_impl Vulkan; $expr $(=> $($return)*)?)
            },
        }
    };

    (
        @arm_impl
        $variant:ident;
        $expr:expr => $wrap_ty:ty
    ) => {
        GraphicsWrap::<$wrap_ty>::$variant($expr)
    };

    (
        @arm_impl
        $variant:ident;
        $expr:expr
    ) => {
        $expr
    };
}

pub(crate) use graphics_match;

use super::XrExtensions;

impl From<openxr::EnvironmentBlendMode> for BlendMode {
    fn from(value: openxr::EnvironmentBlendMode) -> Self {
        use openxr::EnvironmentBlendMode;
        if value == EnvironmentBlendMode::OPAQUE {
            BlendMode::Opaque
        } else if value == EnvironmentBlendMode::ADDITIVE {
            BlendMode::Additive
        } else if value == EnvironmentBlendMode::ALPHA_BLEND {
            BlendMode::AlphaBlend
        } else {
            unreachable!()
        }
    }
}

impl From<BlendMode> for openxr::EnvironmentBlendMode {
    fn from(value: BlendMode) -> Self {
        use openxr::EnvironmentBlendMode;
        match value {
            BlendMode::Opaque => EnvironmentBlendMode::OPAQUE,
            BlendMode::Additive => EnvironmentBlendMode::ADDITIVE,
            BlendMode::AlphaBlend => EnvironmentBlendMode::ALPHA_BLEND,
        }
    }
}
