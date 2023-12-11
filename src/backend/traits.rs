use crate::error::XrError;
use crate::resources::*;
use crate::types::*;

use bevy::render::primitives::Aabb;

pub trait XrEntryTrait {
    fn get_xr_entry(&self) -> Result<XrEntry, XrError>;
    fn get_available_features(&self) -> Result<FeatureList, XrError>;
    fn create_instance(&self, features: FeatureList) -> Result<XrInstance, XrError>;
}

pub trait XrInstanceTrait {
    fn requested_features(&self) -> FeatureList;
    fn create_session(&self, info: SessionCreateInfo) -> Result<XrSession, XrError>;
}

pub trait XrSessionTrait {
    fn sync_controllers(&self, controllers: (XrController, XrController)) -> Result<(), XrError>;
    fn get_reference_space(&self, info: ReferenceSpaceInfo) -> Result<XrReferenceSpace, XrError>;
    fn create_controllers(
        &self,
        info: ActionSetCreateInfo,
    ) -> Result<(XrController, XrController), XrError>;
}

pub trait XrControllerTrait {
    fn get_action_space(&self, info: ActionSpaceInfo) -> Result<XrActionSpace, XrError>;
    fn get_action(&self, id: ActionId) -> Option<XrInput>;
}

pub trait XrInputTrait {
    fn get_action_state(&self) -> ActionState;
    fn get_action_bool(&self) -> Option<bool> {
        if let ActionState::Bool(b) = self.get_action_state() {
            Some(b)
        } else {
            None
        }
    }
    fn get_action_float(&self) -> Option<f32> {
        if let ActionState::Float(f) = self.get_action_state() {
            Some(f)
        } else {
            None
        }
    }
    fn get_action_haptic(&self) -> Option<Haptics> {
        if let ActionState::Haptics(h) = self.get_action_state() {
            Some(h)
        } else {
            None
        }
    }
}

pub trait XrActionSpaceTrait {
    fn locate(&self, base: XrReferenceSpace) -> Pose;
}

pub trait XrReferenceSpaceTrait {
    fn bounds(&self) -> Aabb;
}
