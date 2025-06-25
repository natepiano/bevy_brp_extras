//! Plugin implementation for extra BRP methods

use bevy::prelude::*;
use bevy::remote::RemotePlugin;
use bevy::remote::http::RemoteHttpPlugin;

use crate::{DEFAULT_REMOTE_PORT, discovery, keyboard, screenshot, shutdown};

/// Command prefix for `brp_extras` methods
const EXTRAS_COMMAND_PREFIX: &str = "brp_extras/";

/// Plugin that adds extra BRP methods to a Bevy app
///
/// Currently provides:
/// - `brp_extras/screenshot`: Capture screenshots
/// - `brp_extras/shutdown`: Gracefully shutdown the app
/// - `brp_extras/discover_format`: Discover component format information
/// - `brp_extras/send_keys`: Send keyboard input
#[allow(non_upper_case_globals)]
pub const BrpExtrasPlugin: BrpExtrasPlugin = BrpExtrasPlugin::new();

/// Plugin type for adding extra BRP methods
pub struct BrpExtrasPlugin {
    port: Option<u16>,
}

impl Default for BrpExtrasPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl BrpExtrasPlugin {
    /// Create a new plugin instance with default port
    #[must_use]
    pub const fn new() -> Self {
        Self { port: None }
    }

    /// Create plugin with custom port
    #[must_use]
    pub const fn with_port(port: u16) -> Self {
        Self { port: Some(port) }
    }
}

impl Plugin for BrpExtrasPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy's remote plugins with our custom methods
        info!(
            "Registering BRP extras methods with prefix: {}",
            EXTRAS_COMMAND_PREFIX
        );

        let remote_plugin = RemotePlugin::default()
            .with_method(
                format!("{EXTRAS_COMMAND_PREFIX}screenshot"),
                screenshot::handler,
            )
            .with_method(
                format!("{EXTRAS_COMMAND_PREFIX}shutdown"),
                shutdown::handler,
            )
            .with_method(
                format!("{EXTRAS_COMMAND_PREFIX}discover_format"),
                discovery::handler,
            )
            .with_method(
                format!("{EXTRAS_COMMAND_PREFIX}send_keys"),
                keyboard::send_keys_handler,
            );

        let http_plugin = self.port.map_or_else(RemoteHttpPlugin::default, |port| {
            RemoteHttpPlugin::default().with_port(port)
        });

        app.add_plugins((remote_plugin, http_plugin));

        // Add the system to process timed key releases
        app.add_systems(Update, keyboard::process_timed_key_releases);

        let port = self.port.unwrap_or(DEFAULT_REMOTE_PORT);
        app.add_systems(Startup, move |_world: &mut World| {
            log_initialization(port);
        });
    }
}

fn log_initialization(port: u16) {
    info!("BRP extras enabled on http://localhost:{}", port);
    trace!("Additional BRP methods available:");
    trace!("  - brp_extras/screenshot - Take a screenshot");
    trace!("  - brp_extras/shutdown - Shutdown the app");
    trace!("  - brp_extras/discover_format - Discover component format information");
    trace!("  - brp_extras/send_keys - Send keyboard input");
}
