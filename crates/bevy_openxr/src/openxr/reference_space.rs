use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    init::{OxrSessionResourceCreator, OxrSessionResourceCreators},
    resources::OxrSession,
    types::Result,
};

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
struct OxrPrimaryReferenceSpaceCreator(openxr::ReferenceSpaceType, Option<Arc<openxr::Space>>);

impl OxrSessionResourceCreator for OxrPrimaryReferenceSpaceCreator {
    fn update(&mut self, world: &mut World) -> Result<()> {
        let session = world.resource::<OxrSession>();
        let space = session.create_reference_space(self.0, openxr::Posef::IDENTITY)?;
        self.1 = Some(Arc::new(space));
        Ok(())
    }

    fn insert_to_world(&mut self, world: &mut World) {
        world.insert_resource(OxrPrimaryReferenceSpace(self.1.clone().unwrap()));
    }

    fn insert_to_render_world(&mut self, world: &mut World) {
        self.insert_to_world(world)
    }

    fn remove_from_world(&mut self, world: &mut World) {
        world.remove_resource::<OxrPrimaryReferenceSpace>();
    }

    fn remove_from_render_world(&mut self, world: &mut World) {
        self.remove_from_world(world);
    }
}

/// The Default Reference space used for locating things
#[derive(Resource, Deref, Clone)]
pub struct OxrPrimaryReferenceSpace(pub Arc<openxr::Space>);

/// The Reference space used for locating spaces on this entity
#[derive(Component)]
pub struct OxrReferenceSpace(pub openxr::Space);

impl Plugin for OxrReferenceSpacePlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource::<OxrSessionResourceCreators>()
            .add_resource_creator(OxrPrimaryReferenceSpaceCreator(
                self.default_primary_ref_space,
                None,
            ));
    }
}
