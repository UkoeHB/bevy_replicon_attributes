//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::{Event, EventWriter};
use bevy::ecs::system::SystemParam;
use bevy_replicon::prelude::{SendMode, ToClients};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// System parameter for sending server events controlled by visibility conditions.
///
/// Sent events are forwarded to `bevy_replicon`, which then transmits them to clients.
/// See `bevy_replicon::ServerEventAppExt`.
///
/// If your event is not `Clone`, you need to use `EventWriter<ToClients<T>>` manually.
#[derive(SystemParam)]
pub struct ServerEventSender<'w, T: Event + Clone>
{
    writer: EventWriter<'w, ToClients<T>>,
}

impl<'w, T: Event + Clone> ServerEventSender<'w, T>
{
    /// Sends an event to connected clients that satisfy the visibility condition.
    ///
    /// If [`VisibilityAttributesPlugin`] was loaded with a server id, then this event will be sent to that id.
    ///
    /// Note that the event will be cloned for each client.
    pub fn send(&mut self, attributes: &ClientAttributes, event: T, condition: VisibilityCondition)
    {
        self.writer
            .send_batch(
                attributes
                    .evaluate_connected(&condition)
                    .map(|client_state|
                        ToClients{
                            mode  : SendMode::Direct(client_state.id()),
                            event : event.clone(),
                        }
                    )
                    .chain(
                        attributes
                            .evaluate_server_player(&condition)
                            .map(|server_id|
                                ToClients{
                                    mode  : SendMode::Direct(server_id),
                                    event : event.clone(),
                                }
                            )
                            .into_iter()
                    )
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
