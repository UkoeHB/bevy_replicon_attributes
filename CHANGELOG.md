# Changelog

## [0.10.0]

- Update to `bevy_replicon` v0.32.
- With `VisibilityAttributesPlugin::reconnect_policy` set to `ReconnectPolicy::Reset`, clients are only reset once they start replicating (when the `ReplicatedClient` component is added to the client entity) instead of when they initially connect.


## [0.9.0]

- Update to `bevy_replicon` v0.30.
- Remove `VisibilityConnectSet`. Use `bevy_replicon::prelude::ServerSet::TriggerConnectionEvents` and `::Receive` instead.


## [0.8.0]

- Updated `bevy` to v0.15.


## [0.7.3]

### Fixed

- Fix spurious error message in visibility cache when setting up server-clients.


## [0.7.2]

### Fixed

- The visibility cache no longer attempts to set entity visibility for server-clients, which only care about visibility of events.


## [0.7.1]

### Fixed

- Wait until `StartReplication` events are emitted before resetting clients when using `ReconnectPolicy::Reset`.


## [0.7.0]

- Update to `bevy_replicon` v0.28.1.


## [0.6.0]

- Update to `bevy` v0.14, `bevy_replicon` v0.27.


## [0.5.0]

### Changed

- Update to `bevy_replicon` v0.26.


## [0.4.0]

### Changed

- Update to `bevy_replicon` v0.25.


## [0.3.0]

### Changed

- Update to Bevy v0.13


## [0.2.0]

### Added

- Added a server id option to `VisibilityAttributesPlugin` that can be used if your server is a player.
- Add `ClientAttributes::evaluate_connected` which provides access to replicon client state.

### Changed

- `ServerEventSender::send` now only sends events for connected clients plus the 'server as player'.
- Removed `Visibility` to avoid conflicts with bevy's Visibility type, and now `VisibilityCondition` is a component.

## [0.1.0]

- Initial release.
