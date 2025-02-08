//local shortcuts
use crate::*;
use bevy_replicon_attributes::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_replicon::{prelude::*, test_app::ServerTestAppExt};
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

// event sent to empty is seen by none
#[test]
fn event_to_empty_visible_to_none()
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
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
    client_app.finish();
    server_app.finish();

    let _client_id = common::connect(&mut server_app, &mut client_app);

    // send event to empty
    server_app.world_mut().syscall((E, vis!()), send_event);

    server_app.update();
    server_app.exchange_with_client(&mut client_app);
    client_app.update();

    assert_eq!(client_app.world_mut().syscall((), read_event::<E>).len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// event sent to global is seen by all
#[test]
fn event_to_global_visible_to_all()
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
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
    client_app.finish();
    server_app.finish();

    let _client_id = common::connect(&mut server_app, &mut client_app);

    // send event to empty
    server_app.world_mut().syscall((E, vis!(Global)), send_event);

    server_app.update();
    server_app.exchange_with_client(&mut client_app);
    client_app.update();

    assert_eq!(client_app.world_mut().syscall((), read_event::<E>).len(), 1);
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
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
    client_app1.finish();
    client_app2.finish();
    server_app.finish();

    let client_id1 = common::connect(&mut server_app, &mut client_app1);
    let client_id2 = common::connect(&mut server_app, &mut client_app2);

    // add attributes
    server_app.world_mut().syscall((client_id1, A), add_attribute);
    server_app.world_mut().syscall((client_id2, B), add_attribute);

    // send event to A
    server_app.world_mut().syscall((E, vis!(A)), send_event);

    server_app.update();
    server_app.exchange_with_client(&mut client_app1);
    server_app.exchange_with_client(&mut client_app2);
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(client_app1.world_mut(), (), read_event::<E>).len(), 1);
    assert_eq!(syscall(client_app2.world_mut(), (), read_event::<E>).len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// event sent to client ids is seen by matching clients
#[test]
fn event_to_clients_visible_to_targets()
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
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
    client_app1.finish();
    client_app2.finish();
    server_app.finish();

    let client_id1 = common::connect(&mut server_app, &mut client_app1);
    let _client_id2 = common::connect(&mut server_app, &mut client_app2);

    // send event to client
    server_app.world_mut().syscall((E, vis!(Client(client_id1))), send_event);

    server_app.update();
    server_app.exchange_with_client(&mut client_app1);
    server_app.exchange_with_client(&mut client_app2);
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(client_app1.world_mut(), (), read_event::<E>).len(), 1);
    assert_eq!(syscall(client_app2.world_mut(), (), read_event::<E>).len(), 0);
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
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
    client_app1.finish();
    client_app2.finish();
    server_app.finish();

    let client_id1 = common::connect(&mut server_app, &mut client_app1);
    let client_id2 = common::connect(&mut server_app, &mut client_app2);

    // send event to A
    server_app.world_mut().syscall((E, vis!(A)), send_event);

    // add attributes
    server_app.world_mut().syscall((client_id1, A), add_attribute);
    server_app.world_mut().syscall((client_id2, B), add_attribute);

    server_app.update();
    server_app.exchange_with_client(&mut client_app1);
    server_app.exchange_with_client(&mut client_app2);
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(client_app1.world_mut(), (), read_event::<E>).len(), 0);
    assert_eq!(syscall(client_app2.world_mut(), (), read_event::<E>).len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// event sent with global vis before client connects is not seen
#[test]
fn event_to_global_not_visible_to_unconnected()
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
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
    client_app.finish();
    server_app.finish();

    // send event to empty
    server_app.world_mut().syscall((E, vis!(Global)), send_event);

    let _client_id = common::connect(&mut server_app, &mut client_app);

    server_app.update();
    server_app.exchange_with_client(&mut client_app);
    client_app.update();

    assert_eq!(client_app.world_mut().syscall((), read_event::<E>).len(), 0);
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
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset });
    client_app1.finish();
    client_app2.finish();
    server_app.finish();

    let client_id1 = common::connect(&mut server_app, &mut client_app1);
    let client_id2 = common::connect(&mut server_app, &mut client_app2);

    // add attributes
    server_app.world_mut().syscall((client_id1, A), add_attribute);
    server_app.world_mut().syscall((client_id2, A), add_attribute);

    // send events to A
    server_app.world_mut().syscall((E, vis!(A)), send_event);
    server_app.world_mut().syscall((E, vis!(A)), send_event);

    server_app.update();
    server_app.exchange_with_client(&mut client_app1);
    server_app.exchange_with_client(&mut client_app2);
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(client_app1.world_mut(), (), read_event::<E>).len(), 2);
    assert_eq!(syscall(client_app2.world_mut(), (), read_event::<E>).len(), 2);
}

//-------------------------------------------------------------------------------------------------------------------

// event sent w/ server-as-player is seen by server
#[test]
fn event_to_server_visible_to_server()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    for app in [&mut server_app] {
        app.add_plugins((
            MinimalPlugins,
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{
        server_id: Some(ClientId::SERVER),
        reconnect_policy: ReconnectPolicy::Reset
    });
    server_app.finish();

    // initialize the server app
    server_app.update();

    // send event
    server_app.world_mut().syscall((E, vis!(Global)), send_event);

    // update app for local resending
    server_app.update();

    assert_eq!(server_app.world_mut().syscall((), read_event::<E>).len(), 1);
}

//-------------------------------------------------------------------------------------------------------------------

// event sent w/ server-as-player but wrong visibility is not seen by server
#[test]
fn event_not_visible_to_server()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let mut server_app = App::new();
    for app in [&mut server_app] {
        app.add_plugins((
            MinimalPlugins,
            RepliconPlugins.set(bevy_replicon::prelude::ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
        ))
        .add_server_event::<E>(ChannelKind::Ordered);
    }
    server_app.add_plugins(VisibilityAttributesPlugin{
        server_id: Some(ClientId::SERVER),
        reconnect_policy: ReconnectPolicy::Reset
    });
    server_app.finish();

    // initialize the server app
    server_app.update();

    // send event
    server_app.world_mut().syscall((E, vis!(A)), send_event);

    // update app for local resending
    server_app.update();

    assert_eq!(server_app.world_mut().syscall((), read_event::<E>).len(), 0);
}

//-------------------------------------------------------------------------------------------------------------------
