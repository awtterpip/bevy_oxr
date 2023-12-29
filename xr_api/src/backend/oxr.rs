mod graphics;
mod utils;

use std::sync::Mutex;

use crate::prelude::*;

pub struct OXrEntry(openxr::Entry);

impl OXrEntry {
    pub fn new() -> Self {
        #[cfg(feature = "linked")]
        return OXrEntry(openxr::Entry::linked());
        #[cfg(not(feature = "linked"))]
        return OXrEntry(unsafe { openxr::Entry::load().expect("Failed to load OpenXR runtime") });
    }
}

impl Into<openxr::ExtensionSet> for ExtensionSet {
    fn into(self) -> openxr::ExtensionSet {
        let mut set = openxr::ExtensionSet::default();
        set.khr_vulkan_enable2 = self.vulkan;
        set
    }
}

impl EntryTrait for OXrEntry {
    fn available_extensions(&self) -> Result<ExtensionSet> {
        // self.0.enumerate_extensions();
        Ok(ExtensionSet::default())
    }

    fn create_instance(&self, exts: ExtensionSet) -> Result<Instance> {
        #[allow(unused_mut)]
        let mut enabled_extensions: openxr::ExtensionSet = exts.into();
        #[cfg(target_os = "android")]
        {
            enabled_extensions.khr_android_create_instance = true;
        }
        let xr_instance = self.0.create_instance(
            &openxr::ApplicationInfo {
                application_name: "bevy",
                ..Default::default()
            },
            &enabled_extensions,
            &[],
        )?;
        Ok(OXrInstance(xr_instance, exts).into())
    }
}

pub struct OXrInstance(openxr::Instance, ExtensionSet);

impl InstanceTrait for OXrInstance {
    fn entry(&self) -> Entry {
        OXrEntry(self.0.entry().clone()).into()
    }

    fn enabled_extensions(&self) -> ExtensionSet {
        self.1
    }

    fn create_session(&self, info: SessionCreateInfo) -> Result<Session> {
        graphics::init_oxr_graphics(self.0.clone(), self.1, info.texture_format).map(Into::into)
    }
}

pub struct OXrSession {
    pub(crate) instance: Instance,
    pub(crate) session: openxr::Session<openxr::AnyGraphics>,
    pub(crate) render_resources: Mutex<
        Option<(
            wgpu::Device,
            wgpu::Queue,
            wgpu::AdapterInfo,
            wgpu::Adapter,
            wgpu::Instance,
        )>,
    >,
}

impl SessionTrait for OXrSession {
    fn instance(&self) -> &Instance {
        &self.instance
    }

    fn get_render_resources(
        &self,
    ) -> Option<(
        wgpu::Device,
        wgpu::Queue,
        wgpu::AdapterInfo,
        wgpu::Adapter,
        wgpu::Instance,
    )> {
        std::mem::take(&mut self.render_resources.lock().unwrap())
    }

    fn create_input(&self, bindings: Bindings) -> Result<Input> {
        todo!()
    }

    fn begin_frame(&self) -> Result<(View, View)> {
        todo!()
    }

    fn end_frame(&self) -> Result<()> {
        todo!()
    }
}
