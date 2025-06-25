# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2025-06-25

### Added
- New `brp_extras/send_keys` method for simulating keyboard input
- New `brp_extras/list_key_codes` method to list available key codes
  - Send key press/release events to Bevy applications
  - Support for all standard Bevy `KeyCode` values

## [0.2.0] - Previous Release

### Added
- Screenshot functionality via `brp_extras/screenshot` method
- Graceful shutdown via `brp_extras/shutdown` method
- Component format discovery via `brp_extras/discover_format` method

[Unreleased]: https://github.com/example/bevy_brp_extras/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/example/bevy_brp_extras/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/example/bevy_brp_extras/releases/tag/v0.2.0
