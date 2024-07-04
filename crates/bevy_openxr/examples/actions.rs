// a simple example showing basic actions using the xr utils actions
use bevy::{math::vec3, prelude::*};
use bevy_openxr::{add_xr_plugins, helper_traits::ToQuat, resources::OxrViews};
use bevy_xr::session::XrTrackingRoot;
use bevy_xr_utils::xr_utils_actions::{
    ActiveSet, XRUtilsAction, XRUtilsActionSet, XRUtilsActionState, XRUtilsActionSystemSet,
    XRUtilsActionsPlugin, XRUtilsBinding,
};

fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins(bevy_xr_utils::hand_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Startup,
            create_action_entities.before(XRUtilsActionSystemSet::CreateEvents),
        )
        .add_plugins(XRUtilsActionsPlugin)
        .add_systems(Update, read_action_with_marker_component)
        .add_systems(Update, handle_flight_input)
        // Realtime lighting is expensive, use ambient light instead
        .insert_resource(AmbientLight {
            color: Default::default(),
            brightness: 500.0,
        })
        .run();
}

/// set up a simple 3D scene
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Component)]
struct FlightActionMarker;

fn create_action_entities(mut commands: Commands) {
    //create a set
    let set = commands
        .spawn((
            XRUtilsActionSet {
                name: "flight".into(),
                pretty_name: "pretty flight set".into(),
                priority: u32::MIN,
            },
            ActiveSet, //marker to indicate we want this synced
        ))
        .id();
    //create an action
    let action = commands
        .spawn((
            XRUtilsAction {
                action_name: "flight_input".into(),
                localized_name: "flight_input_localized".into(),
                action_type: bevy_xr::actions::ActionType::Vector,
            },
            FlightActionMarker, //lets try a marker component
        ))
        .id();

    //create a binding
    let binding_index = commands
        .spawn(XRUtilsBinding {
            profile: "/interaction_profiles/valve/index_controller".into(),
            binding: "/user/hand/right/input/thumbstick".into(),
        })
        .id();
    let binding_touch = commands
        .spawn(XRUtilsBinding {
            profile: "/interaction_profiles/oculus/touch_controller".into(),
            binding: "/user/hand/right/input/thumbstick".into(),
        })
        .id();
    //add action to set, this isnt the best
    //TODO look into a better system
    commands.entity(action).add_child(binding_index);
    commands.entity(action).add_child(binding_touch);
    commands.entity(set).add_child(action);
}

fn read_action_with_marker_component(
    mut action_query: Query<&XRUtilsActionState, With<FlightActionMarker>>,
) {
    //now for the actual checking
    for state in action_query.iter_mut() {
        info!("action state is: {:?}", state);
    }
}

//lets add some flycam stuff
fn handle_flight_input(
    action_query: Query<&XRUtilsActionState, With<FlightActionMarker>>,
    mut oxr_root: Query<&mut Transform, With<XrTrackingRoot>>,
    time: Res<Time>,
    //use the views for hmd orientation
    views: ResMut<OxrViews>,
) {
    //now for the actual checking
    for state in action_query.iter() {
        // info!("action state is: {:?}", state);
        match state {
            XRUtilsActionState::Bool(_) => (),
            XRUtilsActionState::Float(_) => (),
            XRUtilsActionState::Vector(vector_state) => {
                //assuming we are mapped to a vector lets fly
                let input_vector = vec3(
                    vector_state.current_state[0],
                    0.0,
                    -vector_state.current_state[1],
                );
                //hard code speed for now
                let speed = 5.0;

                let root = oxr_root.get_single_mut();
                match root {
                    Ok(mut root_position) => {
                        //lets assume HMD based direction for now
                        let view = views.first();
                        match view {
                            Some(v) => {
                                let reference_quat = v.pose.orientation.to_quat();
                                let locomotion_vector = reference_quat.mul_vec3(input_vector);

                                root_position.translation +=
                                    locomotion_vector * speed * time.delta_seconds();
                            }
                            None => return,
                        }
                    }
                    Err(_) => {
                        info!("handle_flight_input: error getting root position for flight actions")
                    }
                }
            }
        }
    }
}
