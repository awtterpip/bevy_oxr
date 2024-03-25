use std::any::TypeId;
use std::marker::PhantomData;

use crate::init::XrPreUpdateSet;
use crate::resources::*;
use crate::types::*;
use bevy::app::{App, Plugin, PreUpdate, Startup};
use bevy::ecs::schedule::common_conditions::resource_added;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::input::InputSystem;
use bevy::log::error;
use bevy::math::{vec2, Vec2};
use bevy::utils::hashbrown::HashMap;
use bevy_xr::actions::ActionPlugin;
use bevy_xr::actions::{Action, ActionList, ActionState};
use bevy_xr::session::session_available;
use bevy_xr::session::session_running;

pub struct XrActionPlugin;

impl Plugin for XrActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_action_sets.run_if(session_available))
            .add_systems(
                PreUpdate,
                sync_actions.run_if(session_running).before(InputSystem),
            )
            .add_systems(
                PreUpdate,
                attach_action_sets
                    .after(XrPreUpdateSet::HandleEvents)
                    .run_if(resource_added::<XrSession>),
            );
    }
}

pub fn create_action_sets(
    instance: Res<XrInstance>,
    action_list: Res<ActionList>,
    mut commands: Commands,
) {
    let (action_set, actions) =
        initialize_action_sets(&instance, &action_list).expect("Failed to initialize action set");

    commands.insert_resource(action_set);
    commands.insert_resource(actions);
}

pub fn attach_action_sets(mut action_set: ResMut<XrActionSet>, session: Res<XrSession>) {
    session
        .attach_action_sets(&[&action_set])
        .expect("Failed to attach action sets");
    action_set.attach();
}

pub fn sync_actions(session: Res<XrSession>, action_set: Res<XrActionSet>) {
    session
        .sync_actions(&[openxr::ActiveActionSet::new(&action_set)])
        .expect("Failed to sync actions");
}

fn initialize_action_sets(
    instance: &XrInstance,
    action_info: &ActionList,
) -> Result<(XrActionSet, XrActions)> {
    let action_set = instance.create_action_set("actions", "actions", 0)?;
    let mut actions = HashMap::new();
    for action_info in action_info.0.iter() {
        use bevy_xr::actions::ActionType::*;
        let action = match action_info.action_type {
            Bool => TypedAction::Bool(action_set.create_action(
                action_info.name,
                action_info.pretty_name,
                &[],
            )?),
            Float => TypedAction::Float(action_set.create_action(
                action_info.name,
                action_info.pretty_name,
                &[],
            )?),
            Vector => TypedAction::Vector(action_set.create_action(
                action_info.name,
                action_info.pretty_name,
                &[],
            )?),
        };
        actions.insert(action_info.type_id, action);
    }
    Ok((XrActionSet::new(action_set), XrActions(actions)))
}

pub struct XrActionUpdatePlugin<A: Action>(PhantomData<A>);

impl<A> Plugin for XrActionUpdatePlugin<A>
where
    A: Action,
    A::ActionType: XrActionTy,
{
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_action_state::<A>.in_set(InputSystem).run_if(session_running));
    }
}

impl<A: Action> Default for XrActionUpdatePlugin<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait XrActionTy: Sized {
    fn get_action_state(
        action: &TypedAction,
        session: &XrSession,
        subaction_path: Option<openxr::Path>,
    ) -> Option<Self>;
}

impl XrActionTy for bool {
    fn get_action_state(
        action: &TypedAction,
        session: &XrSession,
        subaction_path: Option<openxr::Path>,
    ) -> Option<Self> {
        match action {
            TypedAction::Bool(action) => action
                .state(session, subaction_path.unwrap_or(openxr::Path::NULL))
                .ok()
                .map(|state| state.current_state),
            _ => None,
        }
    }
}

impl XrActionTy for f32 {
    fn get_action_state(
        action: &TypedAction,
        session: &XrSession,
        subaction_path: Option<openxr::Path>,
    ) -> Option<Self> {
        match action {
            TypedAction::Float(action) => action
                .state(session, subaction_path.unwrap_or(openxr::Path::NULL))
                .ok()
                .map(|state| state.current_state),
            _ => None,
        }
    }
}

impl XrActionTy for Vec2 {
    fn get_action_state(
        action: &TypedAction,
        session: &XrSession,
        subaction_path: Option<openxr::Path>,
    ) -> Option<Self> {
        match action {
            TypedAction::Vector(action) => action
                .state(session, subaction_path.unwrap_or(openxr::Path::NULL))
                .ok()
                .map(|state| vec2(state.current_state.x, state.current_state.y)),
            _ => None,
        }
    }
}

pub fn update_action_state<A>(
    mut action_state: ResMut<ActionState<A>>,
    session: Res<XrSession>,
    actions: Res<XrActions>,
) where
    A: Action,
    A::ActionType: XrActionTy,
{
    if let Some(action) = actions.get(&TypeId::of::<A>()) {
        if let Some(state) = A::ActionType::get_action_state(action, &session, None) {
            action_state.set(state);
        } else {
            error!(
                "Failed to update value for action '{}'",
                std::any::type_name::<A>()
            );
        }
    }
}

pub trait ActionApp {
    fn register_action<A>(&mut self) -> &mut Self
    where
        A: Action,
        A::ActionType: XrActionTy;
}

impl ActionApp for App {
    fn register_action<A>(&mut self) -> &mut Self
    where
        A: Action,
        A::ActionType: XrActionTy,
    {
        self.add_plugins((
            ActionPlugin::<A>::default(),
            XrActionUpdatePlugin::<A>::default(),
        ))
    }
}
