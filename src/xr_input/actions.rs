use std::error::Error;

use bevy::{prelude::*, utils::HashMap};
use openxr as xr;
use xr::{Action, Binding, Haptic, Posef, Vector2f};

use crate::{
    resources::{XrInstance, XrSession},
    xr_init::XrPrePostSetup,
};

use super::oculus_touch::ActionSets;

pub struct OpenXrActionsPlugin;
impl Plugin for OpenXrActionsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SetupActionSets {
            sets: HashMap::new(),
        });
        app.add_systems(XrPrePostSetup, setup_oxr_actions);
    }
}

#[inline(always)]
fn create_action<T: xr::ActionTy>(
    action: &SetupAction,
    action_name: &'static str,
    oxr_action_set: &xr::ActionSet,
    hands: &[xr::Path],
) -> xr::Action<T> {
    match action.handednes {
        ActionHandednes::Single => oxr_action_set
            .create_action(action_name, &action.pretty_name, &[])
            .unwrap_or_else(|_| panic!("Unable to create action: {}", action_name)),
        ActionHandednes::Double => oxr_action_set
            .create_action(action_name, &action.pretty_name, hands)
            .unwrap_or_else(|_| panic!("Unable to create action: {}", action_name)),
    }
}
pub fn setup_oxr_actions(world: &mut World) {
    let actions = world.remove_resource::<SetupActionSets>().unwrap();
    let instance = world.get_resource::<XrInstance>().unwrap();
    let session = world.get_resource::<XrSession>().unwrap();
    let left_path = instance.string_to_path("/user/hand/left").unwrap();
    let right_path = instance.string_to_path("/user/hand/right").unwrap();
    let hands = [left_path, right_path];

    let mut oxr_action_sets = Vec::new();
    let mut action_sets = XrActionSets { sets: default() };
    // let mut action_bindings: HashMap<&'static str, Vec<xr::Path>> = HashMap::new();
    let mut action_bindings: HashMap<
        (&'static str, &'static str),
        HashMap<&'static str, Vec<xr::Path>>,
    > = HashMap::new();
    for (set_name, set) in actions.sets.into_iter() {
        let mut actions: HashMap<&'static str, TypedAction> = default();
        let oxr_action_set = instance
            .create_action_set(set_name, &set.pretty_name, set.priority)
            .expect("Unable to create action set");
        for (action_name, action) in set.actions.into_iter() {
            use self::create_action as ca;
            let typed_action = match action.action_type {
                ActionType::Vec2 => {
                    TypedAction::Vec2(ca(&action, action_name, &oxr_action_set, &hands))
                }
                ActionType::F32 => {
                    TypedAction::F32(ca(&action, action_name, &oxr_action_set, &hands))
                }
                ActionType::Bool => {
                    TypedAction::Bool(ca(&action, action_name, &oxr_action_set, &hands))
                }
                ActionType::PoseF => {
                    TypedAction::PoseF(ca(&action, action_name, &oxr_action_set, &hands))
                }
                ActionType::Haptic => {
                    TypedAction::Haptic(ca(&action, action_name, &oxr_action_set, &hands))
                }
            };
            actions.insert(action_name, typed_action);
            for (device_path, bindings) in action.bindings.into_iter() {
                for b in bindings {
                    // info!("binding {} to {}", action_name, b);
                    action_bindings
                        .entry((set_name, action_name))
                        .or_default()
                        .entry(device_path)
                        .or_default()
                        .push(instance.string_to_path(b).unwrap());
                }
            }
        }
        oxr_action_sets.push(oxr_action_set);
        action_sets.sets.insert(
            set_name,
            ActionSet {
                // oxr_action_set,
                actions,
                enabled: true,
            },
        );
    }
    let mut b_indings: HashMap<&'static str, Vec<Binding>> = HashMap::new();
    for (dev, mut bindings) in action_sets
        .sets
        .iter()
        .flat_map(|(set_name, set)| {
            set.actions
                .iter()
                .map(move |(action_name, a)| (set_name, action_name, a))
        })
        .zip([&action_bindings].into_iter().cycle())
        .flat_map(move |((set_name, action_name, action), bindings)| {
            bindings
                .get(&(set_name as &'static str, action_name as &'static str))
                .unwrap()
                .iter()
                .map(move |(dev, bindings)| (action, dev, bindings))
        })
        .map(|(action, dev, bindings)| {
            (
                dev,
                bindings
                    .iter()
                    .map(move |binding| match &action {
                        TypedAction::Vec2(a) => Binding::new(a, *binding),
                        TypedAction::F32(a) => Binding::new(a, *binding),
                        TypedAction::Bool(a) => Binding::new(a, *binding),
                        TypedAction::PoseF(a) => Binding::new(a, *binding),
                        TypedAction::Haptic(a) => Binding::new(a, *binding),
                    })
                    .collect::<Vec<_>>(),
            )
        })
    {
        b_indings.entry(dev).or_default().append(&mut bindings);
    }
    for (dev, bindings) in b_indings.into_iter() {
        instance
            .suggest_interaction_profile_bindings(instance.string_to_path(dev).unwrap(), &bindings)
            .expect("Unable to suggest interaction bindings!");
    }
    session
        .attach_action_sets(&oxr_action_sets.iter().collect::<Vec<_>>())
        .expect("Unable to attach action sets!");

    world.insert_resource(ActionSets(oxr_action_sets));
    world.insert_resource(action_sets);
}

