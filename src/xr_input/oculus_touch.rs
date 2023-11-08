use crate::input::XrInput;
use crate::resources::{XrInstance, XrSession};
use crate::xr_input::controllers::{Handed, Touchable};
use crate::xr_input::Hand;
use bevy::prelude::{Commands, Res, Resource};
use openxr::{
    Action, ActionSet, AnyGraphics, Binding, FrameState, Haptic, Instance, Path, Posef, Session,
    Space, SpaceLocation, SpaceVelocity,
};

use std::sync::OnceLock;

pub fn setup_oculus_controller(
    mut commands: Commands,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
) {
    let mut action_sets = vec![];
    let oculus_controller = OculusController::new(
        Instance::clone(&instance),
        Session::clone(&session),
        &mut action_sets,
    )
    .unwrap();
    session
        .attach_action_sets(&action_sets.iter().collect::<Vec<_>>())
        .unwrap();
    commands.insert_resource(oculus_controller);
    commands.insert_resource(ActionSets(action_sets));
}

#[derive(Resource, Clone)]
pub struct ActionSets(pub Vec<ActionSet>);

#[allow(dead_code)]
pub struct OculusControllerRef<'a> {
    oculus_controller: &'a OculusController,
    instance: &'a Instance,
    session: &'a Session<AnyGraphics>,
    frame_state: &'a FrameState,
    xr_input: &'a XrInput,
}

static RIGHT_SUBACTION_PATH: OnceLock<Path> = OnceLock::new();
static LEFT_SUBACTION_PATH: OnceLock<Path> = OnceLock::new();

pub fn init_subaction_path(instance: &Instance) {
    let _ = LEFT_SUBACTION_PATH.set(instance.string_to_path("/user/hand/left").unwrap());
    let _ = RIGHT_SUBACTION_PATH.set(instance.string_to_path("/user/hand/right").unwrap());
}

pub fn subaction_path(hand: Hand) -> Path {
    *match hand {
        Hand::Left => LEFT_SUBACTION_PATH.get().unwrap(),
        Hand::Right => RIGHT_SUBACTION_PATH.get().unwrap(),
    }
}

