//! Shutdown handler for BRP extras

use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::{Value, json};

/// Handler for shutdown requests
///
/// Sends an `AppExit` event to gracefully shutdown the application
#[allow(clippy::unnecessary_wraps)]
pub fn handler(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    // Send app exit event
    world.send_event(bevy::app::AppExit::Success);

    Ok(json!({
        "success": true,
        "message": "Shutdown initiated"
    }))
}
