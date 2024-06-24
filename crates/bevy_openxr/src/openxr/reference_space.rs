use std::sync::Arc;

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        RenderApp,
    }, utils::HashSet,
};
use bevy_xr::{
    session::{XrSessionCreated, XrSessionExiting},
    spaces::{XrDestroySpace, XrPrimaryReferenceSpace, XrReferenceSpace, XrSpace},
};

use crate::{resources::OxrInstance, session::OxrSession};

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

#[derive(Resource)]
struct OxrDefaultPrimaryReferenceSpaceType(openxr::ReferenceSpaceType);
/// The Default Reference space used for locating things
// #[derive(Resource, Deref, ExtrctResource, Clone)]
// pub struct OxrPrimaryReferenceSpace(pub Arc<openxr::Space>);

/// The Reference space used for locating spaces on this entity
// #[derive(Component)]
// pub struct OxrReferenceSpace(pub openxr::Space);

impl Plugin for OxrReferenceSpacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OxrDefaultPrimaryReferenceSpaceType(
            self.default_primary_ref_space,
        ));
        app.add_plugins(ExtractResourcePlugin::<XrPrimaryReferenceSpace>::default());
        app.add_systems(XrSessionCreated, set_primary_ref_space);
        app.add_systems(XrSessionExiting, cleanup);
        app.sub_app_mut(RenderApp)
            .add_systems(XrSessionExiting, cleanup);
    }
}

fn cleanup(
    query: Query<(Entity, &XrReferenceSpace)>,
    mut cmds: Commands,
    instance: Res<OxrInstance>,
    ref_space: Option<Res<XrPrimaryReferenceSpace>>,
) {
    let mut to_destroy = HashSet::<XrSpace>::new();
    if let Some(space) = ref_space {
        to_destroy.insert(***space);
    }
    cmds.remove_resource::<XrPrimaryReferenceSpace>();
    for (e, space) in &query {
        cmds.entity(e).remove::<XrReferenceSpace>();
        to_destroy.insert(**space);
    }
    for space in to_destroy.into_iter() {
        let _ = instance.destroy_space(space);
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
