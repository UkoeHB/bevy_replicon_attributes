## Bevy Replicon Attributes

Extends [bevy_replicon](https://github.com/lifescapegame/bevy_replicon) with attributes-based visibility control for server entities and events.


### Usage

TODO


### Client reconnects

By default all client attributes will be cleared when a client disconnects. If you want to preserve attributes, add this plugin to your server app:

```rust
app.add_plugins(AttributesRepairPlugin);
```

You may also want to use [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair) for preserving replicated state on clients.
