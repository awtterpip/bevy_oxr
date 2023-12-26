use crate::prelude::*;

pub struct OXrEntry(openxr::Entry);

impl EntryTrait for OXrEntry {
    fn available_extensions(&self) -> Result<ExtensionSet> {
        // self.0.enumerate_extensions();
        Ok(ExtensionSet::default())
    }

    fn create_instance(&self, exts: ExtensionSet) -> Result<Instance> {
        todo!()
    }
}

pub struct OXrInstance(openxr::Instance);

impl InstanceTrait for OXrInstance {
    fn entry(&self) -> Entry {
        OXrEntry(self.0.entry().clone()).into()
    }

    fn enabled_extensions(&self) -> ExtensionSet {
        todo!()
    }

    fn create_session(&self, info: SessionCreateInfo) -> Result<Session> {
        todo!()
    }
}
