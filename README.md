# bevy_brp_extras

[![Crates.io](https://img.shields.io/crates/v/bevy_brp_extras.svg)](https://crates.io/crates/bevy_brp_extras)
[![Documentation](https://docs.rs/bevy_brp_extras/badge.svg)](https://docs.rs/bevy_brp_extras/)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/natepiano/bevy_brp_extras#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_brp_extras.svg)](https://crates.io/crates/bevy_brp_extras)
[![CI](https://github.com/natepiano/bevy_brp_extras/workflows/CI/badge.svg)](https://github.com/natepiano/bevy_brp_extras/actions)

bevy_brp_extras does two things
1. Configures your app for bevy remote protocol (BRP)
2. Adds additional methods that can be used with BRP

## Bevy Compatibility

| bevy | bevy_brp_extras |
|------|-----------------|
| 0.16 | 0.1, 0.2        |

The bevy_brp_extras crate follows Bevy's version numbering and releases new versions for each Bevy release. 
The table above shows which versions of bevy_brp_extras are compatible with which versions of Bevy.


## Features

Currently provides three BRP methods:
- `brp_extras/screenshot` - Capture screenshots of the primary window
- `brp_extras/shutdown` - Gracefully shutdown the application  
- `brp_extras/discover_format` - Get correct data formats for BRP spawn/insert/mutation operations

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_brp_extras = "0.2"
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
- **Method**: `brp_extras/screenshot`
- **Parameters**:
  - `path` (string, required): File path where the screenshot should be saved
- **Returns**: Success status with the absolute path where the screenshot will be saved

**Note**: If you're not using this with [bevy_brp_mcp](https://github.com/natepiano/bevy_brp_mcp), you'll need to tell your AI agent that this method requires a `path` parameter, or let it discover this by trying the method and getting an error message.

### Shutdown
- **Method**: `brp_extras/shutdown`
- **Parameters**: None
- **Returns**: Success status with shutdown confirmation

### Format Discovery
- **Method**: `brp_extras/discover_format`
- **Parameters**:
  - `types` (array of strings, required): **Fully-qualified component type paths** (e.g., `"bevy_transform::components::transform::Transform"`, not just `"Transform"`)
- **Returns**: Correct JSON structure needed for BRP spawn, insert, and mutation operations

**Why this exists:** Bevy's built-in `bevy/registry/schema` method provides type schemas, but doesn't show the actual JSON format needed for BRP operations. This method bridges that gap by providing the exact data structures required.

**Example:**
```bash
curl -X POST http://localhost:15702/brp_extras/discover_format \
  -H "Content-Type: application/json" \
  -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_core::name::Name"]}'
```

**Important:** Use fully-qualified type paths, not short names. Use `bevy/list` to find the correct paths.

**Response shows:**
- `spawn_format`: How to structure data for `bevy/spawn` operations
- `mutation_info`: Available mutation paths and formats for `bevy/mutate_component` operations

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
