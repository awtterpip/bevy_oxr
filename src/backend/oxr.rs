use std::rc::Rc;
use std::sync::Mutex;

use bevy::math::UVec2;
use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};

use bevy::utils::HashMap;
use openxr::{
    ActionSet, AnyGraphics, ApplicationInfo, Entry, EnvironmentBlendMode, ExtensionSet, Instance,
    Session,
};

use super::graphics::OXrGraphics;
use super::traits::*;
use super::{oxr_utils::*, XrControllerInner};

use crate::backend::graphics::init_oxr_graphics;
use crate::error::XrError;
use crate::resources::*;
use crate::types::*;

pub struct OXrEntry(Entry);

impl XrEntryTrait for OXrEntry {
    fn get_xr_entry(&self) -> Result<XrEntry, XrError> {
        Ok(OXrEntry(xr_entry()?).into())
    }

    fn get_available_features(&self) -> Result<FeatureList, XrError> {
        let available_extensions = self.0.enumerate_extensions()?;
        let mut feature_list = FeatureList::default();
        feature_list.graphics.vulkan = available_extensions.khr_vulkan_enable2;
        Ok(feature_list)
    }

    fn create_instance(&self, features: FeatureList) -> Result<XrInstance, XrError> {
        let mut enabled_extensions = ExtensionSet::default();
        enabled_extensions.khr_vulkan_enable2 = features.graphics.vulkan;
        let instance = self.0.create_instance(
            &ApplicationInfo {
                application_name: "Ambient",
                ..Default::default()
            },
            &enabled_extensions,
            &[],
        )?;

        Ok(OXrInstance {
            enabled_features: features,
            inner: instance,
        }
        .into())
    }
}

pub struct OXrInstance {
    inner: Instance,
    enabled_features: FeatureList,
}

impl XrInstanceTrait for OXrInstance {
    fn requested_features(&self) -> FeatureList {
        self.enabled_features
    }

    fn create_session(&self, info: SessionCreateInfo) -> Result<XrSession, XrError> {
        Ok(init_oxr_graphics(
            self.inner.clone(),
            self.enabled_features.graphics,
            info.window,
        )?
        .into())
    }
}

pub struct OXrSession {
    pub(crate) instance: Instance,
    pub(crate) graphics: OXrGraphics,
    pub(crate) device: RenderDevice,
    pub(crate) queue: RenderQueue,
    pub(crate) adapter: RenderAdapter,
    pub(crate) adapter_info: RenderAdapterInfo,
    pub(crate) session: Session<AnyGraphics>,
    pub(crate) blend_mode: EnvironmentBlendMode,
    pub(crate) resolution: UVec2,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) buffers: Vec<wgpu::Texture>,
    pub(crate) image_index: Mutex<usize>,
}

impl XrSessionTrait for OXrSession {
    fn sync_controllers(&self, (left, _): (XrController, XrController)) -> Result<(), XrError> {
        let XrControllerInner::OpenXR(controller) = &**left;
        self.session
            .sync_actions(&[(&controller.action_set).into()])?;
        Ok(())
    }

    fn get_reference_space(&self, info: ReferenceSpaceInfo) -> Result<XrReferenceSpace, XrError> {
        todo!()
    }

    fn create_controllers(
        &self,
        info: ActionSetCreateInfo,
    ) -> Result<(XrController, XrController), XrError> {
        let instance = &self.instance;
        let action_set = instance.create_action_set("controllers", "XR Controllers", 0)?;
        let left_path = instance.string_to_path("/user/hand/left").unwrap();
        let right_path = instance.string_to_path("/user/hand/right").unwrap();
        let hand_subpaths = &[left_path, right_path];
        use ActionPath::*;
        let actions = &[
            HandPose,
            PointerPose,
            GripPull,
            TriggerPull,
            TriggerTouch,
            HapticFeedback,
            PrimaryButton,
            PrimaryButtonTouch,
            SecondaryButton,
            SecondaryButtonTouch,
            MenuButton,
            ThumbstickX,
            ThumbstickY,
            ThumbstickTouch,
            ThumbstickClick,
            ThumbrestTouch,
        ];
        let action_map = Rc::new(create_actions(&action_set, actions, hand_subpaths)?);
        Ok((
            OXrController {
                action_set: action_set.clone(),
                actions: action_map.clone(),
                side: Side::Left,
            }
            .into(),
            OXrController {
                action_set,
                actions: action_map,
                side: Side::Right,
            }
            .into(),
        ))
    }
}

pub struct OXrAction {}

pub struct OXrController {
    action_set: ActionSet,
    actions: Rc<HashMap<ActionPath, TypedAction>>,
    side: Side,
}

pub struct OXrActionSpace {}

pub struct OXrReferenceSpace {}
