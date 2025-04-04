//local shortcuts
use bevy_replicon_attributes::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::{prelude::*, shared::backend::connected_client::NetworkId, test_app::{ServerTestAppExt, TestClientEntity}};
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct DummyComponent;

#[derive(Component, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct BasicComponent(pub(super) usize);

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn connect(server_app: &mut App, client_app: &mut App, client_id: u64) -> u64
{
    server_app.connect_client(client_app);
    let entity = **client_app.world().resource::<TestClientEntity>();
    server_app.world_mut().get_entity_mut(entity).unwrap().insert(NetworkId::new(client_id));
    server_app.world_mut().flush();
    client_id
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn _disconnect(server_app: &mut App, client_app: &mut App)
{
    server_app.disconnect_client(client_app);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn _reconnect(server_app: &mut App, client_app: &mut App, client_id: u64)
{
    let mut client = client_app.world_mut().resource_mut::<RepliconClient>();
    assert!(
        client.is_disconnected(),
        "client can't be connected multiple times"
    );

    client.set_status(RepliconClientStatus::Connected);

    let mut server = server_app.world_mut().resource_mut::<RepliconServer>();
    server.set_running(true);

    server_app.world_mut().spawn((ConnectedClient{ max_size: 1200 }, NetworkId::new(client_id)));

    server_app.update(); // Will update `ConnectedClients`, otherwise next call will assign the same ID.
    client_app.update();
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn add_attribute<T: VisibilityAttribute>(In((id, attribute)): In<(u64, T)>, mut attributes: ClientAttributes)
{
    attributes.add(id, attribute);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn remove_attribute<T: VisibilityAttribute>(In((id, attribute)): In<(u64, T)>, mut attributes: ClientAttributes)
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

pub(super) fn _evaluate_connected(In(vis): In<VisibilityCondition>, attributes: ClientAttributes) -> usize
{
    attributes.evaluate_connected(&vis).count()
}

//-------------------------------------------------------------------------------------------------------------------
