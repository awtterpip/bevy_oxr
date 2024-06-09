use std::sync::Arc;

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        RenderApp,
    },
};
use bevy_xr::session::{XrCreateSession, XrDestroySession};

use crate::{init::create_xr_session, resources::OxrSession};

pub struct OxrReferenceSpacePlugin {
    pub default_primary_ref_space: openxr::ReferenceSpaceType,
}
impl Default for OxrReferenceSpacePlugin {
    fn default() -> Self {
        Self {
            default_primary_ref_space: openxr::ReferenceSpaceType::STAGE,
        }
    }
}

/// Resource specifying what the type should be for [`OxrPrimaryReferenceSpace`]. Set through [`OxrReferenceSpacePlugin`].
#[derive(Resource)]
pub struct OxrPrimaryReferenceSpaceType(openxr::ReferenceSpaceType);

/// The Default Reference space used for locating things
#[derive(Resource, Deref, ExtractResource, Clone)]
pub struct OxrPrimaryReferenceSpace(pub Arc<openxr::Space>);

/// The Reference space used for locating spaces on this entity
#[derive(Component)]
pub struct OxrReferenceSpace(pub openxr::Space);

impl Plugin for OxrReferenceSpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<OxrPrimaryReferenceSpace>::default())
            .insert_resource(OxrPrimaryReferenceSpaceType(self.default_primary_ref_space))
            .add_systems(
                XrCreateSession,
                create_primary_reference_space.after(create_xr_session),
            )
            .add_systems(XrDestroySession, destroy_primary_reference_space);

        let render_app = app.sub_app_mut(RenderApp);

        render_app.add_systems(XrDestroySession, destroy_primary_reference_space);
    }
}

pub fn create_primary_reference_space(
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

pub fn destroy_primary_reference_space(world: &mut World) {
    world.remove_resource::<OxrPrimaryReferenceSpace>();
}
