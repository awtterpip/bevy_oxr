use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

use bevy::app::{App, Plugin, PluginsState};
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::world::World;
use bevy::log::{error, info};
use bevy::render::RenderApp;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use js_sys::Object;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, XrFrame, XrReferenceSpace, XrReferenceSpaceType,
    XrRenderStateInit, XrSession, XrSessionMode, XrWebGlLayer,
};
use winit::platform::web::WindowExtWebSys;

#[derive(Clone)]
struct FutureXrSession(Rc<Mutex<Option<Result<(XrSession, XrReferenceSpace), JsValue>>>>);

pub struct XrInitPlugin;

impl Plugin for XrInitPlugin {
    fn build(&self, app: &mut App) {
        let canvas = get_canvas(&mut app.world).unwrap();
        let future_session = FutureXrSession(Default::default());
        app.set_runner(webxr_runner);
        app.insert_non_send_resource(future_session.clone());
        bevy::tasks::IoTaskPool::get().spawn_local(async move {
            let result = init_webxr(
                canvas,
                XrSessionMode::ImmersiveVr,
                XrReferenceSpaceType::Local,
            )
            .await;
            *future_session.0.lock().unwrap() = Some(result);
        });
    }

    fn ready(&self, app: &App) -> bool {
        app.world
            .get_non_send_resource::<FutureXrSession>()
            .and_then(|fxr| fxr.0.try_lock().map(|locked| locked.is_some()).ok())
            .unwrap_or(true)
    }

    fn finish(&self, app: &mut App) {
        info!("finishing");

        if let Some(Ok((session, reference_space))) = app
            .world
            .remove_non_send_resource::<FutureXrSession>()
            .and_then(|fxr| fxr.0.lock().unwrap().take())
        {
            app.insert_non_send_resource(session.clone())
                .insert_non_send_resource(reference_space.clone());
            app.sub_app_mut(RenderApp)
                .insert_non_send_resource(session)
                .insert_non_send_resource(reference_space);
        }
    }
}

fn webxr_runner(mut app: App) {
    fn set_timeout(f: &Closure<dyn FnMut()>, dur: Duration) {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                f.as_ref().unchecked_ref(),
                dur.as_millis() as i32,
            )
            .expect("Should register `setTimeout`.");
    }
    let run_xr_inner = Rc::new(RefCell::new(None));
    let run_xr: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = run_xr_inner.clone();
    *run_xr.borrow_mut() = Some(Closure::new(move || {
        let app = &mut app;
        if app.plugins_state() == PluginsState::Ready {
            app.finish();
            app.cleanup();
            run_xr_app(std::mem::take(app));
        } else {
            set_timeout(
                run_xr_inner.borrow().as_ref().unwrap(),
                Duration::from_millis(1),
            );
        }
    }));
    set_timeout(run_xr.borrow().as_ref().unwrap(), Duration::from_millis(1));
}

fn run_xr_app(mut app: App) {
    let session = app.world.non_send_resource::<XrSession>().clone();
    let inner_closure: Rc<RefCell<Option<Closure<dyn FnMut(f64, XrFrame)>>>> =
        Rc::new(RefCell::new(None));
    let closure = inner_closure.clone();
    *closure.borrow_mut() = Some(Closure::new(move |_time, frame: XrFrame| {
        let session = frame.session();
        app.insert_non_send_resource(frame);
        info!("update");
        app.update();
        session.request_animation_frame(
            inner_closure
                .borrow()
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        );
    }));
    session.request_animation_frame(closure.borrow().as_ref().unwrap().as_ref().unchecked_ref());
}

fn get_canvas(world: &mut World) -> Option<HtmlCanvasElement> {
    let window_entity = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .get_single(world)
        .ok()?;
    let windows = world.get_non_send_resource::<WinitWindows>()?;
    Some(windows.get_window(window_entity)?.canvas())
}

async fn init_webxr(
    canvas: HtmlCanvasElement,
    mode: XrSessionMode,
    reference_type: XrReferenceSpaceType,
) -> Result<(XrSession, XrReferenceSpace), JsValue> {
    let xr = web_sys::window().unwrap().navigator().xr();

    let supports_session = JsFuture::from(xr.is_session_supported(mode)).await?;
    if supports_session == false {
        error!("XR session {:?} not supported", mode);
        return Err(JsValue::from_str(&format!(
            "XR session {:?} not supported",
            mode
        )));
    }

    info!("creating session");
    let session: XrSession = JsFuture::from(xr.request_session(mode)).await?.into();

    info!("creating gl");
    let gl: WebGl2RenderingContext = {
        let gl_attribs = Object::new();
        js_sys::Reflect::set(
            &gl_attribs,
            &JsValue::from_str("xrCompatible"),
            &JsValue::TRUE,
        )?;
        canvas
            .get_context_with_context_options("webgl2", &gl_attribs)?
            .ok_or(JsValue::from_str(
                "Unable to create WebGL rendering context",
            ))?
            .dyn_into()?
    };

    let xr_gl_layer = XrWebGlLayer::new_with_web_gl2_rendering_context(&session, &gl)?;
    let mut render_state_init = XrRenderStateInit::new();
    render_state_init.base_layer(Some(&xr_gl_layer));
    session.update_render_state_with_state(&render_state_init);
    info!("creating ref space");
    let reference_space = JsFuture::from(session.request_reference_space(reference_type))
        .await?
        .into();

    info!("finished");
    Ok((session, reference_space))
}
