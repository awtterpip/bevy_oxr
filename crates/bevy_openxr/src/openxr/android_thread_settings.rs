use crate::resources::OxrInstance;
use crate::session::OxrSession;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Res, ResMut};
use bevy_log::{error, info, warn};

/// The OS thread ID (Linux `gettid()`) of the application's main thread.
#[derive(Resource, Debug, Clone, Copy)]
pub struct MainThreadTid(pub u32);

/// Tracks render-world thread tagging so it is attempted once per render thread.
#[derive(Resource, Debug, Default)]
pub struct AndroidThreadTagState {
    renderer_main_tid: Option<u32>,
    renderer_main_failed_tid: Option<u32>,
    extension_unavailable_logged: bool,
}

pub fn current_thread_tid() -> u32 {
    unsafe { libc::gettid() as u32 }
}

pub fn tag_application_main_thread(instance: &OxrInstance, session: &OxrSession, tid: u32) {
    match tag_thread(
        instance,
        session,
        openxr::sys::AndroidThreadTypeKHR::APPLICATION_MAIN,
        tid,
    ) {
        ThreadTagStatus::Tagged => {
            info!("XR thread tagging: APPLICATION_MAIN tagged (tid={tid})");
        }
        ThreadTagStatus::ExtensionUnavailable => {
            warn!("XR thread tagging: XR_KHR_android_thread_settings unavailable; skipping");
        }
        ThreadTagStatus::Failed(result) => {
            warn!("XR thread tagging: APPLICATION_MAIN failed for tid={tid}: {result:?}");
        }
    }
}

pub fn tag_render_thread_system(
    instance: Res<OxrInstance>,
    session: Res<OxrSession>,
    mut state: ResMut<AndroidThreadTagState>,
) {
    let tid = current_thread_tid();
    if state.renderer_main_tid == Some(tid) || state.renderer_main_failed_tid == Some(tid) {
        return;
    }

    match tag_thread(
        &instance,
        &session,
        openxr::sys::AndroidThreadTypeKHR::RENDERER_MAIN,
        tid,
    ) {
        ThreadTagStatus::Tagged => {
            state.renderer_main_tid = Some(tid);
            state.renderer_main_failed_tid = None;
            info!("XR thread tagging: RENDERER_MAIN tagged (tid={tid})");
        }
        ThreadTagStatus::ExtensionUnavailable => {
            if !state.extension_unavailable_logged {
                state.extension_unavailable_logged = true;
                warn!("XR thread tagging: XR_KHR_android_thread_settings unavailable; skipping");
            }
        }
        ThreadTagStatus::Failed(result) => {
            state.renderer_main_failed_tid = Some(tid);
            warn!("XR thread tagging: RENDERER_MAIN failed for tid={tid}: {result:?}");
        }
    }
}

enum ThreadTagStatus {
    Tagged,
    ExtensionUnavailable,
    Failed(openxr::sys::Result),
}

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
    if result == openxr::sys::Result::SUCCESS {
        ThreadTagStatus::Tagged
    } else {
        error!("XR thread tagging call failed for {thread_type:?}, tid={tid}: {result:?}");
        ThreadTagStatus::Failed(result)
    }
}
