//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::any::TypeId;

use bevy::prelude::*;
use bevy_openxr::{
    actions::{create_action_sets, ActionApp},
    add_xr_plugins, resources::{TypedAction, XrActions, XrInstance},
};
use bevy_xr::actions::{Action, ActionInfo, ActionState, ActionType};
use openxr::Binding;

pub struct Jump;

impl Action for Jump {
    type ActionType = bool;

    fn info() -> ActionInfo {
        ActionInfo {
            pretty_name: "jump",
            name: "jump",
            action_type: ActionType::Bool,
            type_id: TypeId::of::<Self>(),
        }
    }
}


fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_systems(Startup, setup.after(create_action_sets))
        .add_systems(Update, read_action_state)
        .register_action::<Jump>()
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    actions: Res<XrActions>,
    instance: Res<XrInstance>,
) {
    let TypedAction::Bool(action) = actions.get(&TypeId::of::<Jump>()).unwrap() else {
        unreachable!()
    };
    instance.suggest_interaction_profile_bindings(instance.string_to_path("/interaction_profiles/oculus/touch_controller").unwrap(), &[
        Binding::new(action, instance.string_to_path("/user/hand/right/input/a/click").unwrap())
    ]).unwrap();
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
    // // camera
    // commands.spawn(XrCameraBundle {
    //     transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     camera: Camera {
    //         target: RenderTarget::TextureView(ManualTextureViewHandle(XR_TEXTURE_INDEX + 1)),
    //         ..default()
    //     },
    //     ..default()
    // });
}

fn read_action_state(
    state: Res<ActionState<Jump>>
) {
    info!("{}", state.pressed())
}

