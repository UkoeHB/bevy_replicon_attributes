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

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ComponentA;

#[derive(Component, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ComponentB;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// visibility blacklist is not allowed
#[should_panic]
#[test]
fn blacklist_disallowed()
{
    App::new().add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                visibility_policy: VisibilityPolicy::Blacklist,
                ..Default::default()
            }),
        ))
        .add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
}

//-------------------------------------------------------------------------------------------------------------------

// visibility whitelist is allowed
#[test]
fn whitelist_allowed()
{
    App::new().add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
}

//-------------------------------------------------------------------------------------------------------------------

// visibility all is allowed
#[test]
fn all_disallowed()
{
    App::new().add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                visibility_policy: VisibilityPolicy::All,
                ..Default::default()
            }),
        ))
        .add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
}

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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // global Visibility = all can see it
    server_app.world.spawn((Replication, ComponentA, vis!(Global)));

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

// client connects after entity spawned with global visibility
#[test]
fn connect_after_global_vis_spawn()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    // global
    server_app.world.spawn((Replication, ComponentA, vis!(Global)));

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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    // empty = invisible
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

    assert_eq!(client_app.world.entities().len(), 0);
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

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

// VisibilityCache::evaluate_connected only sees connected clients even with replicon-repair and Repair policy
#[test]
fn mismatched_connections_with_repair()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app1 = App::new();
    let mut client_app2 = App::new();
    for app in [&mut server_app, &mut client_app1, &mut client_app2] {
        app.add_plugins((
            MinimalPlugins,
            ReplicationPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .replicate_repair::<ComponentA>();
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Repair });
    client_app1.add_plugins(bevy_replicon_repair::ClientPlugin{ cleanup_prespawns: false });
    client_app2.add_plugins(bevy_replicon_repair::ClientPlugin{ cleanup_prespawns: false });

    let (client_id1, server_port) = common::connect(&mut server_app, &mut client_app1);
    let client_id2 = 2u64;
    common::reconnect(&mut server_app, &mut client_app2, client_id2, server_port);

    // add attribute
    syscall(&mut server_app.world, (client_id1, A), add_attribute);
    syscall(&mut server_app.world, (client_id2, A), add_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app1.update();
    client_app2.update();

    // invoke evaluate_connected
    assert_eq!(syscall(&mut server_app.world, vis!(A), evaluate_connected), 2);

    // disconnect
    client_app1.world.resource_mut::<RenetClient>().disconnect();
    client_app1.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    server_app.update();
    server_app.update();
    assert!(!server_app.world.resource::<RenetServer>().is_connected(ClientId::from_raw(client_id1)));

    // invoke evaluate_connected
    assert_eq!(syscall(&mut server_app.world, vis!(A), evaluate_connected), 1);

    // reconnect
    common::reconnect(&mut server_app, &mut client_app1, client_id1, server_port);
    let _ = client_app1.world.resource_mut::<RepliconTick>().into_inner().increment();  //trigger repair
    client_app1.update();
    assert_eq!(*client_app1.world.resource::<State<ClientRepairState>>(), ClientRepairState::Done);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app1.update();
    client_app2.update();

    // invoke evaluate_connected
    assert_eq!(syscall(&mut server_app.world, vis!(A), evaluate_connected), 2);
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Repair });
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

// [reset] client reconnects and gains visibility of global vis entity but not new non-empty vis entity
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
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
    server_app.world.spawn((Replication, ComponentA, vis!(Global)));
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

// [repair] client reconnects and gains visibility of newly spawned entity and newly spawned global vis entity
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Repair });
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
    server_app.world.spawn((Replication, ComponentA, vis!(Global)));
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
#[test]
fn remove_attribute_from_client()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

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

    // remove attribute
    syscall(&mut server_app.world, (client_id, A), remove_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// client adds/removes attribute in same tick
#[test]
fn add_remove_attribute_same_tick()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // spawn for A
    server_app.world.spawn((Replication, ComponentA, vis!(A)));

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    // remove attribute
    syscall(&mut server_app.world, (client_id, A), remove_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// client adds/removes/adds attribute in same tick
#[test]
fn add_remove_add_attribute_same_tick()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // spawn for A
    server_app.world.spawn((Replication, ComponentA, vis!(A)));

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    // remove attribute
    syscall(&mut server_app.world, (client_id, A), remove_attribute);

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

// multiple clients see different entities
#[test]
fn multiple_clients_different_entities()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    let mut client_app1 = App::new();
    let mut client_app2 = App::new();
    for app in [&mut server_app, &mut client_app1, &mut client_app2] {
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id1, server_port) = common::connect(&mut server_app, &mut client_app1);
    let client_id2 = 2u64;
    common::reconnect(&mut server_app, &mut client_app2, client_id2, server_port);

    // spawns
    server_app.world.spawn((Replication, ComponentA, vis!(A)));
    server_app.world.spawn((Replication, ComponentB, vis!(B)));

    // add attributes
    syscall(&mut server_app.world, (client_id1, A), add_attribute);
    syscall(&mut server_app.world, (client_id2, B), add_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app1.update();
    client_app2.update();

    let _client_entity = client_app1
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app1.world);
    assert_eq!(client_app1.world.entities().len(), 1);

    let _client_entity = client_app2
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentB>)>()
        .single(&client_app2.world);
    assert_eq!(client_app2.world.entities().len(), 1);
}

