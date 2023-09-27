use std::f32::consts::PI;

use bevy::prelude::{
    info, Color, Component, Gizmos, GlobalTransform, Quat, Query, Transform, Vec3, With, Without,
};

use super::trackers::{AimPose, OpenXRTrackingRoot};

#[derive(Component)]
pub struct XRDirectInteractor;

#[derive(Component)]
pub struct XRRayInteractor;

#[derive(Component)]
pub enum XRInteractableState {
    Idle,
    Hover,
    Select,
}

impl Default for XRInteractableState {
    fn default() -> Self {
        XRInteractableState::Idle
    }
}

#[derive(Component)]
pub enum XRInteractorState {
    Idle,
    Selecting,
}
impl Default for XRInteractorState {
    fn default() -> Self {
        XRInteractorState::Idle
    }
}

#[derive(Component)]
pub struct XRInteractable;

pub fn draw_interaction_gizmos(
    mut gizmos: Gizmos,
    interactable_query: Query<
        (&GlobalTransform, &XRInteractableState),
        (With<XRInteractable>, Without<XRDirectInteractor>),
    >,
    interactor_query: Query<
        (
            &GlobalTransform,
            &XRInteractorState,
            Option<&XRDirectInteractor>,
            Option<&XRRayInteractor>,
            Option<&AimPose>,
        ),
        (Without<XRInteractable>),
    >,
    tracking_root_query: Query<(&mut Transform, With<OpenXRTrackingRoot>)>,
) {
    let root = tracking_root_query.get_single().unwrap().0;
    for (global_transform, interactable_state) in interactable_query.iter() {
        let transform = global_transform.compute_transform();
        let color = match interactable_state {
            XRInteractableState::Idle => Color::RED,
            XRInteractableState::Hover => Color::YELLOW,
            XRInteractableState::Select => Color::GREEN,
        };
        gizmos.sphere(transform.translation, transform.rotation, 0.1, color);
    }

    for (interactor_global_transform, interactor_state, direct, ray, aim) in interactor_query.iter()
    {
        let transform = interactor_global_transform.compute_transform();
        match direct {
            Some(_) => {
                let mut local = transform.clone();
                local.scale = Vec3::splat(0.1);
                let quat = Quat::from_euler(
                    bevy::prelude::EulerRot::XYZ,
                    45.0 * (PI / 180.0),
                    0.0,
                    45.0 * (PI / 180.0),
                );
                local.rotation = quat;
                let color = match interactor_state {
                    XRInteractorState::Idle => Color::BLUE,
                    XRInteractorState::Selecting => Color::PURPLE,
                };
                gizmos.cuboid(local, color);
            }
            None => (),
        }
        match ray {
            Some(_) => match aim {
                Some(aim) => {
                    let color = match interactor_state {
                        XRInteractorState::Idle => Color::BLUE,
                        XRInteractorState::Selecting => Color::PURPLE,
                    };
                    gizmos.ray(
                        root.translation + root.rotation.mul_vec3(aim.0.translation),
                        root.rotation.mul_vec3(aim.0.forward()),
                        color,
                    );
                }
                None => todo!(),
            },
            None => (),
        }
    }
}

pub fn direct_interaction(
    mut interactable_query: Query<
        (&GlobalTransform, &mut XRInteractableState),
        (With<XRInteractable>, Without<XRDirectInteractor>),
    >,
    interactor_query: Query<
        (&GlobalTransform, &XRInteractorState),
        (With<XRDirectInteractor>, Without<XRInteractable>),
    >,
) {
    for (xr_interactable_global_transform, mut state) in interactable_query.iter_mut() {
        let mut hovered = false;
        let mut selected = false;
        for (interactor_global_transform, interactor_state) in interactor_query.iter() {
            //check for sphere overlaps
            let size = 0.1;
            if interactor_global_transform
                .compute_transform()
                .translation
                .distance_squared(
                    xr_interactable_global_transform
                        .compute_transform()
                        .translation,
                )
                < (size * size) * 2.0
            {
                //check for selections first
                match interactor_state {
                    XRInteractorState::Idle => hovered = true,
                    XRInteractorState::Selecting => {
                        selected = true;
                    }
                }
            }
        }
        //check what we found
        //also i dont like this
        if selected {
            *state = XRInteractableState::Select;
        } else if hovered {
            *state = XRInteractableState::Hover;
        } else {
            *state = XRInteractableState::Idle;
        }
    }
}

pub fn ray_interaction(
    mut interactable_query: Query<
        (&GlobalTransform, &mut XRInteractableState),
        (With<XRInteractable>, Without<XRDirectInteractor>),
    >,
    interactor_query: Query<
        (&GlobalTransform, &XRInteractorState, Option<&AimPose>),
        (With<XRRayInteractor>, Without<XRInteractable>),
    >,
    tracking_root_query: Query<(&mut Transform, With<OpenXRTrackingRoot>)>,
) {
    for (xr_interactable_global_transform, mut state) in interactable_query.iter_mut() {
        let mut hovered = false;
        let mut selected = false;
        for (interactor_global_transform, interactor_state, aim) in interactor_query.iter() {
            //check for ray-sphere intersection
            let sphere_transform = xr_interactable_global_transform.compute_transform();
            let center = sphere_transform.translation;
            let radius: f32 = 0.1;
            //I hate this but the aim pose needs the root for now
            let root = tracking_root_query.get_single().unwrap().0;
            match aim {
                Some(aim) => {
                    let ray_origin = root.translation + root.rotation.mul_vec3(aim.0.translation);
                    let ray_dir = root.rotation.mul_vec3(aim.0.forward());

                    if ray_sphere_intersection(
                        center,
                        radius,
                        ray_origin,
                        ray_dir.normalize_or_zero(),
                    ) {
                        //check for selections first
                        match interactor_state {
                            XRInteractorState::Idle => hovered = true,
                            XRInteractorState::Selecting => {
                                selected = true;
                            }
                        }
                    }
                }
                None => info!("no aim pose"),
            }
        }
        //check what we found
        //also i dont like this
        if selected {
            *state = XRInteractableState::Select;
        } else if hovered {
            *state = XRInteractableState::Hover;
        } else {
            *state = XRInteractableState::Idle;
        }
    }
}

fn ray_sphere_intersection(center: Vec3, radius: f32, ray_origin: Vec3, ray_dir: Vec3) -> bool {
    let l = center - ray_origin;
    let adj = l.dot(ray_dir);
    let d2 = l.dot(l) - (adj * adj);
    let radius2 = radius * radius;
    if d2 > radius2 {
        return false;
    }
    let thc = (radius2 - d2).sqrt();
    let t0 = adj - thc;
    let t1 = adj + thc;

    if t0 < 0.0 && t1 < 0.0 {
        return false;
    }

    // let distance = if t0 < t1 { t0 } else { t1 };
    return true;
}
