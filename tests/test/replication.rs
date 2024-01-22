//local shortcuts
use crate::*;
use bevy_replicon_attributes::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_replicon::*;
use bevy_replicon::prelude::*;
use bevy_replicon::renet::ClientId;
use bevy_replicon_repair::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(VisibilityAttribute, Default, PartialEq)]
struct A;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct B;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct C;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct D;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct Manual(usize);

impl VisibilityAttribute for Manual
{
    fn inner_attribute_id(&self) -> u64
    {
        self.0 as u64
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ComponentA;

#[derive(Component, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ComponentB;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// normal replication works with plugin
#[test]
fn normal_replication()
{
    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::All,
                ..Default::default()
            }),
        ))
        .replicate::<ComponentA>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    common::connect(&mut server_app, &mut client_app);

    server_app.world.spawn((Replication, ComponentA::default()));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 1);
}

//-------------------------------------------------------------------------------------------------------------------

// visibility via attributes
#[test]
fn basic_visibility()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .replicate::<ComponentA>()
        .replicate::<ComponentB>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // empty Visibility = all can see it
    server_app.world.spawn((Replication, ComponentA, vis!()));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 1);

    // require B
    server_app.world.spawn((Replication, ComponentB, vis!(B)));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    // client doesn't have B yet
    assert!(
        client_app
            .world
            .query_filtered::<Entity, (With<Replication>, With<ComponentB>)>()
            .get_single(&client_app.world)
            .is_err()
    );
    assert_eq!(client_app.world.entities().len(), 1);

    // add B to client
    syscall(&mut server_app.world, (client_id, B), add_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    // client has ComponentB now
    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentB>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 2);
}

//-------------------------------------------------------------------------------------------------------------------

// client connects after entity spawned with empty visibility
#[test]
fn connect_after_empty_vis_spawn()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .replicate::<ComponentA>()
        .replicate::<ComponentB>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    // empty Visibility = all can see it
    server_app.world.spawn((Replication, ComponentA, vis!()));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // connect after spawn
    let (_client_id, _) = common::connect(&mut server_app, &mut client_app);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 1);
}

//-------------------------------------------------------------------------------------------------------------------

// client connects after entity spawned with non-empty visibility
#[test]
fn connect_after_nonempty_vis_spawn()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .replicate::<ComponentA>()
        .replicate::<ComponentB>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    // spawn for A
    server_app.world.spawn((Replication, ComponentA, vis!(A)));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // connect after spawn
    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 1);
}

//-------------------------------------------------------------------------------------------------------------------

// [reset] client reconnects and loses visibility
#[test]
fn reconnect_and_lose_visibility_reset()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .replicate_repair::<ComponentA>()
        .replicate_repair::<ComponentB>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });
    client_app.add_plugins(bevy_replicon_repair::ClientPlugin{ cleanup_prespawns: false });

    let (client_id, server_port) = common::connect(&mut server_app, &mut client_app);

    // spawn for A
    server_app.world.spawn((Replication, ComponentA, vis!(A)));

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 1);

    // disconnect
    client_app.world.resource_mut::<RenetClient>().disconnect();
    client_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    server_app.update();
    assert!(!server_app.world.resource::<RenetServer>().is_connected(ClientId::from_raw(client_id)));

    // don't remove visibility (client is reset)

    // reconnect
    common::reconnect(&mut server_app, &mut client_app, client_id, server_port);
    let _ = client_app.world.resource_mut::<RepliconTick>().into_inner().increment();  //trigger repair
    client_app.update();
    assert_eq!(*client_app.world.resource::<State<ClientRepairState>>(), ClientRepairState::Done);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// [repair] client reconnects and loses visibility (manually remove)
