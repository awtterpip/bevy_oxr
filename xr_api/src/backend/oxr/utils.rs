use glam::Quat;
use openxr::Posef;

use crate::{error::XrError, prelude::Pose};

impl From<openxr::sys::Result> for XrError {
    fn from(_: openxr::sys::Result) -> Self {
        XrError::Placeholder
    }
}

impl From<Posef> for Pose {
    fn from(value: Posef) -> Self {
        let translation = {
            let openxr::Vector3f { x, y, z } = value.position;
            [x, y, z].into()
        };

        let rotation = {
            let openxr::Quaternionf { x, y, z, w } = value.orientation;

            Quat::from_xyzw(x, y, z, w)
        };

        Pose {
            translation,
            rotation,
        }
    }
}
