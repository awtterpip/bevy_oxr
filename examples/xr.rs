//! A simple 3D scene with light shining over a cube sitting on a plane.
use bevy_openxr::{DefaultXrPlugins, LEFT_XR_TEXTURE_HANDLE, RIGHT_XR_TEXTURE_HANDLE};
use bevy::{prelude::*, render::camera::RenderTarget};
use bevy::prelude::Component;
use bevy::render::camera::Viewport;
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrInstance, XrSession, XrViews};

fn main() {
    App::new()
        .add_plugins(DefaultXrPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, head_movement)
        .run();
}

#[derive(Component)]
enum CameraType {
    Left,
    Right,
    Middle,
}


/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }, CameraType::Middle));

    // let viewport = Viewport{
    //     physical_position: Default::default(),
    //     physical_size: UVec2::splat(2000),
    //     depth: 0.0..1.0,
    // };

    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            order: -1,
            target: RenderTarget::TextureView(LEFT_XR_TEXTURE_HANDLE),
            viewport: None,
            ..default()
        },
        ..default()
    }, CameraType::Left));
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            order: -1,
            target: RenderTarget::TextureView(RIGHT_XR_TEXTURE_HANDLE),
            viewport: None,
            ..default()
        },
        ..default()
    }, CameraType::Right));
}
fn head_movement(views: ResMut<XrViews>, mut query: Query<(&mut Transform, &Camera, &CameraType)>) {
    let views = views.lock().unwrap();
    let mut f = || -> Option<()> {
        let midpoint = (views.get(0)?.pose.position.to_vec3()
            + views.get(1)?.pose.position.to_vec3())
            / 2.;
        for (mut t, _, camera_type) in query.iter_mut() {
            match camera_type {
                CameraType::Left => {
                    t.translation = views.get(0)?.pose.position.to_vec3()
                },
                CameraType::Right => {
                    t.translation = views.get(1)?.pose.position.to_vec3()
                },
                CameraType::Middle => {
                    t.translation = midpoint;
                },
            }
        }
        let left_rot = views.get(0).unwrap().pose.orientation.to_quat();
        let right_rot = views.get(1).unwrap().pose.orientation.to_quat();
        let mid_rot = if left_rot.dot(right_rot) >= 0. {
            left_rot.slerp(right_rot, 0.5)
        } else {
            right_rot.slerp(left_rot, 0.5)
        };
        for (mut t, _, camera_type) in query.iter_mut() {
            match camera_type {
                CameraType::Left => {
                    t.rotation = left_rot
                },
                CameraType::Right => {
                    t.rotation = right_rot
                },
                CameraType::Middle => {
                    t.rotation = mid_rot;
                },
            }
        }


    //     for (mut projection, mut transform, eye) in cam.iter_mut() {
    //     let view_idx = match eye {
    //         Eye::Left => 0,
    //         Eye::Right => 1,
    //     };
    //     let view = views.get(view_idx).unwrap();
    //
    //     projection.fov = view.fov;
    //
    //     transform.rotation = view.pose.orientation.to_quat();
    //     let pos = view.pose.position;
    //     transform.translation = pos.to_vec3();
    // }

        Some(())
    };
    f();
}
pub trait Vec3Conv {
    fn to_vec3(&self) -> Vec3;
}

impl Vec3Conv for openxr::Vector3f {
    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}
pub trait QuatConv {
    fn to_quat(&self) -> Quat;
}

impl QuatConv for openxr::Quaternionf {
    fn to_quat(&self) -> Quat {
        Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}

// fn head_movement(right_camera: Query<(&mut Transform, &RightCamera), Without<LeftCamera>>, left_camera: Query<(&mut Transform, &LeftCamera), Without<RightCamera>>, xr_input: Res<bevy_openxr::input::XrInput>, instance: Res<XrInstance>, session: Res<XrSession>) {
//
//     // let stage =
//     //     session.create_reference_space(openxr::ReferenceSpaceType::VIEW, openxr::Posef::IDENTITY).unwrap();
//     // eprintln!("a: {:#?}", stage.locate(&xr_input.stage, xr_input.action_set.).unwrap().pose);
// }