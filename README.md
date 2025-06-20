# bevy_brp_extras

bevy_brp_extras does two things
1. Configures your app for bevy remote protocol (BRP)
2. Adds additional methods that can be used with BRP


## Features

Currently provides two BRP methods:
- `bevy_brp_extras/screenshot` - Capture screenshots of the primary window
- `bevy_brp_extras/shutdown` - Gracefully shutdown the application

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_brp_extras = "0.1"
```

Add the plugin to your Bevy app

```rust
use bevy::prelude::*;
use bevy_brp_extras::BrpExtrasPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BrpExtrasPlugin) // will listen on BRP default port 15702
        .run();
}
```

### Custom Port

You can specify a custom port for the BRP server:

```rust
.add_plugins(BrpExtrasPlugin::with_port(8080))
```

## BRP Method Details

### Screenshot
- **Method**: `bevy_brp_extras/screenshot`
- **Parameters**:
  - `path` (string, required): File path where the screenshot should be saved
- **Returns**: Success status with the absolute path where the screenshot will be saved

### Shutdown
- **Method**: `bevy_brp_extras/shutdown`
- **Parameters**: None
- **Returns**: Success status with shutdown confirmation

## Integration with bevy_brp_mcp

This crate is designed to work seamlessly with [bevy_brp_mcp](https://github.com/natepiano/bevy_brp_mcp), which provides a Model Context Protocol (MCP) server for controlling Bevy apps. When both are used together:

1. Add `BrpExtrasPlugin` to your Bevy app
2. Use `bevy_brp_mcp` with your AI coding assistant
3. The additional methods will be automatically discovered and made available

## License

Dual-licensed under either:
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
