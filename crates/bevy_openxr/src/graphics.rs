#[cfg(feature = "vulkan")]
pub mod vulkan;

use std::any::TypeId;

use bevy::math::UVec2;

use crate::extensions::XrExtensions;
use crate::types::*;

pub unsafe trait GraphicsExt: openxr::Graphics {
    /// Wrap the graphics specific type into the [GraphicsWrap] enum
    fn wrap<T: GraphicsType>(item: T::Inner<Self>) -> GraphicsWrap<T>;
    /// Convert from wgpu format to the graphics format
    fn from_wgpu_format(format: wgpu::TextureFormat) -> Option<Self::Format>;
    /// Convert from the graphics format to wgpu format
    fn to_wgpu_format(format: Self::Format) -> Option<wgpu::TextureFormat>;
    /// Initialize graphics for this backend
    fn init_graphics(
        app_info: &AppInfo,
        instance: &openxr::Instance,
        system_id: openxr::SystemId,
    ) -> Result<(WgpuGraphics, Self::SessionCreateInfo)>;
    /// Convert a swapchain function
    unsafe fn to_wgpu_img(
        image: Self::SwapchainImage,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<wgpu::Texture>;
    fn required_exts() -> XrExtensions;
}

pub trait GraphicsType {
    type Inner<G: GraphicsExt>;
}

impl GraphicsType for () {
    type Inner<G: GraphicsExt> = ();
}

pub type GraphicsBackend = GraphicsWrap<()>;

impl GraphicsBackend {
    const ALL: &'static [Self] = &[Self::Vulkan(())];

    pub fn available_backends(exts: &XrExtensions) -> Vec<Self> {
        Self::ALL
            .iter()
            .copied()
            .filter(|backend| backend.is_available(exts))
            .collect()
    }

    pub fn is_available(&self, exts: &XrExtensions) -> bool {
        self.required_exts().is_available(exts)
    }

    pub fn required_exts(&self) -> XrExtensions {
        graphics_match!(
            self;
            _ => Api::required_exts()
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsWrap<T: GraphicsType> {
    #[cfg(feature = "vulkan")]
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

    fn graphics_type(&self) -> TypeId {
        graphics_match!(
            self;
            _ => TypeId::of::<Api>()
        )
    }

    /// Checks if this struct is using the wanted graphics api.
    pub fn using_graphics<G: GraphicsExt + 'static>(&self) -> bool {
        self.graphics_type() == TypeId::of::<G>()
    }

    /// Checks if the two values are both using the same graphics backend
    pub fn using_graphics_of_val<V: GraphicsType>(&self, other: &GraphicsWrap<V>) -> bool {
        self.graphics_type() == other.graphics_type()
    }
}

macro_rules! graphics_match {
    (
        $field:expr;
        $var:pat => $expr:expr $(=> $($return:tt)*)?
    ) => {
        match $field {
            #[cfg(feature = "vulkan")]
            $crate::graphics::GraphicsWrap::Vulkan($var) => {
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
