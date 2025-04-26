use super::{openxr_session_available, resources::OxrInstance};
use bevy::{ecs::system::SystemId, prelude::*};
use bevy_mod_xr::session::{XrFirst, XrHandleEvents};
use openxr::{Event, EventDataBuffer};

pub struct OxrEventsPlugin;

impl Plugin for OxrEventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OxrEventHandlers>();
        app.add_systems(
            XrFirst,
            poll_events
                .in_set(XrHandleEvents::Poll)
                .run_if(openxr_session_available),
        );
    }
}
/// Polls any OpenXR events and handles them accordingly
pub fn poll_events(world: &mut World) {
    let _span = debug_span!("xr_poll_events").entered();
    let instance = world.resource::<OxrInstance>().clone();
    let handlers = world.remove_resource::<OxrEventHandlers>().unwrap();
    let mut buffer = EventDataBuffer::default();
    while let Some(event) = instance
        .poll_event(&mut buffer)
        .expect("Failed to poll event")
    {
        for handler in handlers
            .0
            .iter()
            .map(|v| SystemId::<OxrEventIn, ()>::from_entity(*v))
        {
            if let Err(err) = world.run_system_with(handler, event) {
                error!("error when running oxr event handler: {err}");
            };
        }
    }
    world.insert_resource(handlers);
}

#[derive(Resource, Debug, Default)]
pub struct OxrEventHandlers(Vec<Entity>);
pub trait OxrEventHandlerExt {
    fn add_oxr_event_handler<M>(
        &mut self,
        system: impl IntoSystem<OxrEventIn<'static>, (), M> + 'static,
    ) -> &mut Self;
}
impl OxrEventHandlerExt for App {
    fn add_oxr_event_handler<M>(
        &mut self,
        system: impl IntoSystem<OxrEventIn<'static>, (), M> + 'static,
    ) -> &mut Self {
        self.init_resource::<OxrEventHandlers>();
        let id = self.register_system(system);
        self.world_mut()
            .resource_mut::<OxrEventHandlers>()
            .0
            .push(id.entity());
        self
    }
}

#[derive(Deref)]
pub struct OxrEventIn<'a>(pub Event<'a>);
impl SystemInput for OxrEventIn<'_> {
    type Param<'i> = OxrEventIn<'i>;

    type Inner<'i> = Event<'i>;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        OxrEventIn(this)
    }
}
