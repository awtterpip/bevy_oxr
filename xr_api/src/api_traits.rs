use crate::prelude::*;

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
    /// Request input modules with the specified bindings.
    fn create_input(&self, bindings: Bindings) -> Result<Input>;
    /// Blocks until a rendering frame is available and then begins it.
    fn begin_frame(&self) -> Result<()>;
    /// Submits rendering work for this frame.
    fn end_frame(&self) -> Result<()>;
}

pub trait InputTrait {
    fn get_haptics(&self, path: ActionId) -> Result<Action<Haptic>>;
    fn get_pose(&self, path: ActionId) -> Result<Action<Pose>>;
    fn get_float(&self, path: ActionId) -> Result<Action<f32>>;
    fn get_bool(&self, path: ActionId) -> Result<Action<bool>>;
}

pub trait ActionTrait {
    fn id(&self) -> ActionId;
}

pub trait ActionInputTrait<A> {}

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
