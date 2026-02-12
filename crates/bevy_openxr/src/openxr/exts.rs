use std::{
    ffi::CStr,
    ops::{BitAnd, BitOr},
};

use bevy_derive::{Deref, DerefMut};
use bevy_ecs::resource::Resource;
use bevy_log::error;
use openxr::ExtensionSet;

#[derive(Clone, Debug, Eq, PartialEq, Deref, DerefMut, Resource)]
pub struct OxrEnabledExtensions(pub OxrExtensions);

#[derive(Clone, Debug, Eq, PartialEq, Deref, DerefMut)]
pub struct OxrExtensions(ExtensionSet);
impl OxrExtensions {
    pub fn raw_mut(&mut self) -> &mut ExtensionSet {
        &mut self.0
    }
    pub fn raw(&self) -> &ExtensionSet {
        &self.0
    }
    pub fn enable_fb_passthrough(&mut self) -> &mut Self {
        self.0.fb_passthrough = true;
        self
    }
    pub fn disable_fb_passthrough(&mut self) -> &mut Self {
        self.0.fb_passthrough = false;
        self
    }
    pub fn enable_hand_tracking(&mut self) -> &mut Self {
        self.0.ext_hand_tracking = true;
        self
    }
    pub fn disable_hand_tracking(&mut self) -> &mut Self {
        self.0.ext_hand_tracking = false;
        self
    }
    pub fn enable_extx_overlay(&mut self) -> &mut Self {
        self.0.extx_overlay = true;
        self
    }
    /// returns true if all of the extensions enabled are also available in `available_exts`
    pub fn is_available(&self, available_exts: &OxrExtensions) -> bool {
        self.0.intersection(&available_exts) == self.0
    }
    /// Returns any extensions needed by `required_exts` that aren't available in `self`
    pub fn unavailable_exts(&self, required_exts: &Self) -> Vec<String> {
        required_exts
            .difference(&self)
            .names()
            .into_iter()
            .filter_map(|v| {
                CStr::from_bytes_with_nul(v)
                    .inspect_err(|err| error!("failed to convert openxr ext name to CStr: {err}"))
                    .ok()
            })
            .filter_map(|v| {
                v.to_str()
                    .inspect_err(|err| error!("openxr ext name is not valid utf8: {err}"))
                    .ok()
            })
            .map(|v| v.to_string())
            .collect()
    }
}
impl BitOr for OxrExtensions {
    type Output = Self;

    // this is horribly slow, but doesn't require a bunch of code duplication
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(
            self.0
                .names()
                .into_iter()
                .chain(rhs.names().into_iter())
                .collect(),
        )
    }
}
impl BitAnd for OxrExtensions {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.intersection(&rhs))
    }
}
impl From<ExtensionSet> for OxrExtensions {
    fn from(value: ExtensionSet) -> Self {
        Self(value)
    }
}
impl From<OxrExtensions> for ExtensionSet {
    fn from(val: OxrExtensions) -> Self {
        val.0
    }
}
impl Default for OxrExtensions {
    fn default() -> Self {
        let exts = ExtensionSet::default();
        //exts.ext_hand_tracking = true;
        Self(exts)
    }
}
