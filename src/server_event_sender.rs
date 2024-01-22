//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy_replicon::prelude::{SendMode, ToClients};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// System param for sending server events controlled by visibility conditions.
///
/// Sent events are forwarded to `bevy_replicon`, which then transmits them to clients.
///
/// If your event is not `Clone`, you need to use `EventWriter<ToClients<T>>` manually.
#[derive(SystemParam)]
pub struct ServerEventSender<'w, T: Event + Clone>
{
    writer: EventWriter<'w, ToClients<T>>,
}

impl<'w, T: Event + Clone> ServerEventSender<'w, T>
{
    /// Sends an event to clients that satisfy the visibility condition.
    ///
    /// Note that the event will be cloned for each client.
    pub fn send(&mut self, attributes: &ClientAttributes, event: T, condition: Visibility)
    {
        self.writer
            .send_batch(
                attributes
                    .evaluate(&condition)
                    .map(|id|
                        ToClients{
                            mode  : SendMode::Direct(id),
                            event : event.clone(),
                        }
                    )
            )
    }
}

//-------------------------------------------------------------------------------------------------------------------
