use bevy::{
    render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue},
    window::RawHandleWrapper,
};

/// This struct stores all of the render resources required to initialize the bevy render plugin
///
/// Returned from [XrSessionTrait::get_render_resources](crate::backend::traits::XrSessionTrait::get_render_resources)
pub struct RenderResources {
    pub adapter: RenderAdapter,
    pub adapter_info: RenderAdapterInfo,
    pub device: RenderDevice,
    pub queue: RenderQueue,
}

/// Information used to create the [XrSession](crate::resources::XrSession)
///
/// Passed into [XrInstanceTrait::create_session](crate::backend::traits::XrInstanceTrait::create_session)
pub struct SessionCreateInfo {
    /// This field is required to be [Some] when using WebXR
    pub canvas: Option<String>,
    pub window: Option<RawHandleWrapper>,
}

#[derive(Clone, Copy)]
pub enum ControllerType {
    OculusTouch,
}

#[derive(Clone)]
pub struct ActionSetCreateInfo {
    pub controller: ControllerType,
    // TODO!() allow custom fields
}

pub struct ReferenceSpaceInfo {}

pub struct ActionSpaceInfo {}

#[derive(Clone, Copy, Default)]
pub struct FeatureList {
    pub graphics: GraphicsFeatures,
}

#[derive(Clone, Copy, Default)]
pub struct GraphicsFeatures {
    pub vulkan: bool,
}

pub struct Pose;

pub struct Haptics;

pub enum ActionState {
    Bool(bool),
    Float(f32),
    Haptics(Haptics),
    Pose(Pose),
}

pub enum ActionType {
    Bool,
    Float,
    Haptics,
    Pose,
}

pub struct ActionId {
    pub action_path: ActionPath,
    pub side: Option<Side>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ActionPath {
    HandPose,
    PointerPose,
    GripPull,
    TriggerPull,
    TriggerTouch,
    HapticFeedback,
    PrimaryButton,
    PrimaryButtonTouch,
    SecondaryButton,
    SecondaryButtonTouch,
    MenuButton,
    ThumbstickX,
    ThumbstickY,
    ThumbstickTouch,
    ThumbstickClick,
    ThumbrestTouch,
}

impl ActionPath {
    pub fn action_name(&self) -> &'static str {
        use ActionPath::*;
        match self {
            HandPose => "hand_pose",
            PointerPose => "pointer_pose",
            GripPull => "grip_pull",
            TriggerPull => "trigger_pull",
            TriggerTouch => "trigger_touch",
            HapticFeedback => "haptic_feedback",
            PrimaryButton => "primary_button",
            PrimaryButtonTouch => "primary_button_touch",
            SecondaryButton => "secondary_button",
            SecondaryButtonTouch => "secondary_button_touch",
            MenuButton => "menu_button",
            ThumbstickX => "thumbstick_x",
            ThumbstickY => "thumbstick_y",
            ThumbstickTouch => "thumbstick_touch",
            ThumbstickClick => "thumbstick_click",
            ThumbrestTouch => "thumbrest_touch",
        }
    }

    pub fn pretty_action_name(&self) -> &'static str {
        use ActionPath::*;
        match self {
            HandPose => "Hand Pose",
            PointerPose => "Pointer Pose",
            GripPull => "Grip Pull",
            TriggerPull => "Trigger Pull",
            TriggerTouch => "Trigger Touch",
            HapticFeedback => "Haptic Feedback",
            PrimaryButton => "Primary Button",
            PrimaryButtonTouch => "Primary Button Touch",
            SecondaryButton => "Secondary Button",
            SecondaryButtonTouch => "Secondary Button Touch",
            MenuButton => "Menu Button",
            ThumbstickX => "Thumbstick X",
            ThumbstickY => "Thumbstick Y",
            ThumbstickTouch => "Thumbstick Touch",
            ThumbstickClick => "Thumbstick Click",
            ThumbrestTouch => "Thumbrest Touch",
        }
    }

    pub fn action_type(&self) -> ActionType {
        use ActionPath::*;
        match self {
            HandPose => ActionType::Pose,
            PointerPose => ActionType::Pose,
            GripPull => ActionType::Float,
            TriggerPull => ActionType::Float,
            TriggerTouch => ActionType::Bool,
            HapticFeedback => ActionType::Haptics,
            PrimaryButton => ActionType::Bool,
            PrimaryButtonTouch => ActionType::Bool,
            SecondaryButton => ActionType::Bool,
            SecondaryButtonTouch => ActionType::Bool,
            MenuButton => ActionType::Bool,
            ThumbstickX => ActionType::Float,
            ThumbstickY => ActionType::Float,
            ThumbstickTouch => ActionType::Bool,
            ThumbstickClick => ActionType::Bool,
            ThumbrestTouch => ActionType::Bool,
        }
    }

    pub fn sidedness(&self) -> ActionSidedness {
        use ActionPath::*;
        match self {
            HandPose => ActionSidedness::Double,
            PointerPose => ActionSidedness::Double,
            GripPull => ActionSidedness::Double,
            TriggerPull => ActionSidedness::Double,
            TriggerTouch => ActionSidedness::Double,
            HapticFeedback => ActionSidedness::Double,
            PrimaryButton => ActionSidedness::Double,
            PrimaryButtonTouch => ActionSidedness::Double,
            SecondaryButton => ActionSidedness::Double,
            SecondaryButtonTouch => ActionSidedness::Double,
            MenuButton => ActionSidedness::Double,
            ThumbstickX => ActionSidedness::Double,
            ThumbstickY => ActionSidedness::Double,
            ThumbstickTouch => ActionSidedness::Double,
            ThumbstickClick => ActionSidedness::Double,
            ThumbrestTouch => ActionSidedness::Double,
        }
    }
}

pub enum ActionSidedness {
    Single,
    Double,
}

pub enum Side {
    Left,
    Right,
}
