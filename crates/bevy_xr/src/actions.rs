use std::marker::PhantomData;

use bevy::{
    app::{App, First, Plugin},
    ecs::system::{ResMut, Resource},
};

pub struct ActionPlugin<A: Action>(PhantomData<A>);

impl<A: Action> Default for ActionPlugin<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<A: Action> Plugin for ActionPlugin<A> {
    fn build(&self, app: &mut App) {
        app.add_systems(First, reset_action_state::<A>)
            .init_resource::<Actions>();
        app.world.resource_mut::<Actions>().0.push(A::INFO);
    }
}

pub enum ActionType {
    Bool,
}

pub trait ActionTy: Send + Sync + Default + Clone + Copy {
    const TYPE: ActionType;
}

impl ActionTy for bool {
    const TYPE: ActionType = ActionType::Bool;
}

pub trait Action: Send + Sync + 'static {
    type ActionType: ActionTy;

    const INFO: ActionInfo;
}

pub struct ActionInfo {
    pub pretty_name: &'static str,
    pub name: &'static str,
    pub action_type: ActionType,
}

#[derive(Resource, Default)]
pub struct Actions(pub Vec<ActionInfo>);

#[derive(Resource, Default)]
pub struct ActionState<A: Action> {
    previous_state: A::ActionType,
    current_state: A::ActionType,
}

impl<A: Action> ActionState<A> {
    pub fn current_state(&self) -> A::ActionType {
        self.current_state
    }

    pub fn previous_state(&self) -> A::ActionType {
        self.previous_state
    }
}

impl<A: Action<ActionType = bool>> ActionState<A> {
    pub fn pressed(&self) -> bool {
        self.current_state
    }

    pub fn just_pressed(&self) -> bool {
        self.previous_state == false && self.current_state == true
    }

    pub fn just_released(&self) -> bool {
        self.previous_state == true && self.current_state == false
    }

    pub fn press(&mut self) {
        self.current_state = true
    }
}

pub fn reset_action_state<A: Action>(mut action_state: ResMut<ActionState<A>>) {
    action_state.previous_state = std::mem::take(&mut action_state.current_state);
}

pub trait ActionApp {
    fn register_action<A: Action>(&mut self) -> &mut Self;
}

impl ActionApp for App {
    fn register_action<A: Action>(&mut self) -> &mut Self {
        self.add_plugins(ActionPlugin::<A>::default())
    }
}
