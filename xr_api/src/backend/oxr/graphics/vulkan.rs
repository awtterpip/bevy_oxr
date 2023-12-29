use crate::backend::oxr::OXrSession;
use crate::error::Result;

pub fn init_oxr_graphics(
    instance: openxr::Instance,
    format: wgpu::TextureFormat,
) -> Result<OXrSession> {
    todo!()
}
