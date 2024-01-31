use glam::{UVec2, Vec2};
use wgpu::{Adapter, AdapterInfo, Device, Queue, TextureView};

use crate::prelude::*;

use self::path::{InputComponent, UntypedActionPath};

pub trait EntryTrait {
    /// Return currently available extensions
    fn available_extensions(&self) -> Result<ExtensionSet>;
    /// Create an [Instance] with the enabled extensions.
    fn create_instance(&self, exts: ExtensionSet) -> Result<Instance>;
}

pub trait InstanceTrait {
    /// Returns the [Entry] used to create this.
    fn entry(&self) -> Entry;
    /// Returns an [ExtensionSet] listing all enabled extensions.
    fn enabled_extensions(&self) -> ExtensionSet;
    /// Creates a [Session] with the requested properties
    fn create_session(&self, info: SessionCreateInfo) -> Result<Session>;
}

pub trait SessionTrait {
    /// Returns the [Instance] used to create this.
    fn instance(&self) -> &Instance;
    /// Get render resources compatible with this session.
    fn get_render_resources(&self)
        -> Option<(Device, Queue, AdapterInfo, Adapter, wgpu::Instance)>;
    /// Request input modules with the specified bindings.
    fn create_input(&self, bindings: Bindings) -> Result<Input>;
    /// Blocks until a rendering frame is available, then returns the views for the left and right eyes.
    fn begin_frame(&self) -> Result<(View, View)>;
    /// Submits rendering work for this frame.
    fn end_frame(&self) -> Result<()>;
    /// Gets the resolution of a single eye.
    fn resolution(&self) -> UVec2;
    /// Gets the texture format for the session.
    fn format(&self) -> wgpu::TextureFormat;
}

pub trait ViewTrait {
    /// Returns the [TextureView] used to render this view.
    fn texture_view(&self) -> Option<TextureView>;
    /// Returns the [Pose] representing the current position of this view.
    fn pose(&self) -> Pose;
    /// Returns the projection matrix for the current view.
    fn projection_matrix(&self) -> glam::Mat4;
    /// Gets the resolution for this view.
    fn resolution(&self) -> UVec2;
    /// Gets the texture format for the view.
    fn format(&self) -> wgpu::TextureFormat;
}

pub trait InputTrait {
    /// Get the haptic action at the specified path.
    fn create_action_haptics(&self, name: &str, path: UntypedActionPath) -> Result<Action<Haptic>>;
    /// Get the pose action at the specified path.
    fn create_action_pose(&self, name: &str, path: UntypedActionPath) -> Result<Action<Pose>>;
    /// Get the float action at the specified path.
    fn create_action_float(&self, name: &str, path: UntypedActionPath) -> Result<Action<f32>>;
    /// Get the boolean action at the specified path.
    fn create_action_bool(&self, name: &str, path: UntypedActionPath) -> Result<Action<bool>>;
    /// Get the Vec2 action at the specified path.
    fn create_action_vec2(&self, name: &str, path: UntypedActionPath) -> Result<Action<Vec2>>;
}

// This impl is moved outside of the trait to ensure that InputTrait stays object safe.
impl dyn InputTrait {
    /// Get the action at the specified path.
    pub fn create_action<P: InputComponent>(
        &self,
        name: &str,
        path: ActionPath<P>,
    ) -> Result<Action<P::PathType>> {
        P::PathType::get(self, name, path.untyped())
    }
}

/// Represents input actions, such as bools, floats, and poses
pub trait ActionInputTrait<A> {
    fn get(&self) -> A;
}

/// Represents haptic actions.
pub trait HapticTrait {}

impl<T: InstanceTrait> EntryTrait for T {
    fn available_extensions(&self) -> Result<ExtensionSet> {
        self.entry().available_extensions()
    }

    fn create_instance(&self, exts: ExtensionSet) -> Result<Instance> {
        self.entry().create_instance(exts)
    }
}

impl<T: SessionTrait> InstanceTrait for T {
    fn entry(&self) -> Entry {
        self.instance().entry()
    }

    fn enabled_extensions(&self) -> ExtensionSet {
        self.instance().enabled_extensions()
    }

    fn create_session(&self, info: SessionCreateInfo) -> Result<Session> {
        self.instance().create_session(info)
    }
}