//-------------------------------------------------------------------------------------------------------------------

// entity visibility added after spawn
#[test]
fn vis_added_post_spawn()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    // spawn for A
    let server_entity = server_app.world.spawn((Replication, ComponentA)).id();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // visibility for A
    server_app.world.entity_mut(server_entity).insert(vis!(A));

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

// entity visibility added to multiple entities in the same tick
#[test]
fn vis_added_multiple_entities_same_tick()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    // spawns
    server_app.world.spawn((Replication, ComponentA, vis!(A)));
    server_app.world.spawn((Replication, ComponentA, vis!(A)));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let num = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .iter_mut(&mut client_app.world)
        .len();
    assert_eq!(num, 2);
    assert_eq!(client_app.world.entities().len(), 2);
}

//-------------------------------------------------------------------------------------------------------------------

// entity visibility added to multiple entities in different ticks
#[test]
fn vis_added_multiple_entities_different_ticks()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    // spawns
    server_app.world.spawn((Replication, ComponentA, vis!(A)));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    server_app.world.spawn((Replication, ComponentA, vis!(A)));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let num = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .iter_mut(&mut client_app.world)
        .len();
    assert_eq!(num, 2);
    assert_eq!(client_app.world.entities().len(), 2);
}

//-------------------------------------------------------------------------------------------------------------------

// entity visibility removed
#[test]
fn vis_removed()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    // spawn
    let server_entity = server_app.world.spawn((Replication, ComponentA, vis!(A))).id();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    let _client_entity = client_app
        .world
        .query_filtered::<Entity, (With<Replication>, With<ComponentA>)>()
        .single(&client_app.world);
    assert_eq!(client_app.world.entities().len(), 1);

    // remove visibility
    server_app.world.entity_mut(server_entity).remove::<Visibility>();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// entity visibility changes to empty
#[test]
fn vis_changes_to_empty()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (_client_id, _) = common::connect(&mut server_app, &mut client_app);

    // spawn
    let server_entity = server_app.world.spawn((Replication, ComponentA, vis!(A))).id();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // empty visibility
    server_app.world.entity_mut(server_entity).insert(vis!());

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// entity visibility changes to global
#[test]
fn vis_changes_to_global()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (_client_id, _) = common::connect(&mut server_app, &mut client_app);

    // spawn
    let server_entity = server_app.world.spawn((Replication, ComponentA, vis!(A))).id();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // global visibility
    server_app.world.entity_mut(server_entity).insert(vis!(Global));

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

// entity visibility changes
#[test]
fn vis_changes()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // add attribute
    syscall(&mut server_app.world, (client_id, B), add_attribute);

    // spawn
    let server_entity = server_app.world.spawn((Replication, ComponentA, vis!(A))).id();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // change to B
    server_app.world.entity_mut(server_entity).insert(vis!(B));

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

// entity visibility changes twice in the same tick
#[test]
fn vis_changes_twice_same_tick()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // add attribute
    syscall(&mut server_app.world, (client_id, B), add_attribute);

    // spawn
    let server_entity = server_app.world.spawn((Replication, ComponentA, vis!(A))).id();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // change to B
    server_app.world.entity_mut(server_entity).insert(vis!(B));

    // change to A
    server_app.world.entity_mut(server_entity).insert(vis!(A));

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// entity visibility added/removed in the same tick
#[test]
fn vis_added_removed_same_tick()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    // spawn
    let server_entity = server_app.world.spawn((Replication, ComponentA)).id();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // insert A
    server_app.world.entity_mut(server_entity).insert(vis!(A));

    // remove A
    server_app.world.entity_mut(server_entity).remove::<Visibility>();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// entity visibility added/removed/added in the same tick
#[test]
fn vis_added_removed_added_same_tick()
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
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });

    let (client_id, _) = common::connect(&mut server_app, &mut client_app);

    // add attribute
    syscall(&mut server_app.world, (client_id, A), add_attribute);

    // spawn
    let server_entity = server_app.world.spawn((Replication, ComponentA)).id();

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(client_app.world.entities().len(), 0);

    // insert A
    server_app.world.entity_mut(server_entity).insert(vis!(A));

    // remove A
    server_app.world.entity_mut(server_entity).remove::<Visibility>();

    // insert A
    server_app.world.entity_mut(server_entity).insert(vis!(A));

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
