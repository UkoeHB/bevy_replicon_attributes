# Changelog

## [0.1.1]

### Changed

- `ServerEventSender::send` now only sends events for connected clients.

### Added

- Add `ClientAttributes::evaluate_connected` which provides access to client change ticks. Useful for manually building server events.
- Add `ServerEventSender::send_with` that allows using a custom event-writer per client.

## [0.1.0]

- Initial release.
