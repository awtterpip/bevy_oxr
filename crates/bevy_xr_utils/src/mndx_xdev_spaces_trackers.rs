use std::convert::identity;

use bevy_app::{App, Plugin, PreUpdate};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{component::Component, entity::Entity, query::With, resource::Resource, schedule::{IntoScheduleConfigs as _, common_conditions::resource_exists}, system::{Commands, Query, Res, ResMut}};
use bevy_log::{error, info};
use bevy_mod_openxr::{
    resources::{OxrInstance, OxrSystemId},
    session::OxrSession,
    spaces::OxrSpaceExt,
};
use bevy_mod_xr::{
    session::{XrPreDestroySession, XrSessionCreated},
    spaces::XrSpace,
};
use openxr_mndx_xdev_space::{InstanceXDevExtensionMNDX, SessionXDevExtensionMNDX, XDev, XDevList};

use crate::generic_tracker::GenericTracker;

pub struct MonadoXDevSpacesPlugin;
impl Plugin for MonadoXDevSpacesPlugin {
    fn build(&self, _app: &mut App) {}
    fn finish(&self, app: &mut App) {
        let Some((instance, system_id)) =
            app.world()
                .get_resource::<OxrInstance>()
                .and_then(|instance| {
                    app.world()
                        .get_resource::<OxrSystemId>()
                        .map(|system_id| (instance, system_id))
                })
        else {
            return;
        };
        if !instance
            .supports_mndx_xdev_spaces(**system_id)
            .is_ok_and(identity)
        {
            return;
        }
        app.add_systems(XrSessionCreated, session_created);
        app.add_systems(
            PreUpdate,
            update_xdev_list.run_if(resource_exists::<PrimaryXDevList>),
        );
        app.add_systems(
            XrPreDestroySession,
            (despawn_xdev_trackers, |mut cmds: Commands| {
                cmds.remove_resource::<PrimaryXDevList>()
            }),
        );
    }
}

fn update_xdev_list(mut xdev_list: ResMut<PrimaryXDevList>, mut cmds: Commands) {
    let Ok(new_gen) = xdev_list
        .get_generation()
        .inspect_err(|err| error!("unable to get xdev list generation: {err}"))
    else {
        return;
    };
    if new_gen != xdev_list.generation {
        xdev_list.generation = new_gen;
        cmds.run_system_cached(despawn_xdev_trackers);
        cmds.run_system_cached(create_xdev_trackers);
    }
}

fn session_created(session: Res<OxrSession>, mut cmds: Commands) {
    let list = match session.get_xdev_list() {
        Ok(v) => v,
        Err(err) => {
            error!("unable to create xdev list: {err}");
            return;
        }
    };
    cmds.insert_resource(PrimaryXDevList {
        generation: list.get_generation().unwrap(),
        list,
    });
    cmds.run_system_cached(despawn_xdev_trackers);
    cmds.run_system_cached(create_xdev_trackers);
}

fn despawn_xdev_trackers(
    xdev_query: Query<(Entity, &XrSpace), With<XDevTracker>>,
    mut cmds: Commands,
    session: Res<OxrSession>,
) {
    for (e, space) in &xdev_query {
        cmds.entity(e).despawn();
        if let Err(err) = session.destroy_space(*space) {
            error!("unable to destroy xdev XrSpace: {err}");
        };
    }
}

fn create_xdev_trackers(xdev_list: Res<PrimaryXDevList>, mut cmds: Commands) {
    let xdevs = match xdev_list.enumerate_xdevs() {
        Err(err) => {
            error!("Unable to enumerate xdevs: {err}");
            return;
        }
        Ok(v) => v,
    };
    for xdev in xdevs
        .into_iter()
        .filter(XDev::can_create_space)
        .filter(|v| v.name().contains("Tracker"))
    {
        info!("new XDev Tracker: {}", xdev.name());
        let xr_space =
            XrSpace::from_openxr_space(xdev.create_space(openxr::Posef::IDENTITY).unwrap());
        cmds.spawn((xr_space, GenericTracker, XDevTracker));
    }
}

#[derive(Clone, Copy, Component, Debug)]
pub struct XDevTracker;
#[derive(Deref, DerefMut, Resource)]
pub struct PrimaryXDevList {
    #[deref]
    pub list: XDevList,
    pub generation: u64,
}
