//local shortcuts
use crate::*;
use bevy_replicon_attributes::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_replicon::*;
use bevy_replicon::prelude::*;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(VisibilityAttribute, Default, PartialEq)]
struct A;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct B;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Event, Copy, Clone, Debug, Serialize, Deserialize)]
struct E;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// event sent to empty is seen by all
#[test]
fn event_to_empty_visible_to_all()
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
        .add_server_event::<E>(EventType::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    let (_client_id, _) = common::connect(&mut server_app, &mut client_app);

    // send event to empty
    syscall(&mut server_app.world, (E, vis!()), send_event);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(syscall(&mut client_app.world, (), read_event::<E>).len(), 1);
}

//-------------------------------------------------------------------------------------------------------------------

// event sent to condition is seen by matching clients
#[test]
fn event_to_condition_visible_to_matching()
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
        .add_server_event::<E>(EventType::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    let (client_id1, server_port) = common::connect(&mut server_app, &mut client_app1);
    let client_id2 = 2u64;
    common::reconnect(&mut server_app, &mut client_app2, client_id2, server_port);

    // add attributes
    syscall(&mut server_app.world, (client_id1, A), add_attribute);
    syscall(&mut server_app.world, (client_id2, B), add_attribute);

    // send event to A
    syscall(&mut server_app.world, (E, vis!(A)), send_event);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(&mut client_app1.world, (), read_event::<E>).len(), 1);
    assert_eq!(syscall(&mut client_app2.world, (), read_event::<E>).len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// event sent before attributes added is not seen
#[test]
fn event_before_attributes_not_seen()
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
        .add_server_event::<E>(EventType::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    let (client_id1, server_port) = common::connect(&mut server_app, &mut client_app1);
    let client_id2 = 2u64;
    common::reconnect(&mut server_app, &mut client_app2, client_id2, server_port);

    // send event to A
    syscall(&mut server_app.world, (E, vis!(A)), send_event);

    // add attributes
    syscall(&mut server_app.world, (client_id1, A), add_attribute);
    syscall(&mut server_app.world, (client_id2, B), add_attribute);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(&mut client_app1.world, (), read_event::<E>).len(), 0);
    assert_eq!(syscall(&mut client_app2.world, (), read_event::<E>).len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// event sent with empty vis before client connects is not seen
#[test]
fn event_to_empty_not_visible_to_unconnected()
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
        .add_server_event::<E>(EventType::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    // send event to empty
    syscall(&mut server_app.world, (E, vis!()), send_event);

    let (_client_id, _) = common::connect(&mut server_app, &mut client_app);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app.update();

    assert_eq!(syscall(&mut client_app.world, (), read_event::<E>).len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// multiple events sent in the same tick are all seen by clients
#[test]
fn multiple_events_same_tick_seen()
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
        .add_server_event::<E>(EventType::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });

    let (client_id1, server_port) = common::connect(&mut server_app, &mut client_app1);
    let client_id2 = 2u64;
    common::reconnect(&mut server_app, &mut client_app2, client_id2, server_port);

    // add attributes
    syscall(&mut server_app.world, (client_id1, A), add_attribute);
    syscall(&mut server_app.world, (client_id2, A), add_attribute);

    // send events to A
    syscall(&mut server_app.world, (E, vis!(A)), send_event);
    syscall(&mut server_app.world, (E, vis!(A)), send_event);

    server_app.update();
    std::thread::sleep(std::time::Duration::from_millis(50));
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(&mut client_app1.world, (), read_event::<E>).len(), 2);
    assert_eq!(syscall(&mut client_app2.world, (), read_event::<E>).len(), 2);
}

//-------------------------------------------------------------------------------------------------------------------
