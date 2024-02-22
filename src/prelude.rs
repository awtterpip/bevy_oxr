use bevy::ecs::schedule::{IntoSystemConfigs, SystemConfigs};

pub use crate::xr_init::schedules::XrSetup;
use crate::xr_init::xr_only;
pub use crate::xr_input::{QuatConv, Vec2Conv, Vec3Conv};

pub trait XrSystems<Marker> {
    fn xr_only(self) -> SystemConfigs;
}

impl<T: IntoSystemConfigs<M>, M> XrSystems<M> for T {
    fn xr_only(self) -> SystemConfigs {
        self.into_configs().run_if(xr_only())
    }
}
