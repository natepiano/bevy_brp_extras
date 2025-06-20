//! Extra BRP methods for Bevy applications
//!
//! This crate provides additional Bevy Remote Protocol (BRP) methods that can be added
//! to your Bevy application for enhanced remote control capabilities.
//!
//! # Usage
//!
//! Add the plugin to your Bevy app:
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_brp_extras::BrpExtrasPlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(BrpExtrasPlugin::default())
//!     .run();
//! ```
//!
//! This will add the following BRP methods to your app:
//! - `bevy_brp_extras/screenshot`: Capture a screenshot
//! - `bevy_brp_extras/shutdown`: Gracefully shutdown the app

mod plugin;

pub use plugin::BrpExtrasPlugin;

/// Default port for remote control connections
///
/// This matches Bevy's RemoteHttpPlugin default port to ensure compatibility.
pub const DEFAULT_REMOTE_PORT: u16 = 15702;