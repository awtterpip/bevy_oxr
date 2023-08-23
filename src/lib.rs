pub mod input;
pub mod resource_macros;
pub mod resources;

use std::sync::{Arc, Mutex};

use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::settings::WgpuSettings;
use bevy::render::{FutureRendererResources, RenderPlugin, renderer};
use bevy::window::{PrimaryWindow, RawHandleWrapper};
use input::XrInput;
use resources::*;

/// Adds OpenXR support to an App
#[derive(Default)]
pub struct OpenXrPlugin;

#[derive(Resource)]
pub struct FutureXrResources(
    pub Arc<
        Mutex<
            Option<(
                XrInstance,
                XrSession,
                XrEnvironmentBlendMode,
                XrSessionRunning,
                XrFrameWaiter,
                XrSwapchain,
                XrInput,
            )>,
        >,
    >,
);

impl Plugin for OpenXrPlugin {
    fn build(&self, app: &mut App) {
        let future_renderer_resources_wrapper = Arc::new(Mutex::new(None));
        app.insert_resource(FutureRendererResources(
            future_renderer_resources_wrapper.clone(),
        ));

        let future_xr_resources_wrapper = Arc::new(Mutex::new(None));
        app.insert_resource(FutureXrResources(
            future_xr_resources_wrapper.clone()
        ));

        let mut system_state: SystemState<Query<&RawHandleWrapper, With<PrimaryWindow>>> =
            SystemState::new(&mut app.world);
        let primary_window = system_state.get(&app.world).get_single().ok().cloned();

        let settings = WgpuSettings::default();
        bevy::tasks::IoTaskPool::get()
            .spawn_local(async move {
                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: settings.backends.unwrap(),
                    dx12_shader_compiler: settings.dx12_shader_compiler.clone(),
                });
                let surface = primary_window.map(|wrapper| unsafe {
                    // SAFETY: Plugins should be set up on the main thread.
                    let handle = wrapper.get_handle();
                    instance
                        .create_surface(&handle)
                        .expect("Failed to create wgpu surface")
                });

                let request_adapter_options = wgpu::RequestAdapterOptions {
                    power_preference: settings.power_preference,
                    compatible_surface: surface.as_ref(),
                    ..Default::default()
                };

                let (device, queue, adapter_info, render_adapter) =
                    renderer::initialize_renderer(&instance, &settings, &request_adapter_options)
                        .await;
                debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
                debug!("Configured wgpu adapter Features: {:#?}", device.features());
                let mut future_renderer_resources_inner =
                    future_renderer_resources_wrapper.lock().unwrap();
                *future_renderer_resources_inner =
                    Some((device, queue, adapter_info, render_adapter, instance));
            })
            .detach();
    }

    fn ready(&self, app: &App) -> bool {
        app.world
            .get_resource::<FutureXrResources>()
            .and_then(|frr| frr.0.try_lock().map(|locked| locked.is_some()).ok())
            .unwrap_or(true)
    }

    fn finish(&self, app: &mut App) {
        if let Some(future_renderer_resources) =
            app.world.remove_resource::<FutureXrResources>()
        {
            let (instance, session, blend_mode, session_running, frame_waiter, swapchain, input) =
                future_renderer_resources.0.lock().unwrap().take().unwrap();

            app.insert_resource(instance.clone())
                .insert_resource(session.clone())
                .insert_resource(blend_mode.clone())
                .insert_resource(session_running.clone())
                .insert_resource(frame_waiter.clone())
                .insert_resource(swapchain.clone());
        }
    }
}

pub struct DefaultXrPlugins;

impl PluginGroup for DefaultXrPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        DefaultPlugins
            .build()
            .add_before::<RenderPlugin, _>(OpenXrPlugin)
    }
}