pub enum ActionHandednes {
    Single,
    Double,
}

#[derive(Clone, Copy)]
pub enum ActionType {
    F32,
    Bool,
    PoseF,
    Haptic,
    Vec2,
}

pub enum TypedAction {
    F32(Action<f32>),
    Bool(Action<bool>),
    PoseF(Action<Posef>),
    Haptic(Action<Haptic>),
    Vec2(Action<Vector2f>),
}

pub struct SetupAction {
    pretty_name: String,
    action_type: ActionType,
    handednes: ActionHandednes,
    bindings: HashMap<&'static str, Vec<&'static str>>,
}

pub struct SetupActionSet {
    pretty_name: String,
    priority: u32,
    actions: HashMap<&'static str, SetupAction>,
}

impl SetupActionSet {
    pub fn new_action(
        &mut self,
        name: &'static str,
        pretty_name: String,
        action_type: ActionType,
        handednes: ActionHandednes,
    ) {
        self.actions.insert(
            name,
            SetupAction {
                pretty_name,
                action_type,
                handednes,
                bindings: default(),
            },
        );
    }
    pub fn suggest_binding(&mut self, device_path: &'static str, bindings: &[XrBinding]) {
        for binding in bindings {
            self.actions
                .get_mut(binding.action)
                .ok_or(anyhow::anyhow!("Missing Action: {}", binding.action))
                .unwrap()
                .bindings
                .entry(device_path)
                .or_default()
                .push(binding.path);
        }
    }
}
pub struct XrBinding {
    action: &'static str,
    path: &'static str,
}

impl XrBinding {
    pub fn new(action_name: &'static str, binding_path: &'static str) -> XrBinding {
        XrBinding {
            action: action_name,
            path: binding_path,
        }
    }
}

#[derive(Resource)]
pub struct SetupActionSets {
    sets: HashMap<&'static str, SetupActionSet>,
}

impl SetupActionSets {
    pub fn add_action_set(
        &mut self,
        name: &'static str,
        pretty_name: String,
        priority: u32,
    ) -> &mut SetupActionSet {
        self.sets.insert(
            name,
            SetupActionSet {
                pretty_name,
                priority,
                actions: HashMap::new(),
            },
        );
        self.sets.get_mut(name).unwrap()
    }
}

pub struct ActionSet {
    // add functionality to enable/disable action sets
    enabled: bool,
    actions: HashMap<&'static str, TypedAction>,
}

#[derive(Resource)]
pub struct XrActionSets {
    sets: HashMap<&'static str, ActionSet>,
}

use std::fmt::Display as FmtDisplay;
impl FmtDisplay for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            ActionError::NoActionSet => "Action Set Not Found!",
            ActionError::NoAction => "Action Not Found!",
            ActionError::WrongActionType => "Wrong Action Type!",
        };
        write!(f, "{}", err)
    }
}
impl Error for ActionError {}
#[derive(Debug)]
pub enum ActionError {
    NoActionSet,
    NoAction,
    WrongActionType,
}

impl XrActionSets {
    pub fn get_action_vec2(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> Result<&Action<Vector2f>, ActionError> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(ActionError::NoActionSet)?
            .actions
            .get(action_name)
            .ok_or(ActionError::NoAction)?;
        match action {
            TypedAction::Vec2(a) => Ok(a),
            _ => Err(ActionError::WrongActionType),
        }
    }
    pub fn get_action_f32(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> Result<&Action<f32>, ActionError> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(ActionError::NoActionSet)?
            .actions
            .get(action_name)
            .ok_or(ActionError::NoAction)?;
        match action {
            TypedAction::F32(a) => Ok(a),
            _ => Err(ActionError::WrongActionType),
        }
    }
    pub fn get_action_bool(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> Result<&Action<bool>, ActionError> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(ActionError::NoActionSet)?
            .actions
            .get(action_name)
            .ok_or(ActionError::NoAction)?;
        match action {
            TypedAction::Bool(a) => Ok(a),
            _ => Err(ActionError::WrongActionType),
        }
    }
    pub fn get_action_posef(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> Result<&Action<Posef>, ActionError> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(ActionError::NoActionSet)?
            .actions
            .get(action_name)
            .ok_or(ActionError::NoAction)?;
        match action {
            TypedAction::PoseF(a) => Ok(a),
            _ => Err(ActionError::WrongActionType),
        }
    }
    pub fn get_action_haptic(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> Result<&Action<Haptic>, ActionError> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(ActionError::NoActionSet)?
            .actions
            .get(action_name)
            .ok_or(ActionError::NoAction)?;
        match action {
            TypedAction::Haptic(a) => Ok(a),
            _ => Err(ActionError::WrongActionType),
        }
    }
}
