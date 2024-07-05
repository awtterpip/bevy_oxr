use bevy::{
    prelude::*,
    render::{extract_resource::ExtractResourcePlugin, RenderApp},
};
use bevy_mod_xr::{
    session::{XrCreateSession, XrDestroySession},
    spaces::{XrPrimaryReferenceSpace, XrReferenceSpace},
};

use crate::{init::create_xr_session, session::OxrSession};

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
struct OxrDefaultPrimaryReferenceSpaceType(openxr::ReferenceSpaceType);

/// The Default Reference space used for locating things
// #[derive(Resource, Deref, ExtrctResource, Clone)]
// pub struct OxrPrimaryReferenceSpace(pub Arc<openxr::Space>);

/// The Reference space used for locating spaces on this entity
#[derive(Component)]
pub struct OxrReferenceSpace(pub openxr::Space);

impl Plugin for OxrReferenceSpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<XrPrimaryReferenceSpace>::default())
            .insert_resource(OxrDefaultPrimaryReferenceSpaceType(
                self.default_primary_ref_space,
            ))
            .add_systems(
                XrCreateSession,
                set_primary_ref_space.after(create_xr_session),
            )
            .add_systems(XrDestroySession, cleanup);

        let render_app = app.sub_app_mut(RenderApp);

        render_app.add_systems(XrDestroySession, cleanup);
    }
}

fn cleanup(query: Query<Entity, With<XrReferenceSpace>>, mut cmds: Commands) {
    cmds.remove_resource::<XrPrimaryReferenceSpace>();
    for e in &query {
        cmds.entity(e).remove::<XrReferenceSpace>();
    }
}

fn set_primary_ref_space(
    session: Res<OxrSession>,
    space_type: Res<OxrDefaultPrimaryReferenceSpaceType>,
    mut cmds: Commands,
) {
    match session.create_reference_space(space_type.0, Transform::IDENTITY) {
        Ok(space) => {
            cmds.insert_resource(XrPrimaryReferenceSpace(space));
        }
        Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT) => {
            error!("Required Extension for Reference Space not loaded");
        }
        Err(err) => error!("Error while creating reference space: {}", err.to_string()),
    };
}
