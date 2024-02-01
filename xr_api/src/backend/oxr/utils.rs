use glam::Quat;
use openxr::Posef;

use crate::{
    error::XrError,
    path::{input, Handed, InputId, PathComponent, UntypedActionPath},
    prelude::Pose,
};

use super::Bindings;

impl From<openxr::sys::Result> for XrError {
    fn from(_: openxr::sys::Result) -> Self {
        XrError::Placeholder
    }
}

impl From<Posef> for Pose {
    fn from(pose: Posef) -> Self {
        // with enough sign errors anything is possible
        let rotation = {
            let o = pose.orientation;
            Quat::from_rotation_x(180.0f32.to_radians()) * glam::quat(o.w, o.z, o.y, o.x)
        };
        let translation = glam::vec3(-pose.position.x, pose.position.y, -pose.position.z);

        Pose {
            translation,
            rotation,
        }
    }
}

impl UntypedActionPath {
    pub(crate) fn into_xr_path(self) -> String {
        let dev_path;
        let sub_path;
        let comp_path = match self.comp {
            PathComponent::Click => "/click",
            PathComponent::Touch => "/touch",
            PathComponent::Value => "/value",
            PathComponent::X => "/x",
            PathComponent::Y => "/y",
            PathComponent::Pose => "/pose",
            PathComponent::Haptic => "/haptic",
        };
        match self.input {
            InputId::Left(hand) => {
                dev_path = "/user/hand/left";
                sub_path = match hand {
                    Handed::PrimaryButton => "/input/x",
                    Handed::SecondaryButton => "/input/y",
                    Handed::Select => "/input/select",
                    Handed::Menu => "/input/menu",
                    Handed::Thumbstick => "/input/thumbstick",
                    Handed::Trigger => "/input/trigger",
                    Handed::Grip if matches!(self.comp, PathComponent::Pose) => "/input/grip",
                    Handed::Grip => "/input/squeeze",
                    Handed::Output => "/output",
                };
            }
            InputId::Right(hand) => {
                dev_path = "/user/hand/right";
                sub_path = match hand {
                    Handed::PrimaryButton => "/input/a",
                    Handed::SecondaryButton => "/input/b",
                    Handed::Select => "/input/select",
                    Handed::Menu => "/input/menu",
                    Handed::Thumbstick => "/input/thumbstick",
                    Handed::Trigger => "/input/trigger",
                    Handed::Grip if matches!(self.comp, PathComponent::Pose) => "/input/grip",
                    Handed::Grip => "/input/squeeze",
                    Handed::Output => "/output",
                };
            }
            InputId::Head(head) => {
                use input::head::Head;
                dev_path = "/user/head";
                sub_path = match head {
                    Head::VolumeUp => "/input/volume_up",
                    Head::VolumeDown => "/input/volume_down",
                    Head::MuteMic => "/input/mute_mic",
                };
            }
        };

        let mut path = dev_path.to_owned();
        path.push_str(sub_path);
        path.push_str(comp_path);
        path
    }

    pub(crate) fn into_name(&self) -> String {
        let comp_path = match self.comp {
            PathComponent::Click => "_click",
            PathComponent::Touch => "_touch",
            PathComponent::Value => "_value",
            PathComponent::X => "_x",
            PathComponent::Y => "_y",
            PathComponent::Pose => "_pose",
            PathComponent::Haptic => "",
        };
        let dev_path = match self.input {
            InputId::Left(hand) => match hand {
                Handed::PrimaryButton => "left_primary_button",
                Handed::SecondaryButton => "left_secondary_button",
                Handed::Select => "left_select",
                Handed::Menu => "left_menu",
                Handed::Thumbstick => "left_thumbstick",
                Handed::Trigger => "left_trigger",
                Handed::Grip => "left_grip",
                Handed::Output => "left_output",
            },
            InputId::Right(hand) => match hand {
                Handed::PrimaryButton => "right_primary_button",
                Handed::SecondaryButton => "right_secondary_button",
                Handed::Select => "right_select",
                Handed::Menu => "right_menu",
                Handed::Thumbstick => "right_thumbstick",
                Handed::Trigger => "right_trigger",
                Handed::Grip => "right_grip",
                Handed::Output => "right_output",
            },
            InputId::Head(head) => {
                use input::head::Head;
                match head {
                    Head::VolumeUp => "volume_up",
                    Head::VolumeDown => "volume_down",
                    Head::MuteMic => "mute_mic",
                }
            }
        };
        let mut path = dev_path.to_string();
        path.push_str(comp_path);
        path
    }
}

impl Bindings {
    pub(crate) fn get_interaction_profile(&self) -> &'static str {
        match self {
            Bindings::OculusTouch => "/interaction_profiles/oculus/touch_controller",
        }
    }
}
