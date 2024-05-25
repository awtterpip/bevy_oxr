use std::{ffi::c_void, mem, ptr};

use bevy::log::info;
use openxr::{sys, OverlaySessionCreateFlagsEXTX};

#[derive(Default, Clone)]
pub struct OxrSessionCreateInfoChain {
    t: Option<sys::SessionCreateInfoOverlayEXTX>,
    // structs: Vec<&'static mut dyn AsAdditionalSessionCreateInfo>,
}

impl OxrSessionCreateInfoChain {
    // pub fn push<T: AsAdditionalSessionCreateInfo>(&mut self, info_struct: T) {
    //     if let Some(last) = self.structs.last_mut() {
    //         info!("pushing info struct and adding it to the chain");
    //         let info = Box::leak(Box::new(info_struct));
    //         last.set_next(info.header());
    //         self.structs.push(info);
    //     } else {
    //         info!("pushing info struct");
    //         let info_struct = Box::leak(Box::new(info_struct));
    //         self.structs.push(info_struct);
    //     }
    // }
    pub fn push(&mut self, t: sys::SessionCreateInfoOverlayEXTX) {
        self.t = Some(t);
    }
    pub fn chain(&self) -> Option<&AdditionalSessionCreateInfo> {
        // let mut iter = self.structs.iter_mut().peekable();
        // while let Some(curr) = iter.next() {
        //     if let Some(next) = iter.peek() {
        //         curr.set_next(next.header());
        //     }
        // }
        // self.structs.first().map(|v| v.header())
        None
    }
    pub fn chain_pointer(&self) -> *const c_void {
        let v = Box::leak(Box::new(sys::SessionCreateInfoOverlayEXTX {
            ty: sys::SessionCreateInfoOverlayEXTX::TYPE,
            next: ptr::null(),
            create_flags: OverlaySessionCreateFlagsEXTX::EMPTY,
            session_layers_placement: 0,
        }));
        v as *const sys::SessionCreateInfoOverlayEXTX as *const c_void
        // self.structs.first().unwrap().header() as *const _ as *const _
    }
}

pub trait AsAdditionalSessionCreateInfo: 'static {
    fn header(&self) -> &AdditionalSessionCreateInfo;
    fn set_next(&mut self, next: &AdditionalSessionCreateInfo);
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AdditionalSessionCreateInfo {
    pub ty: sys::StructureType,
    pub next: *const AdditionalSessionCreateInfo,
}

pub struct OxrSessionCreateInfoOverlay {
    pub flags: OverlaySessionCreateFlagsEXTX,
    pub session_layers_placement: u32,
}
impl OxrSessionCreateInfoOverlay {
    pub const fn to_openxr_info(self) -> sys::SessionCreateInfoOverlayEXTX {
        sys::SessionCreateInfoOverlayEXTX {
            ty: sys::SessionCreateInfoOverlayEXTX::TYPE,
            next: ptr::null(),
            create_flags: self.flags,
            session_layers_placement: self.session_layers_placement,
        }
    }
}
impl Default for OxrSessionCreateInfoOverlay {
    fn default() -> Self {
        Self {
            flags: OverlaySessionCreateFlagsEXTX::EMPTY,
            session_layers_placement: 0,
        }
    }
}

impl AsAdditionalSessionCreateInfo for sys::SessionCreateInfoOverlayEXTX {
    fn header(&self) -> &AdditionalSessionCreateInfo {
        unsafe { mem::transmute(&self) }
    }

    fn set_next(&mut self, next: &AdditionalSessionCreateInfo) {
        self.next = next as *const _ as *const _;
    }
}
