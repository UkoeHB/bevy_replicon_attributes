//local shortcuts
use bevy_replicon_attributes::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::{prelude::*, test_app::ServerTestAppExt};
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct DummyComponent;

#[derive(Component, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct BasicComponent(pub(super) usize);

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn connect(server_app: &mut App, client_app: &mut App) -> ClientId
{
    server_app.connect_client(client_app);
    client_app.world().resource::<RepliconClient>().id().unwrap()
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn disconnect(server_app: &mut App, client_app: &mut App)
{
    server_app.disconnect_client(client_app);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn reconnect(server_app: &mut App, client_app: &mut App, client_id: ClientId)
{
    let mut client = client_app.world_mut().resource_mut::<RepliconClient>();
    assert!(
        client.is_disconnected(),
        "client can't be connected multiple times"
    );

    client.set_status(RepliconClientStatus::Connected {
        client_id: Some(client_id),
    });

    let mut server = server_app.world_mut().resource_mut::<RepliconServer>();
    server.set_running(true);

    server_app.world_mut().send_event(ServerEvent::ClientConnected { client_id });

    server_app.update(); // Will update `ConnectedClients`, otherwise next call will assign the same ID.
    client_app.update();
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn add_attribute<T: VisibilityAttribute>(In((id, attribute)): In<(ClientId, T)>, mut attributes: ClientAttributes)
{
    attributes.add(id, attribute);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn remove_attribute<T: VisibilityAttribute>(In((id, attribute)): In<(ClientId, T)>, mut attributes: ClientAttributes)
{
    attributes.remove(id, attribute);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn send_event<E: Event + Clone>(
    In((event, vis)): In<(E, VisibilityCondition)>,
    attributes: ClientAttributes,
    mut sender: ServerEventSender<E>,
){
    sender.send(&attributes, event, vis);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn read_event<E: Event + Copy>(mut reader: EventReader<E>) -> Vec<E>
{
    let mut events = Vec::default();
    for event in reader.read()
    {
        events.push(*event);
    }
    events
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn evaluate_connected(In(vis): In<VisibilityCondition>, attributes: ClientAttributes) -> usize
{
    attributes.evaluate_connected(&vis).count()
}

//-------------------------------------------------------------------------------------------------------------------
