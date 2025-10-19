use bevy_app::{App, Plugin, PreUpdate, Startup};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::{Or, With};
use bevy_ecs::schedule::IntoScheduleConfigs as _;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_ecs::world::World;
use bevy_log::{debug, error, warn};
use bevy_mod_xr::hands::{
    spawn_hand_bones, HandBone, HandSide, SpawnHandTracker, SpawnHandTrackerCommandExecutor,
    XrHandBoneRadius,
};
use bevy_mod_xr::hands::{LeftHand, RightHand, XrHandBoneEntities};
use bevy_mod_xr::session::{XrPreDestroySession, XrSessionCreated};
use bevy_mod_xr::spaces::{
    XrPrimaryReferenceSpace, XrReferenceSpace, XrSpaceLocationFlags, XrSpaceSyncSet,
    XrSpaceVelocityFlags, XrVelocity,
};
use bevy_transform::components::Transform;
use openxr::{SpaceLocationFlags, SpaceVelocityFlags};

use crate::helper_traits::ToVec3;
use crate::resources::OxrFrameState;
use crate::resources::Pipelined;
use crate::session::OxrSession;
use crate::spaces::{OxrSpaceLocationFlags, OxrSpaceVelocityFlags};
use crate::{openxr_session_available, openxr_session_running};

pub struct HandTrackingPlugin {
    default_hands: bool,
}
impl Default for HandTrackingPlugin {
    fn default() -> Self {
        Self {
            default_hands: true,
        }
    }
}

impl Plugin for HandTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            locate_hands
                .in_set(XrSpaceSyncSet)
                .run_if(openxr_session_running),
        );
        if self.default_hands {
            app.add_systems(XrPreDestroySession, clean_up_default_hands)
                .add_systems(XrSessionCreated, spawn_default_hands);
        }
        app.add_systems(Startup, set_spawn_executor.run_if(openxr_session_available));
    }
}

fn set_spawn_executor(mut cmds: Commands) {
    cmds.insert_resource(SpawnHandTrackerCommandExecutor(handle_tracker_spawn))
}

fn handle_tracker_spawn(world: &mut World, tracker: Entity, side: HandSide) {
    let Some(session) = world.get_resource_mut::<OxrSession>() else {
        error!("unable to get session while creating hand tracker");
        return;
    };
    debug!("spawning hand");
    let oxr_tracker = match session.create_hand_tracker(match side {
        HandSide::Left => openxr::HandEXT::LEFT,
        HandSide::Right => openxr::HandEXT::RIGHT,
    }) {
        Ok(t) => t,
        Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT) => {
            warn!("Handtracking Extension not loaded, Unable to create Handtracker!");
            return;
        }
        Err(err) => {
            warn!("Error while creating Handtracker: {}", err.to_string());
            return;
        }
    };

    world
        .entity_mut(tracker)
        .insert(OxrHandTracker(oxr_tracker));
}

fn spawn_default_hands(mut cmds: Commands) {
    debug!("spawning default hands");
    let left_bones = spawn_hand_bones(&mut cmds, |_| {
        (
            DefaultHandBone,
            LeftHand,
            OxrSpaceLocationFlags(openxr::SpaceLocationFlags::default()),
        )
    });
    let right_bones = spawn_hand_bones(&mut cmds, |_| {
        (
            DefaultHandBone,
            RightHand,
            OxrSpaceLocationFlags(openxr::SpaceLocationFlags::default()),
        )
    });
    cmds.queue(SpawnHandTracker {
        joints: XrHandBoneEntities(left_bones),
        tracker_bundle: DefaultHandTracker,
        side: HandSide::Left,
    });
    cmds.queue(SpawnHandTracker {
        joints: XrHandBoneEntities(right_bones),
        tracker_bundle: DefaultHandTracker,
        side: HandSide::Right,
    });
}

#[derive(Component, Clone, Copy)]
pub struct DefaultHandTracker;
#[derive(Component, Clone, Copy)]
pub struct DefaultHandBone;

#[allow(clippy::type_complexity)]
fn clean_up_default_hands(
    mut cmds: Commands,
    query: Query<Entity, Or<(With<DefaultHandTracker>, With<DefaultHandBone>)>>,
) {
    for e in &query {
        debug!("removing default hand entity");
        cmds.entity(e).despawn();
    }
}

#[derive(Deref, DerefMut, Component)]
pub struct OxrHandTracker(pub openxr::HandTracker);

