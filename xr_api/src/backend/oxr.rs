mod graphics;
mod utils;

use std::sync::Mutex;

use glam::{Mat4, UVec2};
use openxr::EnvironmentBlendMode;
use tracing::{info, info_span, warn};

use crate::{backend::oxr::graphics::VIEW_TYPE, prelude::*};

pub struct OXrEntry(openxr::Entry);

impl OXrEntry {
    pub fn new() -> Self {
        #[cfg(feature = "linked")]
        return OXrEntry(openxr::Entry::linked());
        #[cfg(not(feature = "linked"))]
        return OXrEntry(unsafe { openxr::Entry::load().expect("Failed to load OpenXR runtime") });
    }
}

impl Into<openxr::ExtensionSet> for ExtensionSet {
    fn into(self) -> openxr::ExtensionSet {
        let mut set = openxr::ExtensionSet::default();
        set.khr_vulkan_enable2 = self.vulkan;
        set
    }
}

impl EntryTrait for OXrEntry {
    fn available_extensions(&self) -> Result<ExtensionSet> {
        // self.0.enumerate_extensions();
        Ok(ExtensionSet::default())
    }

    fn create_instance(&self, exts: ExtensionSet) -> Result<Instance> {
        #[allow(unused_mut)]
        let mut enabled_extensions: openxr::ExtensionSet = exts.into();
        #[cfg(target_os = "android")]
        {
            enabled_extensions.khr_android_create_instance = true;
        }
        let xr_instance = self.0.create_instance(
            &openxr::ApplicationInfo {
                application_name: "bevy",
                ..Default::default()
            },
            &enabled_extensions,
            &[],
        )?;
        Ok(OXrInstance(xr_instance, exts).into())
    }
}

#[derive(Clone)]
pub struct OXrInstance(openxr::Instance, ExtensionSet);

impl InstanceTrait for OXrInstance {
    fn entry(&self) -> Entry {
        OXrEntry(self.0.entry().clone()).into()
    }

    fn enabled_extensions(&self) -> ExtensionSet {
        self.1
    }

    fn create_session(&self, info: SessionCreateInfo) -> Result<Session> {
        graphics::init_oxr_graphics(self.clone(), self.1, info.texture_format).map(Into::into)
    }
}

pub struct OXrSession {
    pub(crate) instance: Instance,
    // this could definitely be done better
    pub(crate) inner_instance: openxr::Instance,
    pub(crate) session: openxr::Session<openxr::AnyGraphics>,
    pub(crate) render_resources: Mutex<
        Option<(
            wgpu::Device,
            wgpu::Queue,
            wgpu::AdapterInfo,
            wgpu::Adapter,
            wgpu::Instance,
        )>,
    >,
    pub(crate) frame_state: Mutex<openxr::FrameState>,
    pub(crate) views: Mutex<[openxr::View; 2]>,
    pub(crate) frame_waiter: Mutex<openxr::FrameWaiter>,
    pub(crate) swapchain: graphics::Swapchain,
    pub(crate) stage: openxr::Space,
    pub(crate) head: openxr::Space,
    pub(crate) resolution: UVec2,
    pub(crate) blend_mode: EnvironmentBlendMode,
    pub(crate) format: wgpu::TextureFormat,
}

impl SessionTrait for OXrSession {
    fn instance(&self) -> &Instance {
        &self.instance
    }

    fn get_render_resources(
        &self,
    ) -> Option<(
        wgpu::Device,
        wgpu::Queue,
        wgpu::AdapterInfo,
        wgpu::Adapter,
        wgpu::Instance,
    )> {
        std::mem::take(&mut self.render_resources.lock().unwrap())
    }

    fn create_input(&self, bindings: Bindings) -> Result<Input> {
        todo!()
    }

