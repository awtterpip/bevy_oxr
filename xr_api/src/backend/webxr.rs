use std::sync::{
    mpsc::{channel, Sender},
    Mutex,
};

use crate::prelude::*;

use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::js_sys;
use web_sys::{XrFrame, XrInputSource};

mod utils;

use utils::*;

#[derive(Clone)]
pub struct WebXrEntry(web_sys::XrSystem);

impl WebXrEntry {
    pub fn new() -> Self {
        Self(
            web_sys::window()
                .expect("No window available in current environment")
                .navigator()
                .xr(),
        )
    }
}

impl EntryTrait for WebXrEntry {
    fn available_extensions(&self) -> Result<ExtensionSet> {
        Ok(ExtensionSet::default())
    }

    fn create_instance(&self, exts: ExtensionSet) -> Result<Instance> {
        Ok(WebXrInstance {
            entry: self.clone(),
            exts,
        }
        .into())
    }
}

#[derive(Clone)]
pub struct WebXrInstance {
    entry: WebXrEntry,
    exts: ExtensionSet,
}

impl InstanceTrait for WebXrInstance {
    fn entry(&self) -> Entry {
        self.entry.clone().into()
    }

    fn enabled_extensions(&self) -> ExtensionSet {
        self.exts
    }

    fn create_session(&self, _info: SessionCreateInfo) -> Result<Session> {
        Ok(WebXrSession {
            instance: self.clone().into(),
            session: self
                .entry
                .0
                .request_session(web_sys::XrSessionMode::ImmersiveVr)
                .resolve()
                .map_err(|_| XrError::Placeholder)?,
            end_frame_sender: Mutex::default(),
        }
        .into())
    }
}

pub struct WebXrSession {
    instance: Instance,
    session: web_sys::XrSession,
    end_frame_sender: Mutex<Option<Sender<()>>>,
}

impl SessionTrait for WebXrSession {
    fn instance(&self) -> &Instance {
        &self.instance
    }

    fn create_input(&self, bindings: Bindings) -> Result<Input> {
        Ok(WebXrInput {
            devices: self.session.input_sources(),
            bindings,
        }
        .into())
    }

    fn begin_frame(&self) -> Result<(View, View)> {
        let mut end_frame_sender = self.end_frame_sender.lock().unwrap();
        if end_frame_sender.is_some() {
            Err(XrError::Placeholder)?
        }
        let (tx, rx) = channel::<()>();
        let (tx_end, rx_end) = channel::<()>();
        *end_frame_sender = Some(tx_end);
        let on_frame: Closure<dyn FnMut(f64, XrFrame)> =
            Closure::new(move |_time: f64, _frame: XrFrame| {
                tx.send(()).ok();
                rx_end.recv().ok();
            });

        self.session
            .request_animation_frame(on_frame.as_ref().unchecked_ref());

        rx.recv().ok();
        todo!()
    }

    fn end_frame(&self) -> Result<()> {
        let mut end_frame_sender = self.end_frame_sender.lock().unwrap();
        match std::mem::take(&mut *end_frame_sender) {
            Some(sender) => sender.send(()).ok(),
            None => Err(XrError::Placeholder)?,
        };
        Ok(())
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
        todo!()
    }
}

pub struct WebXrInput {
    devices: web_sys::XrInputSourceArray,
    bindings: Bindings,
}

impl From<web_sys::XrHandedness> for Handedness {
    fn from(value: web_sys::XrHandedness) -> Self {
        match value {
            web_sys::XrHandedness::None => Handedness::None,
            web_sys::XrHandedness::Left => Handedness::Left,
            web_sys::XrHandedness::Right => Handedness::Right,
            _ => todo!(),
        }
    }
}

impl WebXrInput {
    fn get_controller(&self, handedness: Handedness) -> Option<web_sys::XrInputSource> {
        js_sys::try_iter(&self.devices).ok()??.find_map(|dev| {
            if let Ok(dev) = dev {
                let dev: XrInputSource = dev.into();
                if Into::<Handedness>::into(dev.handedness()) == handedness {
                    Some(dev)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

impl InputTrait for WebXrInput {
    fn get_haptics(&self, path: ActionPath) -> Result<Action<Haptic>> {
        // let haptics = self
        // .get_controller(path.handedness)
        // .ok_or(XrError::Placeholder)?
        // .gamepad()
        // .ok_or(XrError::Placeholder)?
        // .haptic_actuators()
        // .iter()
        // .next()
        // .ok_or(XrError::Placeholder)?
        // .into();
        // Ok(WebXrHaptics(haptics, path).into())
        todo!()
    }

    fn get_pose(&self, _path: ActionPath) -> Result<Action<Pose>> {
        todo!()
    }

    fn get_float(&self, _path: ActionPath) -> Result<Action<f32>> {
        todo!()
    }

    fn get_bool(&self, _path: ActionPath) -> Result<Action<bool>> {
        todo!()
    }
}

pub struct WebXrHaptics(web_sys::GamepadHapticActuator, ActionPath);

impl ActionTrait for WebXrHaptics {
    fn id(&self) -> ActionPath {
        self.1
    }
}

impl HapticTrait for WebXrHaptics {}
