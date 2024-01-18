## Bevy Replicon Attributes

Extends [bevy_replicon](https://github.com/lifescapegame/bevy_replicon) with attributes-based visibility control for server entities and events.


### Client reconnects

By default all client attributes will be cleared when a client disconnects. If you want to preserve attributes, add this plugin to your server app:

```rust
app.add_plugins(AttributesRepairPlugin);
```

You may also want to use [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair) for preserving replicated state on clients.


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

Add [`AttributesPlugin`](bevy_replicon_attributes::AttributesPlugin) to your server app:

```rust
use bevy_replicon_attributes::prelude::*;

app.add_plugins(AttributesPlugin);
```

#### Define attributes

Basic attributes can be derived with `VisibilityAttribute`, which requires `Default` and `PartialEq`. Only zero-sized types should use this derive.

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

The [`inner_attribute_id`](bevy_replicon_attributes::VisibilityAttribute::inner_attribute_id) defined here is used to differentiate `InLocation` instances.

#### Add attributes to a client

Attributes can be modified on clients with the [`ClientAttributes`](bevy_replicon_attributes::ClientAttributes) system parameter.

A client's attribute list is used when evaluating entity [`VisibilityConditions`](bevy_replicon_attributes::VisibilityCondition) to determine if entities should be replicated to the client.

```rust
use bevy::prelude::*;
use bevy_renet::renet::ServerEvent;
use bevy_replicon_attributes::prelude::*;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct IsDisconnected;

fn update_client_visibility_on_connect_events(
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

Client attribute lists are evaluated against entity visibility conditions to determine if entities can be seen by clients.

For convenience we have a [`visibility!()`](bevy_replicon_attributes::visibility) macro which produces new [`Visibility`](bevy_replicon_attributes::VisibilityCondition) components (simple wrappers around [`VisibilityConditions`](bevy_replicon_attributes::VisibilityCondition)).

Here is a low-level example how it works. In practice you only need to add attributes to clients and add [`Visibility`](bevy_replicon_attributes::VisibilityCondition) components to entities. This crate will take care of translating that information into entity visibility within `bevy_replicon`.

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
    let condition = visibility!(InLocation(0, 20));

    // Evaluate condition.
    assert!(condition.evaluate(|a| client_attributes.contains(&a)));

    // Spawn entity.
    commands.spawn((Replication, condition));
}
```

Here are examples of more complex visibility conditions:
```rust
// Basic
visibility!();
visibility!(A);
visibility!(not(B));
visibility!(or(A, B));
visibility!(and(A, B));
visibility!(and(A, not(B)));

// Composition
visibility!(and(A, visibility!(B)));

// Modification
visibility!()
    .and(A)
    .or(B)
    .replace(or(A, B), and(C(1), D))  //pattern replacement
    .replace_type::<C>(E(2))  //replaces all attributes of type C with E(2)
    .remove(E(2))  //removing nodes causes the condition to simplify itself
    ;

```

Note that we include a [`replicate_to!()`](bevy_replicon_attributes::replicate_to) macro that simplifies spawning replicated entities.

```rust
use bevy::prelude::*;
use bevy_replicon_attributes::prelude::*;

fn easy_spawn(mut commands: Commands)
{
    commands.spawn(
        (
            Ward,
            replicate_to!(and(InLocation(x, y), InTeam(team)))
            //equivalent: (Replication, visibility!(and(InLocation(x, y), InTeam(team))))
        )
    );
}
```
