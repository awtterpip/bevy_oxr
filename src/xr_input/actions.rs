use bevy::{prelude::*, utils::HashMap};
use openxr as xr;
use xr::{Action, Binding, Haptic, Posef};

use crate::{resources::{XrInstance, XrSession}, xr_init::XrPrePostSetup};

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
    let mut a_iter = actions.sets.into_iter();
    let mut action_bindings: HashMap<
        (&'static str, &'static str),
        HashMap<&'static str, Vec<xr::Path>>,
    > = HashMap::new();
    while let Some((set_name, set)) = a_iter.next() {
        let mut actions: HashMap<&'static str, TypedAction> = default();
        let oxr_action_set = instance
            .create_action_set(set_name, &set.pretty_name, set.priority)
            .expect("Unable to create action set");
        for (action_name, action) in set.actions.into_iter() {
            let typed_action = match action.action_type {
                ActionType::F32 => TypedAction::F32(match action.handednes {
                    ActionHandednes::Single => oxr_action_set
                        .create_action(action_name, &action.pretty_name, &[])
                        .expect(&format!("Unable to create action: {}", action_name)),
                    ActionHandednes::Double => oxr_action_set
                        .create_action(action_name, &action.pretty_name, &hands)
                        .expect(&format!("Unable to create action: {}", action_name)),
                }),
                ActionType::Bool => TypedAction::Bool(match action.handednes {
                    ActionHandednes::Single => oxr_action_set
                        .create_action(action_name, &action.pretty_name, &[])
                        .expect(&format!("Unable to create action: {}", action_name)),
                    ActionHandednes::Double => oxr_action_set
                        .create_action(action_name, &action.pretty_name, &hands)
                        .expect(&format!("Unable to create action: {}", action_name)),
                }),
                ActionType::PoseF => TypedAction::PoseF(match action.handednes {
                    ActionHandednes::Single => oxr_action_set
                        .create_action(action_name, &action.pretty_name, &[])
                        .expect(&format!("Unable to create action: {}", action_name)),
                    ActionHandednes::Double => oxr_action_set
                        .create_action(action_name, &action.pretty_name, &hands)
                        .expect(&format!("Unable to create action: {}", action_name)),
                }),
                ActionType::Haptic => TypedAction::Haptic(match action.handednes {
                    ActionHandednes::Single => oxr_action_set
                        .create_action(action_name, &action.pretty_name, &[])
                        .expect(&format!("Unable to create action: {}", action_name)),
                    ActionHandednes::Double => oxr_action_set
                        .create_action(action_name, &action.pretty_name, &hands)
                        .expect(&format!("Unable to create action: {}", action_name)),
                }),
            };
            actions.insert(action_name, typed_action);
            for (device_path, bindings) in action.bindings.into_iter() {
                for b in bindings {
                    info!("binding {} to {}", action_name, b);
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
                .get(&(set_name.clone(), action_name.clone()))
                .unwrap()
                .iter()
                .map(move |(dev, bindings)| (action.clone(), dev.clone(), bindings))
        })
        .map(|(action, dev, bindings)| {
            info!("Hi");
            (
                dev,
                bindings
                    .into_iter()
                    .map(move |binding| match &action {
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
        info!(dev);
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
}

pub enum TypedAction {
    F32(Action<f32>),
    Bool(Action<bool>),
    PoseF(Action<Posef>),
    Haptic(Action<Haptic>),
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
    // oxr_action_set: xr::ActionSet,
    enabled: bool,
    actions: HashMap<&'static str, TypedAction>,
}

#[derive(Resource)]
pub struct XrActionSets {
    sets: HashMap<&'static str, ActionSet>,
}

impl XrActionSets {
    pub fn get_action_f32(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> anyhow::Result<&Action<f32>> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(anyhow::anyhow!("Action Set Not Found!"))?
            .actions
            .get(action_name)
            .ok_or(anyhow::anyhow!("Action Not Found!"))?;
        match action {
            TypedAction::F32(a) => Ok(a),
            _ => anyhow::bail!("wrong action type"),
        }
    }
    pub fn get_action_bool(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> anyhow::Result<&Action<bool>> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(anyhow::anyhow!("Action Set Not Found!"))?
            .actions
            .get(action_name)
            .ok_or(anyhow::anyhow!("Action Not Found!"))?;
        match action {
            TypedAction::Bool(a) => Ok(a),
            _ => anyhow::bail!("wrong action type"),
        }
    }
    pub fn get_action_posef(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> anyhow::Result<&Action<Posef>> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(anyhow::anyhow!("Action Set Not Found!"))?
            .actions
            .get(action_name)
            .ok_or(anyhow::anyhow!("Action Not Found!"))?;
        match action {
            TypedAction::PoseF(a) => Ok(a),
            _ => anyhow::bail!("wrong action type"),
        }
    }
    pub fn get_action_haptic(
        &self,
        action_set: &'static str,
        action_name: &'static str,
    ) -> anyhow::Result<&Action<Haptic>> {
        let action = self
            .sets
            .get(action_set)
            .ok_or(anyhow::anyhow!("Action Set Not Found!"))?
            .actions
            .get(action_name)
            .ok_or(anyhow::anyhow!("Action Not Found!"))?;
        match action {
            TypedAction::Haptic(a) => Ok(a),
            _ => anyhow::bail!("wrong action type"),
        }
    }
}
