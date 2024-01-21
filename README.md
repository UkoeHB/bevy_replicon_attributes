## Bevy Replicon Attributes

Extends [bevy_replicon](https://github.com/lifescapegame/bevy_replicon) with attributes-based visibility control for server entities and events.


### Basic example

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::{ClientId, Replication};
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
        commands.spawn((Bat, Replication, vis!(HasNightVision)));

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

Add replicon to your server app. Most of the time you want the whitelist visibility policy.

See [renet](https://github.com/lucaspoffo/renet) for how to set up a renet server.

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;

app.add_plugins(bevy::time::TimePlugin)  //required by bevy_renet
    .add_plugins(ReplicationPlugins
        .build()
        .disable::<ClientPlugin>()
        .set(ServerPlugin{
            visibility_policy: VisibilityPolicy::Whitelist,
            ..Default::default(),
        })
    );
```

Add [`VisibilityAttributesPlugin`](bevy_replicon_attributes::VisibilityAttributesPlugin) to your server app. You must specify a [`ReconnectPolicy`](bevy_replicon_attributes::ReconnectPolicy):

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
use bevy_renet::renet::ServerEvent;
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

#### Entity visibility

Entity visibility is controlled by [`VisibilityConditions`](bevy_replicon_attributes::VisibilityCondition), which are arbitrary combinations of [`VisibilityAttributes`](bevy_replicon_attributes::VisibilityAttribute) and [`not()`](bevy_replicon_attributes::not)/[`and()`](bevy_replicon_attributes::and)/[`or()`](bevy_replicon_attributes::or) logic.

Entity visibility conditions are evaluated against client attribute lists to determine if entities can be seen by clients.

An empty visibility condition always evaluates to `true`. This way if an entity has no `Visibility` component it will be visible to no clients (assuming you use a whitelist policy), if it has an empty `Visibility` it will be visible to all clients, and if it has a non-empty `Visibility` then it will be visible to clients that match the condition. Semantically, an empty condition matches 'anything', and a non-empty condition is equivalent to `and(ANYTHING, condition)`.

For convenience we have a [`vis!()`](bevy_replicon_attributes::vis) macro which produces new [`Visibility`](bevy_replicon_attributes::VisibilityCondition) components (simple wrappers around [`VisibilityConditions`](bevy_replicon_attributes::VisibilityCondition)). The [`any!()`](bevy_replicon_attributes::vis)/[`all!()`](bevy_replicon_attributes::vis)/[`none!()`](bevy_replicon_attributes::vis) macros can be used in side the `vis!()` macro in addition to `not()`/`and()`/`or()`.

Here is a low-level example how it works. In practice you only need to add [`VisibilityAttributes`](bevy_replicon_attributes::VisibilityAttribute) to clients and [`Visibility`](bevy_replicon_attributes::VisibilityCondition) components to entities. This crate will take care of translating that information into entity visibility within `bevy_replicon`.

```rust
use bevy::prelude::*;
use bevy_replicon_attributes::prelude::*;

fn entity_demo(
    mut commands   : Commands,
    mut attributes : ClientAttributes,
){
    // Add location to client.
    let dummy_client = ClientId::from_raw(0u64);
    attributes.add(dummy_client, InLocation(0, 20));
    let client_attributes = attributes.get(client_id).unwrap();

    // Make location condition.
    let condition = vis!(InLocation(0, 20));

    // Evaluate condition.
    assert!(condition.evaluate(|a| client_attributes.contains(&a)));

    // Spawn entity.
    commands.spawn((Replication, condition));
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
vis!(any!(A, B, C));   //equivalent: vis!(or(A, or(B, C)))
vis!(all!(A, B, C));   //equivalent: vis!(and(A, and(B, C)))
vis!(none!(A, B, C));  //equivalent: vis!(none(or(A, or(B, C)))))

// Modification
vis!()
    .and(A)
    .or(B)
    .replace(or(A, B), and(C(1), D))  //pattern replacement
    .replace_type::<C>(E(2))  //replaces all attributes of type C with E(2)
    .remove(E(2))  //removing nodes causes the condition to simplify itself
    ;

```

#### Replicating entities

We include a [`replicate_to!()`](bevy_replicon_attributes::replicate_to) macro that simplifies spawning replicated entities.

```rust
use bevy::prelude::*;
use bevy_replicon_attributes::prelude::*;

fn easy_spawn(mut commands: Commands)
{
    commands.spawn(
        (
            Ward,
            replicate_to!(and(InLocation(x, y), InTeam(team)))
            //equivalent: (Replication, vis!(and(InLocation(x, y), InTeam(team))))
        )
    );
}
```

#### Server events

Visibility of server events can be controlled with [`ServerEventSender`](bevy_replicon_attributes::ServerEventSender).

Server events must be registered with `bevy_replicon`. Clients will receive events with `EventReader<T>`.

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::prelude::*;

#[derive(Event, Copy, Clone)]
struct E;

fn setup(app: &mut App)
{
    // Replicon server event registration
    app.add_server_event::<E>();
}

fn send_event(mut sender: ServerEventSender<E>, attributes: ClientAttributes)
{
    sender.send(&attributes, E, vis!(any!(Client(1), Client(2), Client(3))));
}
```