fn locate_hands(
    default_ref_space: Res<XrPrimaryReferenceSpace>,
    frame_state: Res<OxrFrameState>,
    tracker_query: Query<(
        &OxrHandTracker,
        Option<&XrReferenceSpace>,
        &XrHandBoneEntities,
    )>,
    session: Res<OxrSession>,
    mut bone_query: Query<(
        &HandBone,
        &mut XrHandBoneRadius,
        &mut Transform,
        Option<&mut XrVelocity>,
        &mut OxrSpaceLocationFlags,
        &mut XrSpaceLocationFlags,
        Option<&mut OxrSpaceVelocityFlags>,
        Option<&mut XrSpaceVelocityFlags>,
    )>,
    pipelined: Option<Res<Pipelined>>,
) {
    for (tracker, ref_space, hand_entities) in &tracker_query {
        let wants_velocities = hand_entities
            .0
            .iter()
            .filter_map(|e| bone_query.get(*e).ok())
            .any(|v| v.3.is_some());
        let time = if pipelined.is_some() {
            openxr::Time::from_nanos(
                frame_state.predicted_display_time.as_nanos()
                    + frame_state.predicted_display_period.as_nanos(),
            )
        } else {
            frame_state.predicted_display_time
        };
        let ref_space = ref_space.map(|v| &v.0).unwrap_or(&default_ref_space.0);
        let mut clear_flags = || {
            for e in hand_entities.0.iter() {
                let Ok((_, _, _, _, mut flags, mut xr_flags, vel_flags, xr_vel_flags)) =
                    bone_query.get_mut(*e)
                else {
                    continue;
                };
                flags.0 = SpaceLocationFlags::EMPTY;
                if let Some(mut flags) = vel_flags {
                    flags.0 = SpaceVelocityFlags::EMPTY;
                }
                xr_flags.position_tracked = false;
                xr_flags.rotation_tracked = false;
                if let Some(mut flags) = xr_vel_flags {
                    flags.linear_valid = false;
                    flags.angular_valid = false;
                }
            }
        };
        let (joints, vels) = if wants_velocities {
            let (loc, vel) =
                match session.locate_hand_joints_with_velocities(tracker, ref_space, time) {
                    Ok(Some(v)) => v,
                    Ok(None) => {
                        clear_flags();
                        continue;
                    }
                    Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT) => {
                        error!("HandTracking Extension not loaded");
                        clear_flags();
                        continue;
                    }
                    Err(err) => {
                        warn!("Error while locating hand joints: {}", err.to_string());
                        clear_flags();
                        continue;
                    }
                };
            (loc, Some(vel))
        } else {
            let space = match session.locate_hand_joints(tracker, ref_space, time) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    clear_flags();
                    continue;
                }
                Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT) => {
                    error!("HandTracking Extension not loaded");
                    clear_flags();
                    continue;
                }
                Err(err) => {
                    warn!("Error while locating hand joints: {}", err.to_string());
                    clear_flags();
                    continue;
                }
            };
            (space, None)
        };
        let bone_entities = match bone_query.get_many_mut(hand_entities.0) {
            Ok(v) => v,
            Err(err) => {
                warn!("unable to get entities, {}", err);
                continue;
            }
        };
        for (
            bone,
            mut bone_radius,
            mut transform,
            velocity,
            mut location_flags,
            mut xr_location_flags,
            velocity_flags,
            xr_velocity_flags,
        ) in bone_entities
        {
            let joint = joints[*bone as usize];
            if let Some(mut velocity) = velocity {
                let Some(vels) = vels.as_ref() else {
                    error!("somehow got a hand bone with an XrVelocity component, but there are no velocities");
                    continue;
                };
                let Some(mut vel_flags) = velocity_flags else {
                    error!("somehow got a hand bone with an XrVelocity component, but without velocity flags");
                    continue;
                };
                let Some(mut xr_vel_flags) = xr_velocity_flags else {
                    error!("somehow got a hand bone with an XrVelocity component, but without velocity flags");
                    continue;
                };
                let vel = vels[*bone as usize];
                let flags = OxrSpaceVelocityFlags(vel.velocity_flags);
                if flags.linear_valid() {
                    velocity.linear = vel.linear_velocity.to_vec3();
                }
                if flags.angular_valid() {
                    velocity.angular = vel.angular_velocity.to_vec3();
                }
                xr_vel_flags.linear_valid = flags.linear_valid();
                xr_vel_flags.angular_valid = flags.angular_valid();
                *vel_flags = flags;
            }

            **bone_radius = joint.radius;
            let flags = OxrSpaceLocationFlags(joint.location_flags);
            if flags.pos_valid() {
                transform.translation.x = joint.pose.position.x;
                transform.translation.y = joint.pose.position.y;
                transform.translation.z = joint.pose.position.z;
            }

            if flags.rot_valid() {
                transform.rotation.x = joint.pose.orientation.x;
                transform.rotation.y = joint.pose.orientation.y;
                transform.rotation.z = joint.pose.orientation.z;
                transform.rotation.w = joint.pose.orientation.w;
            }
            xr_location_flags.position_tracked = flags.pos_valid() && flags.pos_tracked();
            xr_location_flags.rotation_tracked = flags.rot_valid() && flags.rot_tracked();
            *location_flags = flags;
        }
    }
}
