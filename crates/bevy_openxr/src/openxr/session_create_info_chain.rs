use std::{ffi::c_void, mem, ptr};

use bevy::log::info;
use openxr::{sys, OverlaySessionCreateFlagsEXTX};

#[derive(Default)]
pub struct OxrSessionCreateInfoChain {
    structs: Vec<Box<dyn AsAdditionalSessionCreateInfo>>,
}

impl OxrSessionCreateInfoChain {
    pub fn push<T: AsAdditionalSessionCreateInfo>(&mut self, info_struct: T) {
        if let Some(last) = self.structs.last_mut() {
            info!("pushing info struct and adding it to the chain");
            let mut info = Box::new(info_struct);
            info.as_mut().clear_next();
            last.as_mut().set_next(info.as_ref().header());
            self.structs.push(info);
        } else {
            info!("pushing info struct");
            let mut info_struct = Box::new(info_struct);
            info_struct.as_mut().clear_next();
            self.structs.push(info_struct);
        }
    }
    pub fn chain(&self) -> Option<&AdditionalSessionCreateInfo> {
        self.structs.first().map(|v| v.as_ref().header())
    }
    pub fn chain_pointer(&self) -> *const c_void {
        self.chain()
            .map(|v| v as *const _ as *const c_void)
            .unwrap_or(ptr::null())
    }
}

pub trait AsAdditionalSessionCreateInfo: 'static {
    fn header(&self) -> &AdditionalSessionCreateInfo;
    fn set_next(&mut self, next: &AdditionalSessionCreateInfo);
    fn clear_next(&mut self);
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AdditionalSessionCreateInfo {
    pub ty: sys::StructureType,
    pub next: *const AdditionalSessionCreateInfo,
}

