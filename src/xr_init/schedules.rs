use bevy::{ecs::schedule::{ScheduleLabel, Schedule, ExecutorKind}, app::App};

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPreSetup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrSetup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPrePostSetup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPostSetup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPreCleanup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrCleanup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPostCleanup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPreRenderUpdate;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrRenderUpdate;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPostRenderUpdate;

pub(super) fn add_schedules(app: &mut App) {
    let schedules = [
        Schedule::new(XrPreSetup),
        Schedule::new(XrSetup),
        Schedule::new(XrPrePostSetup),
        Schedule::new(XrPostSetup),
        Schedule::new(XrPreRenderUpdate),
        Schedule::new(XrRenderUpdate),
        Schedule::new(XrPostRenderUpdate),
        Schedule::new(XrPreCleanup),
        Schedule::new(XrCleanup),
        Schedule::new(XrPostCleanup),
    ];
    for mut schedule in schedules {
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        schedule.set_apply_final_deferred(true);
        app.add_schedule(schedule);
    }
}
