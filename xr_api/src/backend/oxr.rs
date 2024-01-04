mod graphics;
mod utils;

use std::sync::Mutex;

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
    pub(crate) frame_waiter: Mutex<openxr::FrameWaiter>,
    pub(crate) swapchain: graphics::Swapchain,
    pub(crate) stage: openxr::Space,
    pub(crate) head: openxr::Space,
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

        {
            let _span = info_span!("xr_acquire_image").entered();
            self.swapchain.acquire_image().unwrap()
        }
        {
            let _span = info_span!("xr_wait_image").entered();
            self.swapchain.wait_image().unwrap();
        }
        {
            let _span = info_span!("xr_update_manual_texture_views").entered();
            let (left, right) = self.swapchain.get_render_views();
            let left = OXrView {
                texture: Mutex::new(Some(left)),
                view: views[0],
            };
            let right = OXrView {
                texture: Mutex::new(Some(right)),
                view: views[1],
            };
            Ok((left.into(), right.into()))
        }
    }

    fn end_frame(&self) -> Result<()> {
        todo!()
    }
}

pub struct OXrView {
    texture: Mutex<Option<wgpu::TextureView>>,
    view: openxr::View,
}

impl ViewTrait for OXrView {
    fn texture_view(&self) -> wgpu::TextureView {
        std::mem::take(&mut *self.texture.lock().unwrap()).unwrap()
    }

    fn pose(&self) -> Pose {
        self.view.pose.clone().into()
    }

    fn projection_matrix(&self) -> glam::Mat4 {
        todo!()
    }
}

pub struct OXrInput {
    action_set: openxr::ActionSet,
}
