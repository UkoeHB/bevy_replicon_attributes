## Bevy Replicon Attributes

Extends [`bevy_replicon`](https://github.com/lifescapegame/bevy_replicon) with attributes-based visibility control for server entities and events.


### Memory use

To optimize for server load, this crate does not clean up its internal buffers more than necessary. This means constantly generating unique attributes can be considered a memory leak.

For most users that should not be a concern, but long-running servers that continuously generate unique visibility attributes may run into issues.


### Usage

TODO


### [`bevy_replicon_repair`](https://github.com/UkoeHB/bevy_replicon_repair) Compatibility

If using this crate in combination with [`bevy_replicon_repair`](https://github.com/UkoeHB/bevy_replicon_repair), add this plugin to your server app:

```rust
app.add_plugins(RepliconAttributesRepair);
```
