use std::sync::{Arc, Mutex};

use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::settings::WgpuSettings;
use bevy::render::FutureRendererResources;
use bevy::window::{PrimaryWindow, RawHandleWrapper};
use state::XrState;

mod graphics;
mod state;
mod swapchain;

pub struct OpenXrPlugin {
    pub wgpu_settings: WgpuSettings,
}

#[derive(Resource)]
struct FutureXrResources (
    Arc<
        Mutex<
            Option<
                XrState
            >
        >
    >
);

impl Plugin for OpenXrPlugin {
    fn build(&self, app: &mut App) {
        if let Some(backends) = self.wgpu_settings.backends {
            let future_renderer_resources_wrapper = Arc::new(Mutex::new(None));
            let future_xr_resources_wrapper = Arc::new(Mutex::new(None));
            app.insert_resource(FutureRendererResources(
                future_renderer_resources_wrapper.clone(),
            ));

            app.insert_resource(FutureXrResources(
                future_xr_resources_wrapper.clone(),
            ));

            let mut system_state: SystemState<Query<&RawHandleWrapper, With<PrimaryWindow>>> =
                SystemState::new(&mut app.world);
            let primary_window = system_state.get(&app.world).get_single().ok().cloned();

            let settings = self.wgpu_settings.clone();
            bevy::tasks::IoTaskPool::get()
                .spawn_local(async move {
                    
                })
                .detach();
        }
    }
}
