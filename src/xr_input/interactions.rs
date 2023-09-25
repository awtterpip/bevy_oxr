use std::f32::consts::PI;

use bevy::prelude::{
    info, Color, Component, Gizmos, GlobalTransform, Quat, Query, Vec3, With, Without,
};

#[derive(Component)]
pub struct XRDirectInteractor;
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
        (&GlobalTransform, &XRInteractorState),
        (With<XRDirectInteractor>, Without<XRInteractable>),
    >,
) {
    for (global_transform, interactable_state) in interactable_query.iter() {
        let transform = global_transform.compute_transform();
        let color = match interactable_state {
            XRInteractableState::Idle => Color::RED,
            XRInteractableState::Hover => Color::YELLOW,
            XRInteractableState::Select => Color::GREEN,
        };
        gizmos.sphere(transform.translation, transform.rotation, 0.1, color);
    }

    for (interactor_global_transform, interactor_state) in interactor_query.iter() {
        let mut transform = interactor_global_transform.compute_transform();
        transform.scale = Vec3::splat(0.1);
        let quat = Quat::from_euler(
            bevy::prelude::EulerRot::XYZ,
            45.0 * (PI / 180.0),
            0.0,
            45.0 * (PI / 180.0),
        );
        transform.rotation = quat;
        let color = match interactor_state {
            XRInteractorState::Idle => Color::BLUE,
            XRInteractorState::Selecting => Color::PURPLE,
        };
        gizmos.cuboid(transform, color);
    }
}

pub fn hover_interaction(
    mut interactable_query: Query<
        (&GlobalTransform, &mut XRInteractableState),
        (With<XRInteractable>, Without<XRDirectInteractor>),
    >,
    interactor_query: Query<
        (&GlobalTransform, &XRInteractorState),
        (With<XRDirectInteractor>, Without<XRInteractable>),
    >,
) {
    'interactable: for (xr_interactable_global_transform, mut state) in
        interactable_query.iter_mut()
    {
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
                info!("we overlapping");
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
