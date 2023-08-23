use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use glam::{Quat, Vec3};
use openxr as xr;

use crate::input::{PostFrameData, XrInput};

pub type XrPose = (Vec3, Quat);
pub const VIEW_TYPE: xr::ViewConfigurationType = xr::ViewConfigurationType::PRIMARY_STEREO;

pub enum XrState {
    Vulkan(XrStateInner<xr::Vulkan>),
}

pub struct XrStateInner<G: xr::Graphics> {
    instance: xr::Instance,
    session: xr::Session<G>,
    session_running: AtomicBool,
    frame: Mutex<FrameInner<G>>,
    frame_state: Mutex<Option<xr::FrameState>>,
    post_frame_data: Mutex<Option<PostFrameData>>,
    event_buffer: UnsafeCell<xr::EventDataBuffer>,
    input: XrInput,
}

unsafe impl<G: xr::Graphics> Sync for XrStateInner<G> {}
unsafe impl<G: xr::Graphics> Send for XrStateInner<G> {}

impl<G: xr::Graphics> XrStateInner<G> {
    pub fn preframe(&self) -> xr::Result<()> {
        let event_buffer = unsafe { &mut *self.event_buffer.get() };
        while let Some(event) = self.instance.poll_event(event_buffer)? {
            use xr::Event::*;
            match event {
                SessionStateChanged(e) => {
                    // Session state change is where we can begin and end sessions, as well as
                    // find quit messages!
                    match e.state() {
                        xr::SessionState::READY => {
                            self.session
                                .begin(VIEW_TYPE)?; // TODO! support other view types
                            self.session_running
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        xr::SessionState::STOPPING => {
                            self.session.end()?;
                            self.session_running
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => {
                            *self.frame_state.lock().unwrap() = None;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                InstanceLossPending(_) => {
                    *self.frame_state.lock().unwrap() = None;
                    return Ok(());
                }
                EventsLost(e) => {}
                _ => {}
            }
        }
        if !self
            .session_running
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            // Don't grind up the CPU
            std::thread::sleep(std::time::Duration::from_millis(10));
            *self.frame_state.lock().unwrap() = None;
            return Ok(());
        }

        *self.frame_state.lock().unwrap() = Some(self.frame.lock().unwrap().begin()?);

        Ok(())
    }

    pub fn post_frame(&self) -> xr::Result<(wgpu::TextureView, wgpu::TextureView)> {
        *self.post_frame_data.lock().unwrap() = Some(self.input.post_frame(self.frame_state.lock().unwrap().unwrap().clone())?);
        Ok(self.frame.lock().unwrap().get_render_views())
    }

    pub fn post_queue_submit(&self) -> xr::Result<()> {
        let pfd = self.post_frame_data.lock().unwrap();
        self.frame.lock().unwrap().post_queue_submit(self.frame_state.lock().unwrap().unwrap().clone(), &(*pfd).clone().unwrap().views, self.input.stage())
    }
}

pub struct FrameInner<G: xr::Graphics> {
    waiter: xr::FrameWaiter,
    stream: xr::FrameStream<G>,
    blend_mode: xr::EnvironmentBlendMode,
    views: Vec<xr::ViewConfigurationView>,
    swapchain: xr::Swapchain<G>,
    resolution: Extent2D,
    buffers: Vec<wgpu::Texture>,
}

impl<G: xr::Graphics> FrameInner<G> {
    fn begin(&mut self) -> xr::Result<xr::FrameState> {
        let frame_state = self.waiter.wait()?;
        self.stream.begin()?;
        Ok(frame_state)
    }

    fn get_render_views(&mut self) -> (wgpu::TextureView, wgpu::TextureView) {
        let image_index = self.swapchain.acquire_image().unwrap();
        self.swapchain.wait_image(xr::Duration::INFINITE).unwrap();

        let texture = &self.buffers[image_index as usize];

        (
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                array_layer_count: Some(1),
                base_array_layer: 0,
                ..Default::default()
            }),
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                array_layer_count: Some(1),
                base_array_layer: 1,
                ..Default::default()
            }),
        )
    }

    fn post_queue_submit(
        &mut self,
        xr_frame_state: xr::FrameState,
        views: &[openxr::View],
        stage: &xr::Space,
    ) -> xr::Result<()> {
        self.swapchain.release_image()?;
        let rect = xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di {
                width: self.resolution.width as _,
                height: self.resolution.height as _,
            },
        };
        self.stream.end(
            xr_frame_state.predicted_display_time,
            self.blend_mode,
            &[&xr::CompositionLayerProjection::new().space(stage).views(&[
                xr::CompositionLayerProjectionView::new()
                    .pose(views[0].pose)
                    .fov(views[0].fov)
                    .sub_image(
                        xr::SwapchainSubImage::new()
                            .swapchain(&self.swapchain)
                            .image_array_index(0)
                            .image_rect(rect),
                    ),
                xr::CompositionLayerProjectionView::new()
                    .pose(views[1].pose)
                    .fov(views[1].fov)
                    .sub_image(
                        xr::SwapchainSubImage::new()
                            .swapchain(&self.swapchain)
                            .image_array_index(1)
                            .image_rect(rect),
                    ),
            ])],
        )?;

        Ok(())
    }
}

pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}
