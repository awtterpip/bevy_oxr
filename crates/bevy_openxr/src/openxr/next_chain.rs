use openxr::sys;
use std::{ffi::c_void, ptr};

/// An abstraction for the next pointer fields for openxr calls
#[derive(Default)]
pub struct OxrNextChain {
    structs: Vec<Box<dyn OxrNextChainStructProvider>>,
}

impl OxrNextChain {
    pub fn push<T: OxrNextChainStructProvider>(&mut self, info_struct: T) {
        if let Some(last) = self.structs.last_mut() {
            let mut info = Box::new(info_struct);
            info.as_mut().clear_next();
            last.as_mut().set_next(info.as_ref().header());
            self.structs.push(info);
        } else {
            let mut info_struct = Box::new(info_struct);
            info_struct.as_mut().clear_next();
            self.structs.push(info_struct);
        }
    }
    pub fn chain(&self) -> Option<&OxrNextChainStructBase> {
        self.structs.first().map(|v| v.as_ref().header())
    }
    pub fn chain_pointer(&self) -> *const c_void {
        self.chain()
            .map(|v| v as *const _ as *const c_void)
            .unwrap_or(ptr::null())
    }
}

pub trait OxrNextChainStructProvider: 'static {
    fn header(&self) -> &OxrNextChainStructBase;
    fn set_next(&mut self, next: &OxrNextChainStructBase);
    fn clear_next(&mut self);
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct OxrNextChainStructBase {
    pub ty: sys::StructureType,
    pub next: *const OxrNextChainStructBase,
}
