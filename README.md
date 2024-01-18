## Bevy Replicon Attributes

Extends [bevy_replicon](https://github.com/lifescapegame/bevy_replicon) with attributes-based visibility control for server entities and events.


### Usage

TODO


### [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair)

If using [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair) in addition to this crate, add this plugin to your server app:

```rust
app.add_plugins(RepliconAttributesRepair);
```

Note that you need to re-add a client's attributes immediately after they reconnect, otherwise previously-visible entities will be despawned.
