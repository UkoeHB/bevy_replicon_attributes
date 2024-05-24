# Changelog

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
