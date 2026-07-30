#[cfg(target_os = "android")]
use crate::resources::OxrInstance;
#[cfg(target_os = "android")]
use crate::session::OxrSession;
use bevy_ecs::resource::Resource;
#[cfg(target_os = "android")]
use bevy_ecs::world::World;
#[cfg(target_os = "android")]
use bevy_log::{error, info, warn};

/// The OS thread ID (Linux `gettid()`) that created the active OpenXR session.
#[cfg(target_os = "android")]
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ApplicationMainThread {
    pub(crate) tid: u32,
    pub(crate) tagged: bool,
}

/// Tracks one `RENDERER_MAIN` attempt for each render-world session generation.
///
/// The render schedule is multi-threaded, so tagging must be performed by an
/// exclusive system. That system runs on the render sub-app's schedule thread,
/// rather than an arbitrary ECS worker.
#[derive(Resource, Debug, Default)]
pub struct AndroidThreadTagState {
    active_session: Option<openxr::sys::Session>,
    session_generation: u64,
    renderer_main_tid: Option<u32>,
    renderer_main_failed_tid: Option<u32>,
    shared_main_tid: Option<u32>,
    extension_unavailable_logged: bool,
}

#[cfg(target_os = "android")]
pub fn current_thread_tid() -> u32 {
    unsafe { libc::gettid() as u32 }
}

#[cfg(target_os = "android")]
pub fn tag_application_main_thread(instance: &OxrInstance, session: &OxrSession, tid: u32) -> bool {
    match tag_thread(
        instance,
        session,
        openxr::sys::AndroidThreadTypeKHR::APPLICATION_MAIN,
        tid,
    ) {
        ThreadTagStatus::Tagged => {
            info!("XR thread tagging: APPLICATION_MAIN tagged (tid={tid})");
            true
        }
        ThreadTagStatus::ExtensionUnavailable => {
            warn!("XR thread tagging: XR_KHR_android_thread_settings unavailable; skipping");
            false
        }
        ThreadTagStatus::Failed(result) => {
            warn!("XR thread tagging: APPLICATION_MAIN failed for tid={tid}: {result:?}");
            false
        }
    }
}

/// Must remain an exclusive system: Bevy's ordinary `Render` systems can run on
/// arbitrary compute workers, while `RENDERER_MAIN` must identify the thread
/// executing the render sub-app schedule.
#[cfg(target_os = "android")]
pub fn tag_render_thread_system(world: &mut World) {
    let Some(instance) = world.get_resource::<OxrInstance>().cloned() else {
        return;
    };
    let Some(session) = world.get_resource::<OxrSession>().cloned() else {
        return;
    };

    let session_raw = session.as_raw();
    let tid = current_thread_tid();
    let application_main = *world.resource::<ApplicationMainThread>();
    let mut state = world.resource_mut::<AndroidThreadTagState>();
    state.begin_session_generation(session_raw);
    if tid == application_main.tid && application_main.tagged {
        if state.note_shared_main_thread(tid) {
            info!(
                "XR thread tagging: render schedule shares APPLICATION_MAIN tid={tid}; retaining the existing classification"
            );
        }
        return;
    }
    if !state.should_attempt(tid) {
        return;
    }
    drop(state);

    match tag_thread(
        &instance,
        &session,
        openxr::sys::AndroidThreadTypeKHR::RENDERER_MAIN,
        tid,
    ) {
        ThreadTagStatus::Tagged => {
            let mut state = world.resource_mut::<AndroidThreadTagState>();
            state.renderer_main_tid = Some(tid);
            state.renderer_main_failed_tid = None;
            info!(
                "XR thread tagging: RENDERER_MAIN tagged (tid={tid}, application_main_tid={}, application_main_tagged={})",
                application_main.tid, application_main.tagged
            );
        }
        ThreadTagStatus::ExtensionUnavailable => {
            let mut state = world.resource_mut::<AndroidThreadTagState>();
            if !state.extension_unavailable_logged {
                state.extension_unavailable_logged = true;
                warn!("XR thread tagging: XR_KHR_android_thread_settings unavailable; skipping");
            }
        }
        ThreadTagStatus::Failed(result) => {
            let mut state = world.resource_mut::<AndroidThreadTagState>();
            state.renderer_main_failed_tid = Some(tid);
            warn!("XR thread tagging: RENDERER_MAIN failed for tid={tid}: {result:?}");
        }
    }
}

