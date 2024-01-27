# Changelog

## [0.2.0]

### Changed

- Added a server id option to `VisibilityAttributesPlugin` that can be used if your server is a player.
- `ServerEventSender::send` now only sends events for connected clients plus the 'server as player'.
- Removed `Visibility` to avoid conflicts with bevy's Visibility type, and now `VisibilityCondition` is a component.

### Added

- Add `ClientAttributes::evaluate_connected` which provides access to client change ticks. Useful for manually building server events.
- Add `ServerEventSender::send_with` that allows using a custom event-writer per client.

## [0.1.0]

- Initial release.
