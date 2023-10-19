use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::{info, App, Startup, Commands, SpatialBundle},
};
use bevy_openxr::{xr_input::{debug_gizmos::OpenXrDebugRenderer, trackers::{OpenXRLeftController, OpenXRController, OpenXRTracker, OpenXRRightController}}, DefaultXrPlugins};

mod setup;
use crate::setup::setup_scene;

fn main() {
    color_eyre::install().unwrap();

    info!("Running bevy_openxr demo");
    App::new()
        //lets get the usual diagnostic stuff added
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        //lets get the xr defaults added
        .add_plugins(DefaultXrPlugins)
        //lets add the debug renderer for the controllers
        .add_plugins(OpenXrDebugRenderer)
        //lets setup the starting scene
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, spawn_controllers_example) //you need to spawn controllers or it crashes TODO:: Fix this
        .run();
}

fn spawn_controllers_example(mut commands: Commands) {
    //left hand
    commands.spawn((
        OpenXRLeftController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        // XRRayInteractor,
        // AimPose(Transform::default()),
        // XRInteractorState::default(),
    ));
    //right hand
    commands.spawn((
        OpenXRRightController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        // XRDirectInteractor,
        // XRInteractorState::default(),
    ));
}