/// Called in the render sub-app before its session resources are removed.
#[cfg(target_os = "android")]
pub fn clear_render_thread_tag_state(world: &mut World) {
    world
        .resource_mut::<AndroidThreadTagState>()
        .clear_for_session_teardown();
    world.remove_resource::<ApplicationMainThread>();
}

impl AndroidThreadTagState {
    fn clear_for_session_teardown(&mut self) {
        self.active_session = None;
        self.renderer_main_tid = None;
        self.renderer_main_failed_tid = None;
        self.shared_main_tid = None;
        self.extension_unavailable_logged = false;
    }

    fn begin_session_generation(&mut self, session: openxr::sys::Session) {
        if self.active_session == Some(session) {
            return;
        }

        self.active_session = Some(session);
        self.session_generation = self.session_generation.wrapping_add(1);
        self.renderer_main_tid = None;
        self.renderer_main_failed_tid = None;
        self.shared_main_tid = None;
        self.extension_unavailable_logged = false;
    }

    fn note_shared_main_thread(&mut self, tid: u32) -> bool {
        if self.shared_main_tid == Some(tid) {
            return false;
        }
        self.shared_main_tid = Some(tid);
        true
    }

    fn should_attempt(&self, tid: u32) -> bool {
        !self.extension_unavailable_logged
            && self.renderer_main_tid != Some(tid)
            && self.renderer_main_failed_tid != Some(tid)
    }
}

#[cfg(target_os = "android")]
enum ThreadTagStatus {
    Tagged,
    ExtensionUnavailable,
    Failed(openxr::sys::Result),
}

#[cfg(target_os = "android")]
fn tag_thread(
    instance: &OxrInstance,
    session: &OxrSession,
    thread_type: openxr::sys::AndroidThreadTypeKHR,
    tid: u32,
) -> ThreadTagStatus {
    let Some(khr) = instance.exts().khr_android_thread_settings.as_ref() else {
        return ThreadTagStatus::ExtensionUnavailable;
    };

    let result =
        unsafe { (khr.set_android_application_thread)(session.as_raw(), thread_type, tid) };
    if is_success_result(result) {
        ThreadTagStatus::Tagged
    } else {
        error!("XR thread tagging call failed for {thread_type:?}, tid={tid}: {result:?}");
        ThreadTagStatus::Failed(result)
    }
}

fn is_success_result(result: openxr::sys::Result) -> bool {
    matches!(
        result,
        openxr::sys::Result::SUCCESS | openxr::sys::Result::SESSION_LOSS_PENDING
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use openxr::sys::Handle;

    fn session(raw: u64) -> openxr::sys::Session {
        openxr::sys::Session::from_raw(raw)
    }

    #[test]
    fn session_generation_resets_cached_thread_outcomes() {
        let mut state = AndroidThreadTagState::default();
        state.begin_session_generation(session(1));
        state.renderer_main_tid = Some(41);
        state.renderer_main_failed_tid = Some(42);
        state.shared_main_tid = Some(43);
        state.extension_unavailable_logged = true;

        state.begin_session_generation(session(2));

        assert_eq!(state.session_generation, 2);
        assert_eq!(state.renderer_main_tid, None);
        assert_eq!(state.renderer_main_failed_tid, None);
        assert_eq!(state.shared_main_tid, None);
        assert!(!state.extension_unavailable_logged);
        assert!(state.should_attempt(41));
    }

    #[test]
    fn teardown_resets_state_even_when_a_runtime_reuses_the_session_handle() {
        let mut state = AndroidThreadTagState::default();
        state.begin_session_generation(session(1));
        state.renderer_main_tid = Some(41);
        state.shared_main_tid = Some(41);

        state.clear_for_session_teardown();
        state.begin_session_generation(session(1));

        assert_eq!(state.session_generation, 2);
        assert_eq!(state.shared_main_tid, None);
        assert!(state.should_attempt(41));
    }

    #[test]
    fn shared_main_thread_is_reported_once_per_session_generation() {
        let mut state = AndroidThreadTagState::default();
        state.begin_session_generation(session(1));

        assert!(state.note_shared_main_thread(41));
        assert!(!state.note_shared_main_thread(41));

        state.begin_session_generation(session(2));
        assert!(state.note_shared_main_thread(41));
    }

    #[test]
    fn session_loss_pending_is_a_success_status() {
        assert!(is_success_result(openxr::sys::Result::SESSION_LOSS_PENDING));
        assert!(!is_success_result(
            openxr::sys::Result::ERROR_RUNTIME_FAILURE
        ));
    }
}