    fn begin_frame(&self) -> Result<(View, View)> {
        {
            let _span = info_span!("xr_poll_events");
            while let Some(event) = self
                .inner_instance
                .poll_event(&mut Default::default())
                .unwrap()
            {
                use openxr::Event::*;
                match event {
                    SessionStateChanged(e) => {
                        // Session state change is where we can begin and end sessions, as well as
                        // find quit messages!
                        info!("entered XR state {:?}", e.state());
                        match e.state() {
                            openxr::SessionState::READY => {
                                self.session.begin(VIEW_TYPE).unwrap();
                            }
                            openxr::SessionState::STOPPING => {
                                self.session.end().unwrap();
                            }
                            openxr::SessionState::EXITING | openxr::SessionState::LOSS_PENDING => {
                                return Err(XrError::Placeholder);
                            }
                            _ => {}
                        }
                    }
                    InstanceLossPending(_) => return Err(XrError::Placeholder),
                    EventsLost(e) => {
                        warn!("lost {} XR events", e.lost_event_count());
                    }
                    _ => {}
                }
            }
        }
        {
            let _span = info_span!("xr_wait_frame").entered();
            *self.frame_state.lock().unwrap() = match self.frame_waiter.lock().unwrap().wait() {
                Ok(a) => a,
                Err(e) => {
                    warn!("error: {}", e);
                    return Err(XrError::Placeholder);
                }
            };
        }
        {
            let _span = info_span!("xr_begin_frame").entered();
            self.swapchain.begin().unwrap()
        }

        {
            let _span = info_span!("xr_acquire_image").entered();
            self.swapchain.acquire_image().unwrap()
        }
        {
            let _span = info_span!("xr_wait_image").entered();
            self.swapchain.wait_image().unwrap();
        }
        let views = {
            let _span = info_span!("xr_locate_views").entered();
            self.session
                .locate_views(
                    VIEW_TYPE,
                    self.frame_state.lock().unwrap().predicted_display_time,
                    &self.stage,
                )
                .unwrap()
                .1
        };
        *self.views.lock().unwrap() = [views[0].clone(), views[1].clone()];
        {
            let _span = info_span!("xr_update_manual_texture_views").entered();
            let (left, right) = self.swapchain.get_render_views();
            let left = OXrView {
                texture: Mutex::new(Some(left)),
                view: views[0],
                resolution: self.resolution,
                format: self.format,
            };
            let right = OXrView {
                texture: Mutex::new(Some(right)),
                view: views[1],
                resolution: self.resolution,
                format: self.format,
            };
            Ok((left.into(), right.into()))
        }
    }

    fn end_frame(&self) -> Result<()> {
        {
            let _span = info_span!("xr_release_image").entered();
            self.swapchain.release_image().unwrap();
        }
        {
            let _span = info_span!("xr_end_frame").entered();
            let frame_state = self.frame_state.lock().unwrap().clone();
            let result = self.swapchain.end(
                frame_state.predicted_display_time,
                &*self.views.lock().unwrap(),
                &self.stage,
                self.resolution,
                self.blend_mode,
                frame_state.should_render,
                // passthrough_layer.map(|p| p.into_inner()),
            );
            match result {
                Ok(_) => {}
                Err(e) => warn!("error: {}", e),
            }
        }
        Ok(())
    }

    fn resolution(&self) -> UVec2 {
        self.resolution
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
}

pub struct OXrView {
    texture: Mutex<Option<wgpu::TextureView>>,
    view: openxr::View,
    resolution: UVec2,
    format: wgpu::TextureFormat,
}

impl ViewTrait for OXrView {
    fn texture_view(&self) -> Option<wgpu::TextureView> {
        std::mem::take(&mut *self.texture.lock().unwrap())
    }

    fn pose(&self) -> Pose {
        self.view.pose.clone().into()
    }

