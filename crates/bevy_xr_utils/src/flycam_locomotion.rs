use bevy::{math::vec3, prelude::*};
use bevy_openxr::{helper_traits::ToQuat, init::OxrTrackingRoot, resources::OxrViews};

use crate::xr_utils_actions::{
    ActiveSet, XRUtilsAction, XRUtilsActionSet, XRUtilsActionState, XRUtilsActionSystemSet,
    XRUtilsBinding,
};

pub struct FlycamLocomotionPlugin;
impl Plugin for FlycamLocomotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TranslationInputVector>();
        app.add_systems(Update, fly_cam_locomotion);
        app.add_systems(
            PreUpdate,
            fly_cam_input.after(XRUtilsActionSystemSet::SyncActionStates),
        );
        app.add_systems(
            Startup,
            spawn_fly_cam_actions.before(XRUtilsActionSystemSet::CreateEvents),
        );
    }
}

//this is the users input vector for translation
#[derive(Event, Debug)]
pub struct TranslationInputVector(Vec3);

pub fn fly_cam_locomotion(
    mut root_query: Query<&mut Transform, With<OxrTrackingRoot>>,
    mut translation_input_reader: EventReader<TranslationInputVector>,
    time: Res<Time>,
    config_option: Option<Res<FlyCamConfig>>,
    views: ResMut<OxrViews>,
) {
    let config = match config_option {
        Some(c) => c,
        None => {
            info!("no locomotion config");
            return;
        }
    };

    let mut root_transform = root_query
        .get_single_mut()
        .expect("there was an error getting the single root");

    let first_view = views.first();
    let view = match first_view {
        Some(view) => view,
        None => {
            info!("no views");
            return;
        }
    };

    let reference_quat = match config.locomotion_reference {
        LocomotionReference::Root => root_transform.rotation.clone(),
        LocomotionReference::HMD => view.pose.orientation.to_quat(),
        LocomotionReference::Controller => todo!(),
    };

    let speed = config.locomotion_speed;
    for translation_input in translation_input_reader.read() {
        //calculate locomotion vector with reference quat
        let locomotion_vec = reference_quat.mul_vec3(translation_input.0);
        root_transform.translation += locomotion_vec * speed * time.delta_seconds();
    }
}

#[derive(Component)]
pub struct TranslationInputAction;

#[derive(Component)]
pub struct ControllerPoseAction;

pub fn spawn_fly_cam_actions(mut commands: Commands) {
    //create a set
    let set = commands
        .spawn((
            XRUtilsActionSet {
                name: "locomotion".into(),
                pretty_name: "locomotion set".into(),
                priority: u32::MIN,
            },
            ActiveSet, //marker to indicate we want this synced
        ))
        .id();
    //create an action
    let action = commands
        .spawn((
            XRUtilsAction {
                action_name: "translation_input".into(),
                localized_name: "translation_input_localized".into(),
                action_type: bevy_xr::actions::ActionType::Vector,
            },
            TranslationInputAction, //lets try a marker component
        ))
        .id();

    //create a binding
    let binding = commands
        .spawn(XRUtilsBinding {
            profile: "/interaction_profiles/valve/index_controller".into(),
            binding: "/user/hand/left/input/thumbstick".into(),
        })
        .id();

    //add action to set, this isnt the best
    //TODO look into a better system
    commands.entity(action).add_child(binding);
    commands.entity(set).add_child(action);
}

pub fn fly_cam_input(
    mut action_query: Query<&XRUtilsActionState, With<TranslationInputAction>>,
    mut translation_input_writer: EventWriter<TranslationInputVector>,
) {
    for state in action_query.iter_mut() {
        info!("{:?}", state);
        match state {
            XRUtilsActionState::Bool(_) => (),
            XRUtilsActionState::Float(_) => (),
            XRUtilsActionState::Vector(state) => {
                let input = state.current_state;
                let input_vector = vec3(input[0], 0.0, -input[1]); //x is negative i guess?
                let input_event = TranslationInputVector(input_vector);
                translation_input_writer.send(input_event);
            }
        }
    }
}

pub enum LocomotionReference {
    Root,
    HMD,
    Controller,
}

#[derive(Resource)]
pub struct FlyCamConfig {
    pub locomotion_reference: LocomotionReference,
    pub locomotion_speed: f32,
}

impl Default for FlyCamConfig {
    fn default() -> Self {
        Self {
            locomotion_reference: LocomotionReference::HMD,
            locomotion_speed: 1.0,
        }
    }
}
