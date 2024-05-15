// a simple example showing basic actions

use std::borrow::Cow;

use bevy::prelude::*;
use bevy_openxr::{
    action_binding::OxrSuggestActionBinding,
    action_set_attaching::{AttachedActionSets, OxrAttachActionSet},
    add_xr_plugins,
    resources::{OxrInstance, OxrSession},
};
use openxr::{ActiveActionSet, Path};

fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins(bevy_xr_utils::hand_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, create_action)
        .init_resource::<TestAction>()
        .add_systems(Update, read_action)
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
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Resource, Default)]
struct TestAction {
    action: Option<openxr::Action<bool>>,
}

fn create_action(
    mut writer: EventWriter<OxrSuggestActionBinding>,
    instance: ResMut<OxrInstance>,
    mut test: ResMut<TestAction>,
    mut set_writer: EventWriter<OxrAttachActionSet>,
) {
    let name = "test";
    let pretty_name = "pretty test";
    let priority = u32::MIN;
    //create action set
    let set: openxr::ActionSet = instance
        .create_action_set(name, pretty_name, priority)
        .unwrap();

    let action_name = "action_name";
    let localized_name = "localized_name";
    //create new action from action set
    let bool_action: openxr::Action<bool> = set
        .create_action::<bool>(action_name, localized_name, &[])
        .unwrap();

    //interaction profile
    let profile = Cow::from("/interaction_profiles/valve/index_controller");
    //bindings
    let binding = vec![Cow::from("/user/hand/right/input/a/click")];
    let sugestion = OxrSuggestActionBinding {
        action: bool_action.as_raw(),
        interaction_profile: profile,
        bindings: binding,
    };

    //finally send the suggestion
    writer.send(sugestion);
    set_writer.send(OxrAttachActionSet(set.clone()));

    test.action = Some(bool_action);
}

fn read_action(
    session: ResMut<OxrSession>,
    test: ResMut<TestAction>,
    attached: ResMut<AttachedActionSets>
) {
    //maybe sync before?
    let why = &attached.sets.iter().map(|v| ActiveActionSet::from(v)).collect::<Vec<_>>();
    let sync = session.sync_actions(&why[..]);
    match sync {
        Ok(_) =>info!("sync ok"),
        Err(_) => error!("sync error"),
    }


    //now check the action?
    let action = &test.action;
    match action {
        Some(act) => {
            let thing = act.state(&session, Path::NULL);
            match thing {
                Ok(a) => {
                    info!("action state: {:?}", a);
                }
                Err(_) => info!("error getting state"),
            }
        }
        None => info!("no action"),
    }
}