    fn projection_matrix(&self) -> glam::Mat4 {
        //  symmetric perspective for debugging
        // let x_fov = (self.fov.angle_left.abs() + self.fov.angle_right.abs());
        // let y_fov = (self.fov.angle_up.abs() + self.fov.angle_down.abs());
        // return Mat4::perspective_infinite_reverse_rh(y_fov, x_fov / y_fov, self.near);

        let fov = self.view.fov;
        let is_vulkan_api = false; // FIXME wgpu probably abstracts this
        let near_z = 0.1;
        let far_z = -1.; //   use infinite proj
                         // let far_z = self.far;

        let tan_angle_left = fov.angle_left.tan();
        let tan_angle_right = fov.angle_right.tan();

        let tan_angle_down = fov.angle_down.tan();
        let tan_angle_up = fov.angle_up.tan();

        let tan_angle_width = tan_angle_right - tan_angle_left;

        // Set to tanAngleDown - tanAngleUp for a clip space with positive Y
        // down (Vulkan). Set to tanAngleUp - tanAngleDown for a clip space with
        // positive Y up (OpenGL / D3D / Metal).
        // const float tanAngleHeight =
        //     graphicsApi == GRAPHICS_VULKAN ? (tanAngleDown - tanAngleUp) : (tanAngleUp - tanAngleDown);
        let tan_angle_height = if is_vulkan_api {
            tan_angle_down - tan_angle_up
        } else {
            tan_angle_up - tan_angle_down
        };

        // Set to nearZ for a [-1,1] Z clip space (OpenGL / OpenGL ES).
        // Set to zero for a [0,1] Z clip space (Vulkan / D3D / Metal).
        // const float offsetZ =
        //     (graphicsApi == GRAPHICS_OPENGL || graphicsApi == GRAPHICS_OPENGL_ES) ? nearZ : 0;
        // FIXME handle enum of graphics apis
        let offset_z = 0.;

        let mut cols: [f32; 16] = [0.0; 16];

        if far_z <= near_z {
            // place the far plane at infinity
            cols[0] = 2. / tan_angle_width;
            cols[4] = 0.;
            cols[8] = (tan_angle_right + tan_angle_left) / tan_angle_width;
            cols[12] = 0.;

            cols[1] = 0.;
            cols[5] = 2. / tan_angle_height;
            cols[9] = (tan_angle_up + tan_angle_down) / tan_angle_height;
            cols[13] = 0.;

            cols[2] = 0.;
            cols[6] = 0.;
            cols[10] = -1.;
            cols[14] = -(near_z + offset_z);

            cols[3] = 0.;
            cols[7] = 0.;
            cols[11] = -1.;
            cols[15] = 0.;

            //  bevy uses the _reverse_ infinite projection
            //  https://dev.theomader.com/depth-precision/
            let z_reversal = Mat4::from_cols_array_2d(&[
                [1f32, 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., -1., 0.],
                [0., 0., 1., 1.],
            ]);

            return z_reversal * Mat4::from_cols_array(&cols);
        } else {
            // normal projection
            cols[0] = 2. / tan_angle_width;
            cols[4] = 0.;
            cols[8] = (tan_angle_right + tan_angle_left) / tan_angle_width;
            cols[12] = 0.;

            cols[1] = 0.;
            cols[5] = 2. / tan_angle_height;
            cols[9] = (tan_angle_up + tan_angle_down) / tan_angle_height;
            cols[13] = 0.;

            cols[2] = 0.;
            cols[6] = 0.;
            cols[10] = -(far_z + offset_z) / (far_z - near_z);
            cols[14] = -(far_z * (near_z + offset_z)) / (far_z - near_z);

            cols[3] = 0.;
            cols[7] = 0.;
            cols[11] = -1.;
            cols[15] = 0.;
        }

        Mat4::from_cols_array(&cols)
    }

    fn resolution(&self) -> UVec2 {
        self.resolution
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
}

pub struct OXrInput {
    action_set: openxr::ActionSet,
}

impl InputTrait for OXrInput {
    fn get_haptics(&self, path: ActionPath) -> Result<Action<Haptic>> {
        todo!()
    }

    fn get_pose(&self, path: ActionPath) -> Result<Action<Pose>> {
        todo!()
    }

    fn get_float(&self, path: ActionPath) -> Result<Action<f32>> {
        todo!()
    }

    fn get_bool(&self, path: ActionPath) -> Result<Action<bool>> {
        todo!()
    }
}