impl OculusControllerRef<'_> {
    pub fn grip_space(&self, hand: Hand) -> (SpaceLocation, SpaceVelocity) {
        match hand {
            Hand::Left => self.oculus_controller.grip_space.left.relate(
                &self.xr_input.stage,
                self.frame_state.predicted_display_time,
            ),
            Hand::Right => self.oculus_controller.grip_space.right.relate(
                &self.xr_input.stage,
                self.frame_state.predicted_display_time,
            ),
        }
        .unwrap()
    }
    pub fn aim_space(&self, hand: Hand) -> (SpaceLocation, SpaceVelocity) {
        match hand {
            Hand::Left => self.oculus_controller.aim_space.left.relate(
                &self.xr_input.stage,
                self.frame_state.predicted_display_time,
            ),
            Hand::Right => self.oculus_controller.aim_space.right.relate(
                &self.xr_input.stage,
                self.frame_state.predicted_display_time,
            ),
        }
        .unwrap()
    }
    pub fn squeeze(&self, hand: Hand) -> f32 {
        let action = &self.oculus_controller.squeeze;
        action
            .state(self.session, subaction_path(hand))
            .unwrap()
            .current_state
    }
    pub fn trigger(&self, hand: Hand) -> f32 {
        self.oculus_controller
            .trigger
            .inner
            .state(self.session, subaction_path(hand))
            .unwrap()
            .current_state
    }
    pub fn trigger_touched(&self, hand: Hand) -> bool {
        self.oculus_controller
            .trigger
            .touch
            .state(self.session, subaction_path(hand))
            .unwrap()
            .current_state
    }
    pub fn x_button(&self) -> bool {
        self.oculus_controller
            .x_button
            .inner
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn x_button_touched(&self) -> bool {
        self.oculus_controller
            .x_button
            .touch
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn y_button(&self) -> bool {
        self.oculus_controller
            .y_button
            .inner
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn y_button_touched(&self) -> bool {
        self.oculus_controller
            .y_button
            .touch
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn menu_button(&self) -> bool {
        self.oculus_controller
            .menu_button
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn a_button(&self) -> bool {
        self.oculus_controller
            .a_button
            .inner
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn a_button_touched(&self) -> bool {
        self.oculus_controller
            .a_button
            .touch
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn b_button(&self) -> bool {
        self.oculus_controller
            .b_button
            .inner
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn b_button_touched(&self) -> bool {
        self.oculus_controller
            .b_button
            .touch
            .state(self.session, Path::NULL)
            .unwrap()
            .current_state
    }
    pub fn thumbstick_touch(&self, hand: Hand) -> bool {
        self.oculus_controller
            .thumbstick_touch
            .state(self.session, subaction_path(hand))
            .unwrap()
            .current_state
    }
    pub fn thumbstick(&self, hand: Hand) -> Thumbstick {
        Thumbstick {
            x: self
                .oculus_controller
                .thumbstick_x
                .state(self.session, subaction_path(hand))
                .unwrap()
                .current_state,
            y: self
                .oculus_controller
                .thumbstick_y
                .state(self.session, subaction_path(hand))
                .unwrap()
                .current_state,
            click: self
                .oculus_controller
                .thumbstick_click
                .state(self.session, subaction_path(hand))
                .unwrap()
                .current_state,
        }
    }
    pub fn thumbrest_touch(&self, hand: Hand) -> bool {
        self.oculus_controller
            .thumbrest_touch
            .state(self.session, subaction_path(hand))
            .unwrap()
            .current_state
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Thumbstick {
    pub x: f32,
    pub y: f32,
    pub click: bool,
}

impl OculusController {
    pub fn get_ref<'a>(
        &'a self,
        instance: &'a Instance,
        session: &'a Session<AnyGraphics>,
        frame_state: &'a FrameState,
        xr_input: &'a XrInput,
    ) -> OculusControllerRef {
        OculusControllerRef {
            oculus_controller: self,
            instance,
            session,
            frame_state,
            xr_input,
        }
    }
}

#[derive(Resource)]
pub struct OculusController {
    pub grip_space: Handed<Space>,
    pub aim_space: Handed<Space>,
    pub grip_pose: Action<Posef>,
    pub aim_pose: Action<Posef>,
    pub squeeze: Action<f32>,
    pub trigger: Touchable<f32>,
    pub haptic_feedback: Action<Haptic>,
    pub x_button: Touchable<bool>,
    pub y_button: Touchable<bool>,
    pub menu_button: Action<bool>,
    pub a_button: Touchable<bool>,
    pub b_button: Touchable<bool>,
    pub thumbstick_x: Action<f32>,
    pub thumbstick_y: Action<f32>,
    pub thumbstick_touch: Action<bool>,
    pub thumbstick_click: Action<bool>,
    pub thumbrest_touch: Action<bool>,
}
impl OculusController {
    pub fn new(
        instance: Instance,
        session: Session<AnyGraphics>,
        action_sets: &mut Vec<ActionSet>,
    ) -> anyhow::Result<Self> {
        let action_set =
            instance.create_action_set("oculus_input", "Oculus Touch Controller Input", 0)?;
        init_subaction_path(&instance);
        let left_path = instance.string_to_path("/user/hand/left").unwrap();
        let right_path = instance.string_to_path("/user/hand/right").unwrap();
        let hands = [left_path, right_path];
        let grip_pose = action_set.create_action::<Posef>("hand_pose", "Hand Pose", &hands)?;
        let aim_pose = action_set.create_action::<Posef>("pointer_pose", "Pointer Pose", &hands)?;

        let this = OculusController {
            grip_space: Handed {
                left: grip_pose.create_space(session.clone(), left_path, Posef::IDENTITY)?,
                right: grip_pose.create_space(session.clone(), right_path, Posef::IDENTITY)?,
            },
            aim_space: Handed {
                left: aim_pose.create_space(session.clone(), left_path, Posef::IDENTITY)?,
                right: aim_pose.create_space(session, right_path, Posef::IDENTITY)?,
            },
            grip_pose,
            aim_pose,
            squeeze: action_set.create_action("squeeze", "Grip Pull", &hands)?,
            trigger: Touchable {
                inner: action_set.create_action("trigger", "Trigger Pull", &hands)?,
                touch: action_set.create_action("trigger_touched", "Trigger Touch", &hands)?,
            },
            haptic_feedback: action_set.create_action(
                "haptic_feedback",
                "Haptic Feedback",
                &hands,
            )?,
            x_button: Touchable {
                inner: action_set.create_action("x_button", "X Button", &[])?,
                touch: action_set.create_action("x_button_touch", "X Button Touch", &[])?,
            },
            y_button: Touchable {
                inner: action_set.create_action("y_button", "Y Button", &[])?,
                touch: action_set.create_action("y_button_touch", "Y Button Touch", &[])?,
            },
            menu_button: action_set.create_action("menu_button", "Menu Button", &[])?,
            a_button: Touchable {
                inner: action_set.create_action("a_button", "A Button", &[])?,
                touch: action_set.create_action("a_button_touch", "A Button Touch", &[])?,
            },
            b_button: Touchable {
                inner: action_set.create_action("b_button", "B Button", &[])?,
                touch: action_set.create_action("b_button_touch", "B Button Touch", &[])?,
            },
            thumbstick_x: action_set.create_action("thumbstick_x", "Thumbstick X", &hands)?,
            thumbstick_y: action_set.create_action("thumbstick_y", "Thumbstick Y", &hands)?,
            thumbstick_touch: action_set.create_action(
                "thumbstick_touch",
                "Thumbstick Touch",
                &hands,
            )?,
            thumbstick_click: action_set.create_action(
                "thumbstick_click",
                "Thumbstick Click",
                &hands,
            )?,
            thumbrest_touch: action_set.create_action(
                "thumbrest_touch",
                "Thumbrest Touch",
                &hands,
            )?,
        };
        let i = instance;
        i.suggest_interaction_profile_bindings(
            i.string_to_path("/interaction_profiles/oculus/touch_controller")?,
            &[
                Binding::new(
                    &this.grip_pose,
                    i.string_to_path("/user/hand/left/input/grip/pose")?,
                ),
                Binding::new(
                    &this.grip_pose,
                    i.string_to_path("/user/hand/right/input/grip/pose")?,
                ),
                Binding::new(
                    &this.aim_pose,
                    i.string_to_path("/user/hand/left/input/aim/pose")?,
                ),
                Binding::new(
                    &this.aim_pose,
                    i.string_to_path("/user/hand/left/input/aim/pose")?,
                ),
                Binding::new(
                    &this.squeeze,
                    i.string_to_path("/user/hand/left/input/squeeze/value")?,
                ),
                Binding::new(
                    &this.squeeze,
                    i.string_to_path("/user/hand/right/input/squeeze/value")?,
                ),
                Binding::new(
                    &this.trigger.inner,
                    i.string_to_path("/user/hand/right/input/trigger/value")?,
                ),
                Binding::new(
                    &this.trigger.inner,
                    i.string_to_path("/user/hand/left/input/trigger/value")?,
                ),
                Binding::new(
                    &this.trigger.touch,
                    i.string_to_path("/user/hand/right/input/trigger/touch")?,
                ),
                Binding::new(
                    &this.trigger.touch,
                    i.string_to_path("/user/hand/left/input/trigger/touch")?,
                ),
                Binding::new(
                    &this.haptic_feedback,
                    i.string_to_path("/user/hand/right/output/haptic")?,
                ),
                Binding::new(
                    &this.haptic_feedback,
                    i.string_to_path("/user/hand/left/output/haptic")?,
                ),
                Binding::new(
                    &this.x_button.inner,
                    i.string_to_path("/user/hand/left/input/x/click")?,
                ),
                Binding::new(
                    &this.x_button.touch,
                    i.string_to_path("/user/hand/left/input/x/touch")?,
                ),
                Binding::new(
                    &this.y_button.inner,
                    i.string_to_path("/user/hand/left/input/y/click")?,
                ),
                Binding::new(
                    &this.y_button.touch,
                    i.string_to_path("/user/hand/left/input/y/touch")?,
                ),
                Binding::new(
                    &this.menu_button,
                    i.string_to_path("/user/hand/left/input/menu/click")?,
                ),
                Binding::new(
                    &this.a_button.inner,
                    i.string_to_path("/user/hand/right/input/a/click")?,
                ),
                Binding::new(
                    &this.a_button.touch,
                    i.string_to_path("/user/hand/right/input/a/touch")?,
                ),
                Binding::new(
                    &this.b_button.inner,
                    i.string_to_path("/user/hand/right/input/b/click")?,
                ),
                Binding::new(
                    &this.b_button.touch,
                    i.string_to_path("/user/hand/right/input/b/touch")?,
                ),
                Binding::new(
                    &this.thumbstick_x,
                    i.string_to_path("/user/hand/left/input/thumbstick/x")?,
                ),
                Binding::new(
                    &this.thumbstick_x,
                    i.string_to_path("/user/hand/right/input/thumbstick/x")?,
                ),
                Binding::new(
                    &this.thumbstick_y,
                    i.string_to_path("/user/hand/left/input/thumbstick/y")?,
                ),
                Binding::new(
                    &this.thumbstick_y,
                    i.string_to_path("/user/hand/right/input/thumbstick/y")?,
                ),
                Binding::new(
                    &this.thumbstick_click,
                    i.string_to_path("/user/hand/left/input/thumbstick/click")?,
                ),
                Binding::new(
                    &this.thumbstick_click,
                    i.string_to_path("/user/hand/right/input/thumbstick/click")?,
                ),
                Binding::new(
                    &this.thumbstick_touch,
                    i.string_to_path("/user/hand/left/input/thumbstick/touch")?,
                ),
                Binding::new(
                    &this.thumbstick_touch,
                    i.string_to_path("/user/hand/right/input/thumbstick/touch")?,
                ),
                Binding::new(
                    &this.thumbrest_touch,
                    i.string_to_path("/user/hand/left/input/thumbrest/touch")?,
                ),
                Binding::new(
                    &this.thumbrest_touch,
                    i.string_to_path("/user/hand/right/input/thumbrest/touch")?,
                ),
            ],
        )?;

        action_sets.push(action_set);
        Ok(this)
    }
}
