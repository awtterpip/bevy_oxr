use crate::error::XrError;

impl From<openxr::sys::Result> for XrError {
    fn from(value: openxr::sys::Result) -> Self {
        XrError::Placeholder
    }
}
