use bevy_ecs::resource::Resource;
use bevy_render::extract_resource::ExtractResource;
use openxr::EnvironmentBlendMode;

#[derive(Resource, ExtractResource, Clone)]
pub struct OxrEnvironmentBlendModes {
    available_blend_modes: Vec<EnvironmentBlendMode>,
    current_blend_mode: EnvironmentBlendMode,
}

impl OxrEnvironmentBlendModes {
    pub(crate) fn new(
        available_modes: Vec<EnvironmentBlendMode>,
        preferences: &[EnvironmentBlendMode],
    ) -> Option<OxrEnvironmentBlendModes> {
        let blend_mode = preferences.iter().find(|m| available_modes.contains(m))?;

        Some(Self {
            available_blend_modes: available_modes,
            current_blend_mode: *blend_mode,
        })
    }

    /// returns whether the blend_mode was changed
    pub fn set_blend_mode(&mut self, blend_mode: EnvironmentBlendMode) -> bool {
        if self.available_blend_modes.contains(&blend_mode) {
            self.current_blend_mode = blend_mode;
            return true;
        }
        false
    }

    pub fn blend_mode(&self) -> EnvironmentBlendMode {
        self.current_blend_mode
    }

    pub fn available_blend_modes(&self) -> &[EnvironmentBlendMode] {
        &self.available_blend_modes
    }
}
