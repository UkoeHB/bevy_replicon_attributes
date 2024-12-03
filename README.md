## Bevy Replicon Attributes

Extends [bevy_replicon](https://github.com/lifescapegame/bevy_replicon) with attributes-based visibility control for server entities and events.


### Basic example

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::{ClientId, Replicated};
use bevy_replicon_attributes::prelude::*;

#[derive(Component)]
struct Bat;

#[derive(Event)]
struct SpawnBat;
#[derive(Event, Copy, Clone)]
struct BatAlert;
#[derive(Event)]
struct GainedNightVision(ClientId);

#[derive(VisibilityAttribute, Default, PartialEq)]
struct HasNightVision;
#[derive(VisibilityAttribute, Default, PartialEq)]
struct IsAwake;
#[derive(VisibilityAttribute, Default, PartialEq)]
struct HatesBats;

fn spawn_bats(
    mut commands : Commands,
    mut events   : EventReader<SpawnBat>,
    mut sender   : ServerEventSender<BatAlert>,
    attributes   : ClientAttributes,
){
    for _ in events.read()
    {
        // Entity
        commands.spawn((Bat, Replicated, vis!(HasNightVision)));

        // Server event
        sender.send(&attributes, BatAlert, vis!(all!(HasNightVision, IsAwake, HatesBats)));
    }
}

fn gain_night_vision(
    mut events     : EventReader<GainedNightVision>,
    mut attributes : ClientAttributes,
){
    for client_id in events.read()
    {
        // Client attribute
        attributes.add(client_id, HasNightVision);
    }
}
```


### Usage

#### Setup

Add replicon to your server app. This crate only works with `VisibilityPolicy::All` and `VisibilityPolicy::Whitelist`.

See [renet](https://github.com/lucaspoffo/renet) for how to set up a renet server.

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;

app.add_plugins(bevy::time::TimePlugin)  //required by bevy_renet
    .add_plugins(RepliconPlugins
        .build()
        .disable::<ClientPlugin>()
        .set(ServerPlugin{
            visibility_policy: VisibilityPolicy::Whitelist,
            ..Default::default(),
        })
    );
```

Add [`VisibilityAttributesPlugin`](bevy_replicon_attributes::VisibilityAttributesPlugin) to your server app *after* the replicon plugins. The plugin will panic if you used `VisibilityPolicy::Blacklist`. You must specify a [`ReconnectPolicy`](bevy_replicon_attributes::ReconnectPolicy):

```rust
use bevy_replicon_attributes::prelude::*;

app.add_plugins(VisibilityAttributesPlugin{ reconnect_policy: ReconnectPolicy::Reset });
```

If you choose [`ReconnectPolicy::Repair`](bevy_replicon_attributes::ReconnectPolicy::Repair), we recommend also using [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair) for preserving replicated state on clients.

#### Define attributes

Attributes can be derived with `VisibilityAttribute`, which requires `Default` and `PartialEq`. Only zero-sized types should use this derive.

```rust
#[derive(VisibilityAttribute, Default, PartialEq)]
struct InStartingArea;
```

More complex attributes should implement [`VisibilityAttribute`](bevy_replicon_attributes::VisibilityAttribute) manually.

```rust
struct InLocation(x: u32, y: u32);

impl VisibilityAttribute for InLocation
{
    fn inner_attribute_id(&self) -> u64
    {
        ((self.x as u64) << 32) + (self.y as u64)
    }
}
```

The [`inner_attribute_id`](bevy_replicon_attributes::VisibilityAttribute::inner_attribute_id) defined here is used to differentiate attribute instances of the same type.

#### Add attributes to a client

Add attributes to clients with the [`ClientAttributes`](bevy_replicon_attributes::ClientAttributes) system parameter.

Client attributes are used when evaluating entity [`VisibilityConditions`](bevy_replicon_attributes::VisibilityCondition) to determine if entities should be replicated to a client.

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::ServerEvent;
use bevy_replicon_attributes::prelude::*;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct IsDisconnected;

fn update_visibility_on_connect_events(
    mut server_events : EventReader<ServerEvent>,
    mut attributes    : ClientAttributes,
){
    for event in server_events.read()
    {
        match event
        {
            ServerEvent::ClientConnected{ id } =>
            {
                attributes.remove(id, IsDisconnected);
                attributes.add(id, InStartingArea);
            }
            ServerEvent::ClientDisconnected{ id, _ } =>
            {
                attributes.add(id, IsDisconnected);
            }
        }
    }
}
```

#### Default client attributes

All clients are given the [`Global`](bevy_replicon_attributes::Global) and [`Client`](bevy_replicon_attributes::Client) builtin attributes each time they connect.

#### Entity visibility

Entity visibility is controlled by [`VisibilityConditions`](bevy_replicon_attributes::VisibilityCondition), which are arbitrary combinations of [`VisibilityAttributes`](bevy_replicon_attributes::VisibilityAttribute) and [`not()`](bevy_replicon_attributes::not)/[`and()`](bevy_replicon_attributes::and)/[`or()`](bevy_replicon_attributes::or) logic.

Entity visibility conditions are evaluated against client attribute lists to determine if entities can be seen by clients.

For convenience we have a [`vis!()`](bevy_replicon_attributes::vis) macro which produces new [`VisibilityCondition`](bevy_replicon_attributes::VisibilityCondition) components. The [`any!()`](bevy_replicon_attributes::vis)/[`all!()`](bevy_replicon_attributes::vis)/[`none!()`](bevy_replicon_attributes::vis) macros can be used inside the `vis!()` macro in addition to `not()`/`and()`/`or()`.

An empty visibility condition always evaluates to `false`. If you want global visibility for an entity, use the builtin [`Global`](bevy_replicon_attributes::Global) attribute that is given to clients when they connect.

Here is a low-level example how it works. In practice you only need to add [`VisibilityAttributes`](bevy_replicon_attributes::VisibilityAttribute) to clients and [`VisibilityCondition`](bevy_replicon_attributes::VisibilityCondition) components to entities. This crate will take care of translating that information into entity visibility within `bevy_replicon`.

```rust
use bevy::prelude::*;
use bevy_replicon_attributes::prelude::*;

fn entity_demo(
    mut commands   : Commands,
    mut attributes : ClientAttributes,
){
    let client_id = ClientId::from_raw(0u64);

    // Add location to client.
    attributes.add(client_id, InLocation(0, 20));

    // Make location condition.
    let location = vis!(InLocation(0, 20));

    // Evaluate condition.
    let client_attributes = attributes.get(client_id).unwrap();
    assert!(location.evaluate(|a| client_attributes.contains(&a)));

    // Spawn entity.
    commands.spawn((Replicated, location));
}
```

Here are examples of more complex visibility conditions:
```rust
// Basic
vis!();
vis!(A);
vis!(not(B));
vis!(and(A, B));
vis!(or(A, B));
vis!(and(A, not(B)));

// Composition
vis!(and(A, vis!(B)));

// Helpers
vis!(any!(A, B, C));   // vis!(or(A, or(B, C)))
vis!(all!(A, B, C));   // vis!(and(A, and(B, C)))
vis!(none!(A, B, C));  // vis!(not(or(A, or(B, C)))))

// Modification
vis!()
    .and(A)                           // vis!(A)
    .or(B)                            // vis!(or(A, B))
    .replace(or(A, B), and(C(1), D))  // vis!(and(C(1), D))
    .replace_type::<C>(E(2))          // vis!(and(E(2), D))
    .remove(E(2))                     // vis!(D)
    ;

```

#### Server events

Visibility of server events can be controlled with the [`ServerEventSender`](bevy_replicon_attributes::ServerEventSender) system parameter.

Server events must be registered with `bevy_replicon`. Clients will receive server events with `EventReader<T>`.

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::prelude::*;

#[derive(Event, Copy, Clone)]
struct E;

fn setup(app: &mut App)
{
    // Replicon server event registration
    app.add_server_event::<E>(EventType::Ordered);
}

fn send_event(mut sender: ServerEventSender<E>, attributes: ClientAttributes)
{
    sender.send(&attributes, E, vis!(any!(Client::from(1), Client::from(2), Client::from(3))));
}
```



## `bevy_replicon` compatability

| `bevy_replicon` | `bevy_replicon_attributes` |
|-------|----------------|
| 0.29  | 0.8 - master   |
| 0.27  | 0.6 - 0.7      |
| 0.26  | 0.5            |
| 0.25  | 0.4            |
| 0.21  | 0.1 - 0.3      |