#[test]
fn reconnect_and_lose_visibility_repair()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .replicate_repair::<ComponentA>()
        .replicate_repair::<ComponentB>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Repair });
    client_app.add_plugins(bevy_replicon_repair::ClientPlugin{ cleanup_prespawns: false });

    let (client_id, server_port) = common::connect(&mut server_app, &mut client_app);

    // spawn for A
    server_app.world.spawn((Replication, ComponentA, vis!(A)));

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 1);

    // disconnect
    client_app.world.resource_mut::<RenetClient>().disconnect();
    client_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    server_app.update();
    assert!(!server_app.world.resource::<RenetServer>().is_connected(ClientId::from_raw(client_id)));

    // remove visibility
    syscall(&mut server_app.world, (client_id, A), remove_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));

    // reconnect
    common::reconnect(&mut server_app, &mut client_app, client_id, server_port);
    let _ = client_app.world.resource_mut::<RepliconTick>().into_inner().increment();  //trigger repair
    client_app.update();
    assert_eq!(*client_app.world.resource::<State<ClientRepairState>>(), ClientRepairState::Done);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// [reset] client reconnects and gains visibility of empty vis entity but not new non-empty vis entity
#[test]
fn reconnect_and_vis_accuracy_reset()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .replicate_repair::<ComponentA>()
        .replicate_repair::<ComponentB>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });
    client_app.add_plugins(bevy_replicon_repair::ClientPlugin{ cleanup_prespawns: false });

    let (client_id, server_port) = common::connect(&mut server_app, &mut client_app);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    // add attribute
    syscall(&mut server_app.world, (client_id, B), add_attribute);

    // disconnect
    client_app.world.resource_mut::<RenetClient>().disconnect();
    client_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    server_app.update();
    assert!(!server_app.world.resource::<RenetServer>().is_connected(ClientId::from_raw(client_id)));

    // spawns
    server_app.world.spawn((Replication, ComponentA, vis!()));
    server_app.world.spawn((Replication, ComponentB, vis!(B)));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));

    // reconnect
    common::reconnect(&mut server_app, &mut client_app, client_id, server_port);
    let _ = client_app.world.resource_mut::<RepliconTick>().into_inner().increment();  //trigger repair
    client_app.update();
    assert_eq!(*client_app.world.resource::<State<ClientRepairState>>(), ClientRepairState::Done);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    // did not receive component B, the attribute was lost on disconnect
    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 1);
}

//-------------------------------------------------------------------------------------------------------------------

// [repair] client reconnects and gains visibility of newly spawned entity and newly spawned empty vis entity
#[test]
fn reconnect_and_vis_accuracy_repair()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .replicate_repair::<ComponentA>()
        .replicate_repair::<ComponentB>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Repair });
    client_app.add_plugins(bevy_replicon_repair::ClientPlugin{ cleanup_prespawns: false });

    let (client_id, server_port) = common::connect(&mut server_app, &mut client_app);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    // add attribute
    syscall(&mut server_app.world, (client_id, B), add_attribute);

    // disconnect
    client_app.world.resource_mut::<RenetClient>().disconnect();
    client_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    server_app.update();
    assert!(!server_app.world.resource::<RenetServer>().is_connected(ClientId::from_raw(client_id)));

    // spawns
    server_app.world.spawn((Replication, ComponentA, vis!()));
    server_app.world.spawn((Replication, ComponentB, vis!(B)));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));

    // reconnect
    common::reconnect(&mut server_app, &mut client_app, client_id, server_port);
    let _ = client_app.world.resource_mut::<RepliconTick>().into_inner().increment();  //trigger repair
    client_app.update();
    assert_eq!(*client_app.world.resource::<State<ClientRepairState>>(), ClientRepairState::Done);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    // received component B, the attribute was retained
    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentB>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 2);
}

//-------------------------------------------------------------------------------------------------------------------


// client removes attribute
// client adds/removes attribute in same tick
// client adds/removes/adds attribute in same tick
// multiple clients see different entities

// entity visibility added after spawn
// entity visibility added to multiple entities in the same tick
// entity visibility added to multiple entities in different ticks
// entity visibility removed
// entity visibility changes to empty
// entity visibility changes
// entity visibility changes twice in the same tick
// entity visibility added/removed in the same tick
// entity visibility added/removed/added in the same tick
