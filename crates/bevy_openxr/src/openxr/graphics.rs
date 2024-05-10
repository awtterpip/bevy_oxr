#[cfg(all(feature = "d3d12", windows))]
mod d3d12;
#[cfg(feature = "vulkan")]
pub mod vulkan;

use std::any::TypeId;

use bevy::math::UVec2;

use crate::types::{AppInfo, OxrExtensions, Result, WgpuGraphics};

/// This is an extension trait to the [`Graphics`](openxr::Graphics) trait and is how the graphics API should be interacted with.
pub unsafe trait GraphicsExt: openxr::Graphics {
    /// Wrap the graphics specific type into the [GraphicsWrap] enum
    fn wrap<T: GraphicsType>(item: T::Inner<Self>) -> GraphicsWrap<T>;
    /// Returns all of the required openxr extensions to use this graphics API.
    fn required_exts() -> OxrExtensions;
    /// Convert from wgpu format to the graphics format
    fn from_wgpu_format(format: wgpu::TextureFormat) -> Option<Self::Format>;
    /// Convert from the graphics format to wgpu format
    fn into_wgpu_format(format: Self::Format) -> Option<wgpu::TextureFormat>;
    /// Convert an API specific swapchain image to a [`Texture`](wgpu::Texture).
    ///
    /// # Safety
    ///
    /// The `image` argument must be a valid handle.
    unsafe fn to_wgpu_img(
        image: Self::SwapchainImage,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<wgpu::Texture>;
    /// Initialize graphics for this backend and return a [`WgpuGraphics`] for bevy and an API specific [Self::SessionCreateInfo] for openxr
    fn init_graphics(
        app_info: &AppInfo,
        instance: &openxr::Instance,
        system_id: openxr::SystemId,
    ) -> Result<(WgpuGraphics, Self::SessionCreateInfo)>;
}

/// A type that can be used in [`GraphicsWrap`].
///
/// # Example
///
/// ```
/// pub struct XrSession(GraphicsWrap<XrSession>);
///
/// impl GraphicsType for XrSession {
///     type Inner<G: GraphicsExt> = openxr::Session<G>;
/// }
///
/// ```
///
/// In this example, `GraphicsWrap<XrSession>` is now an enum with variants for each graphics API. The enum can be matched to get the graphics specific inner type.
pub trait GraphicsType {
    type Inner<G: GraphicsExt>;
}

impl GraphicsType for () {
    type Inner<G: GraphicsExt> = ();
}

/// This is a special variant of [GraphicsWrap] using the unit struct as the inner type. This is to simply represent a graphics backend without storing data.
pub type GraphicsBackend = GraphicsWrap<()>;

impl GraphicsBackend {
    const ALL: &'static [Self] = &[
        #[cfg(feature = "vulkan")]
        Self::Vulkan(()),
        #[cfg(all(feature = "d3d12", windows))]
        Self::D3D12(()),
    ];

    pub fn available_backends(exts: &OxrExtensions) -> Vec<Self> {
        Self::ALL
            .iter()
            .copied()
            .filter(|backend| backend.is_available(exts))
            .collect()
    }

    pub fn is_available(&self, exts: &OxrExtensions) -> bool {
        self.required_exts().is_available(exts)
    }

    pub fn required_exts(&self) -> OxrExtensions {
        graphics_match!(
            self;
            _ => Api::required_exts()
        )
    }
}

/// This struct is for creating agnostic objects for OpenXR graphics API specific structs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsWrap<T: GraphicsType> {
    #[cfg(feature = "vulkan")]
    Vulkan(T::Inner<openxr::Vulkan>),
    #[cfg(all(feature = "d3d12", windows))]
    D3D12(T::Inner<openxr::D3D12>),
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

/// This macro can be used to quickly run the same code for every variant of [GraphicsWrap].
/// The first argument should be an expression that returns either a reference or owned value of [GraphicsWrap].
/// The second argument should be a match arm with the pattern for the inner type.
///
/// # Example
///
/// ```
/// pub struct OxrFrameStream(GraphicsWrap<OxrFrameStream>);
///
/// impl GraphicsType for OxrFrameStream {
///     // Here is the inner type
///     type Inner<G: GraphicsExt> = openxr::FrameStream<G>;
/// }
///
/// fn begin(frame_stream: &mut OxrFrameStream) {
///     graphics_match! {
///         // get the inner 'GraphicsWrap' struct
///         &mut frame_stream.0;
///         // now we can handle every match arm with a single arm
///         // important: the first pattern here represents the inner type of `GraphicsWrap`
///         // it is already destructured for you.
///         stream => stream.begin();
///     }
/// }
/// ```
///
/// Additionally, if you want the type that implements `GraphicsExt` in the scope of the match body, you can access that type through the type alias `Api`.
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
            #[cfg(all(feature = "d3d12", windows))]
            $crate::graphics::GraphicsWrap::D3D12($var) => {
                #[allow(unused)]
                type Api = openxr::D3D12;
                graphics_match!(@arm_impl D3D12; $expr $(=> $($return)*)?)
            },
        }
    };

    (
        @arm_impl
        $variant:ident;
        $expr:expr => $wrap_ty:ty
    ) => {
        $crate::graphics::GraphicsWrap::<$wrap_ty>::$variant($expr)
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
