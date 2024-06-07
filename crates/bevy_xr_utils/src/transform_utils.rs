use bevy::prelude::*;
use bevy_openxr::{helper_traits::{ToQuat, ToVec3}, init::OxrTrackingRoot, resources::OxrViews};

pub struct TransformUtilitiesPlugin;

impl Plugin for TransformUtilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SnapToRotation>();
        app.add_event::<SnapToPosition>();
        app.add_systems(PostUpdate, handle_transform_events);
    }
}

//events
#[derive(Event, Debug)]
pub struct SnapToRotation(pub Quat);

#[derive(Event, Debug)]
pub struct SnapToPosition(pub Vec3);

pub fn handle_transform_events(
    mut root_query: Query<&mut Transform, With<OxrTrackingRoot>>,
    views: ResMut<OxrViews>,
    mut position_reader: EventReader<SnapToPosition>,
    mut rotation_reader: EventReader<SnapToRotation>,
) {
    let result = root_query.get_single_mut();
    match result {
        Ok(mut root_transform) => {
            //rotation first
            let view = views.first();
            match view {
                Some(view) => {
                    //get our parameters together
                    let mut view_translation = view.pose.position.to_vec3();
                    let view_quat = view.pose.orientation.to_quat();
                    let view_yaw = view_quat.to_euler(EulerRot::XYZ).1;
                    let science = Quat::from_axis_angle(*root_transform.up(), view_yaw);
                    let invert_science = science.inverse();
                    view_translation.y = 0.0; //we want to do rotations around the same height as the root
                    let root_local = root_transform.translation;
                    let view_global = root_transform.rotation.mul_vec3(view_translation) + root_local;
                    //now set our rotation for every event, this does mean only the last event matters
                    for snap in rotation_reader.read() {
                        let rotation_quaternion = snap.0.mul_quat(invert_science);
                        root_transform.rotate_around(view_global, rotation_quaternion);
                    }
                    //position second
                    for position in position_reader.read() {
                        root_transform.translation = position.0;
                    }
                    
                },
                None => debug!("error getting first view"),
            }
        }
        Err(_) => debug!("error getting root transform"),
    }
}
