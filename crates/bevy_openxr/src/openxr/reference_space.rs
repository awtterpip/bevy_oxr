use std::sync::Arc;

use bevy::{
    prelude::*,
    render::extract_resource::{ExtractResource, ExtractResourcePlugin},
};
use bevy_xr::session::{status_changed_to, XrStatus};

use crate::{init::OxrPreUpdateSet, resources::OxrSession};

pub struct OxrReferenceSpacePlugin {
    default_primary_ref_space: openxr::ReferenceSpaceType,
}
impl Default for OxrReferenceSpacePlugin {
    fn default() -> Self {
        Self {
            default_primary_ref_space: openxr::ReferenceSpaceType::LOCAL_FLOOR_EXT,
        }
    }
}

#[derive(Resource)]
struct OxrPrimaryReferenceSpaceType(openxr::ReferenceSpaceType);
// TODO: this will keep the session alive so we need to remove this in the render world too
/// The Default Reference space used for locating things
#[derive(Resource, Deref, ExtractResource, Clone)]
pub struct OxrPrimaryReferenceSpace(pub Arc<openxr::Space>);

/// The Reference space used for locating spaces on this entity
#[derive(Component)]
pub struct OxrReferenceSpace(pub openxr::Space);

impl Plugin for OxrReferenceSpacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OxrPrimaryReferenceSpaceType(self.default_primary_ref_space));
        app.add_plugins(ExtractResourcePlugin::<OxrPrimaryReferenceSpace>::default());
        app.add_systems(
            PreUpdate,
            set_primary_ref_space
                .run_if(status_changed_to(XrStatus::Ready))
                .in_set(OxrPreUpdateSet::UpdateCriticalComponents),
        );
    }
}

fn set_primary_ref_space(
    session: Res<OxrSession>,
    space_type: Res<OxrPrimaryReferenceSpaceType>,
    mut cmds: Commands,
) {
    match session.create_reference_space(space_type.0, openxr::Posef::IDENTITY) {
        Ok(space) => {
            cmds.insert_resource(OxrPrimaryReferenceSpace(Arc::new(space)));
        }
        Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT) => {
            error!("Required Extension for Reference Space not loaded");
        }
        Err(err) => error!("Error while creating reference space: {}", err.to_string()),
    };
}
