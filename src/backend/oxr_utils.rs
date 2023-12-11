use crate::{
    error::XrError,
    types::{ActionPath, ActionSidedness, ActionType, ControllerType, Side},
};
use bevy::utils::HashMap;
use openxr::{Action, ActionSet, Binding, Entry, Haptic, Path, Posef};

#[cfg(feature = "linked")]
pub fn xr_entry() -> Result<openxr::Entry, XrError> {
    Ok(Entry::linked())
}

#[cfg(not(feature = "linked"))]
pub fn xr_entry() -> Result<openxr::Entry, XrError> {
    unsafe { Entry::load().map_err(|_| XrError {}) }
}

pub fn create_actions(
    action_set: &ActionSet,
    actions: &[ActionPath],
    hand_subpaths: &[Path],
) -> Result<HashMap<ActionPath, TypedAction>, XrError> {
    let mut action_map = HashMap::new();
    for action in actions {
        let subaction_paths = match action.sidedness() {
            ActionSidedness::Single => &[],
            ActionSidedness::Double => hand_subpaths,
        };
        let name = action.action_name();
        let localized_name = action.pretty_action_name();
        let typed_action = match action.action_type() {
            ActionType::Bool => TypedAction::Bool(action_set.create_action(
                name,
                localized_name,
                subaction_paths,
            )?),
            ActionType::Float => {
                TypedAction::F32(action_set.create_action(name, localized_name, subaction_paths)?)
            }
            ActionType::Haptics => TypedAction::Haptic(action_set.create_action(
                name,
                localized_name,
                subaction_paths,
            )?),
            ActionType::Pose => TypedAction::PoseF(action_set.create_action(
                name,
                localized_name,
                subaction_paths,
            )?),
        };

        action_map.insert(*action, typed_action);
    }
    Ok(action_map)
}

pub enum TypedAction {
    F32(Action<f32>),
    Bool(Action<bool>),
    PoseF(Action<Posef>),
    Haptic(Action<Haptic>),
}

impl TypedAction {
    fn make_binding(&self, name: Path) -> Binding {
        match self {
            TypedAction::F32(a) => Binding::new(a, name),
            TypedAction::Bool(a) => Binding::new(a, name),
            TypedAction::PoseF(a) => Binding::new(a, name),
            TypedAction::Haptic(a) => Binding::new(a, name),
        }
    }
}

impl ControllerType {
    pub(crate) fn set_controller_bindings(
        &self,
        instance: &openxr::Instance,
        bindings: &HashMap<ActionPath, TypedAction>,
    ) -> Result<(), XrError> {
        match self {
            ControllerType::OculusTouch => {
                instance.suggest_interaction_profile_bindings(
                    instance.string_to_path("/interaction_profiles/oculus/touch_controller")?,
                    &[],
                )?;
            }
        };
        Ok(())
    }

    fn get_binding_paths(&self, path: ActionPath) -> &[&'static str] {
        match self {
            ControllerType::OculusTouch => match path {
                ActionPath::HandPose => &[
                    "/user/hand/left/input/grip/pose",
                    "/user/hand/right/input/grip/pose",
                ],
                ActionPath::PointerPose => &[
                    "/user/hand/left/input/aim/pose",
                    "/user/hand/right/input/aim/pose",
                ],
                ActionPath::GripPull => &[
                    "/user/hand/left/input/squeeze/value",
                    "/user/hand/right/input/squeeze/value",
                ],
                ActionPath::TriggerPull => &[
                    "/user/hand/left/input/trigger/value",
                    "/user/hand/right/input/trigger/value",
                ],
                ActionPath::TriggerTouch => &[
                    "/user/hand/left/input/trigger/touch",
                    "/user/hand/right/input/trigger/touch",
                ],
                ActionPath::HapticFeedback => &[
                    "/user/hand/left/output/haptic",
                    "/user/hand/right/output/haptic",
                ],
                ActionPath::PrimaryButton => &[],
                ActionPath::PrimaryButtonTouch => todo!(),
                ActionPath::SecondaryButton => todo!(),
                ActionPath::SecondaryButtonTouch => todo!(),
                ActionPath::MenuButton => todo!(),
                ActionPath::ThumbstickX => todo!(),
                ActionPath::ThumbstickY => todo!(),
                ActionPath::ThumbstickTouch => todo!(),
                ActionPath::ThumbstickClick => todo!(),
                ActionPath::ThumbrestTouch => todo!(),
            },
        }
    }
}
