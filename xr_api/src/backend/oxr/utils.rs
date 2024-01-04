use openxr::Posef;

use crate::{error::XrError, prelude::Pose};

impl From<openxr::sys::Result> for XrError {
    fn from(value: openxr::sys::Result) -> Self {
        XrError::Placeholder
    }
}

impl From<Posef> for Pose {
    fn from(value: Posef) -> Self {
        todo!()
    }
}
