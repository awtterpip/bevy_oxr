use bevy::{prelude::*, utils::HashMap};
use openxr as xr;
use xr::{Action, Binding, Posef};

use crate::resources::XrInstance;

pub fn setup_oxr_actions(world: &mut World, instance: Ref<XrInstance>) {
    let left_path = instance.string_to_path("/user/hand/left").unwrap();
    let right_path = instance.string_to_path("/user/hand/right").unwrap();
    let hands = [left_path, right_path];

    let actions = world.remove_resource::<SetupActionSets>().unwrap();
    let mut action_sets = ActionSets { sets: default() };
    let mut action_bindings: HashMap<&'static str, Vec<xr::Path>> = HashMap::new();
    let mut a_iter = actions.sets.into_iter();
    while let Some((set_name, set)) = a_iter.next() {
        let mut actions: HashMap<&'static str, TypedAction> = default();
        let oxr_action_set = instance
            .create_action_set(set_name, set.pretty_name, set.priority)
            .unwrap();
        for (action_name, action) in set.actions.into_iter() {
            let typed_action = match action.action_type {
                ActionType::F32 => TypedAction::F32(match action.handednes {
                    ActionHandednes::Single => oxr_action_set
                        .create_action(action_name, action.pretty_name, &[])
                        .unwrap(),
                    ActionHandednes::Double => oxr_action_set
                        .create_action(action_name, action.pretty_name, &hands)
                        .unwrap(),
                }),
                ActionType::Bool => TypedAction::Bool(match action.handednes {
                    ActionHandednes::Single => oxr_action_set
                        .create_action(action_name, action.pretty_name, &[])
                        .unwrap(),
                    ActionHandednes::Double => oxr_action_set
                        .create_action(action_name, action.pretty_name, &hands)
                        .unwrap(),
                }),
                ActionType::PoseF => TypedAction::PoseF(match action.handednes {
                    ActionHandednes::Single => oxr_action_set
                        .create_action(action_name, action.pretty_name, &[])
                        .unwrap(),
                    ActionHandednes::Double => oxr_action_set
                        .create_action(action_name, action.pretty_name, &hands)
                        .unwrap(),
                }),
            };
            actions.insert(action_name, typed_action);
            for (device_path, bindings) in action.bindings.into_iter() {
                for b in bindings {
                    action_bindings
                        .entry(device_path)
                        .or_default()
                        .push(instance.string_to_path(b).unwrap());
                }
            }
        }
        action_sets.sets.insert(
            set_name,
            ActionSet {
                oxr_action_set,
                actions,
            },
        );
    }
    for (dev, bindings) in action_sets
        .sets
        .iter()
        .flat_map(|(_, set)| set.actions.iter().map(|(_, a)| a))
        .zip(action_bindings.into_iter())
        .map(|(action, (dev, bindings))| {
            (
                dev,
                bindings
                    .into_iter()
                    .map(move |binding| match &action {
                        TypedAction::F32(a) => Binding::new(a, binding),
                        TypedAction::Bool(a) => Binding::new(a, binding),
                        TypedAction::PoseF(a) => Binding::new(a, binding),
                    })
                    .collect::<Vec<_>>(),
            )
        })
    {
        instance
            .suggest_interaction_profile_bindings(instance.string_to_path(dev).unwrap(), &bindings)
            .unwrap();
    }
}

pub enum ActionHandednes {
    Single,
    Double,
}

pub enum ActionType {
    F32,
    Bool,
    PoseF,
}

pub enum TypedAction {
    F32(Action<f32>),
    Bool(Action<bool>),
    PoseF(Action<Posef>),
}

pub struct SetupAction {
    pretty_name: &'static str,
    action_type: ActionType,
    handednes: ActionHandednes,
    bindings: HashMap<&'static str, Vec<&'static str>>,
}

pub struct SetupActionSet {
    pretty_name: &'static str,
    priority: u32,
    actions: HashMap<&'static str, SetupAction>,
}

impl SetupActionSet {
    pub fn new_action(
        &mut self,
        name: &'static str,
        pretty_name: &'static str,
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
    pub fn suggest_binding(
        &mut self,
        action_name: &'static str,
        device_path: &'static str,
        action_path: &'static str,
    ) {
        self.actions
            .get_mut(action_name)
            .unwrap()
            .bindings
            .entry(device_path)
            .or_default()
            .push(action_path);
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
        pretty_name: &'static str,
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
    oxr_action_set: xr::ActionSet,
    actions: HashMap<&'static str, TypedAction>,
}

#[derive(Resource)]
pub struct ActionSets {
    sets: HashMap<&'static str, ActionSet>,
}
