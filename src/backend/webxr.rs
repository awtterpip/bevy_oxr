use web_sys::XrRenderStateInit;
use web_sys::XrWebGlLayer;

use super::traits::*;
use super::web_utils::*;

use crate::error::XrError;
use crate::resources::*;
use crate::types::*;

pub const BEVY_CANVAS_ID: &str = "bevy_canvas";
pub const BEVY_CANVAS_QUERY: &str = "canvas[data-bevy-webxr=\"bevy_canvas\"]";

pub struct WebXrEntry(web_sys::XrSystem);

impl XrEntryTrait for WebXrEntry {
    fn get_xr_entry(&self) -> Result<XrEntry, XrError> {
        if let Some(window) = web_sys::window() {
            Ok(WebXrEntry(window.navigator().xr()).into())
        } else {
            Err(XrError {})
        }
    }

    fn get_available_features(&self) -> Result<FeatureList, XrError> {
        Ok(FeatureList::default())
    }

    fn create_instance(&self, features: FeatureList) -> Result<XrInstance, XrError> {
        Ok(WebXrInstance {
            xr: self.0.clone(),
            features,
        }
        .into())
    }
}

pub struct WebXrInstance {
    xr: web_sys::XrSystem,
    features: FeatureList,
}

impl XrInstanceTrait for WebXrInstance {
    fn requested_features(&self) -> FeatureList {
        self.features
    }

    fn create_session(&self, info: SessionCreateInfo) -> Result<XrSession, XrError> {
        let session = self
            .xr
            .request_session(web_sys::XrSessionMode::ImmersiveVr)
            .resolve()?;

        let gl = create_webgl_context(true, &info.canvas.expect("Expected canvas string"))?;
        let xr_gl_layer = XrWebGlLayer::new_with_web_gl2_rendering_context(&session, &gl)?;
        let mut render_state_init = XrRenderStateInit::new();
        render_state_init.base_layer(Some(&xr_gl_layer));
        session.update_render_state_with_state(&render_state_init);

        Ok(WebXrSession(session).into())
    }
}

pub struct WebXrSession(web_sys::XrSession);

impl XrSessionTrait for WebXrSession {
    fn sync_actions(&self, action_set: XrActionSet) {
        todo!()
    }

    fn get_reference_space(&self, _info: ReferenceSpaceInfo) -> Result<XrReferenceSpace, XrError> {
        let space = self
            .0
            .request_reference_space(web_sys::XrReferenceSpaceType::BoundedFloor)
            .resolve()?;
        Ok(WebXrReferenceSpace(space).into())
    }

    fn create_action_set(&self, info: ActionSetCreateInfo) -> Result<XrActionSet, XrError> {
        todo!()
    }
}

pub struct WebXrActionSet(Vec<XrAction>);

pub struct WebXrAction(web_sys::XrInputSource);

pub struct WebXrActionSpace(web_sys::XrJointSpace);

pub struct WebXrReferenceSpace(web_sys::XrBoundedReferenceSpace);
