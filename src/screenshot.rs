//! Screenshot handler for BRP extras

use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult, error_codes};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use serde_json::{Value, json};

/// Handler for screenshot requests
///
/// Takes a screenshot of the primary window and saves it to the specified path.
/// The path parameter must be provided in the request.
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    // Get the path from params
    let path = params
        .as_ref()
        .and_then(|v| v.get("path"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| BrpError {
            code:    error_codes::INVALID_PARAMS,
            message: "Missing 'path' parameter".to_string(),
            data:    None,
        })?;

    // Convert to absolute path
    let path_buf = std::path::Path::new(path);
    let absolute_path = if path_buf.is_absolute() {
        path_buf.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| BrpError {
                code:    error_codes::INTERNAL_ERROR,
                message: format!("Failed to get current directory: {e}"),
                data:    None,
            })?
            .join(path_buf)
    };

    let absolute_path_str = absolute_path.to_string_lossy().to_string();

    // Log the screenshot request
    info!("Screenshot requested for: {}", absolute_path_str);

    // Check if we have a primary window
    let window_exists = world.query::<&Window>().iter(world).any(|w| {
        info!(
            "Found window - resolution: {:?}, visible: {:?}",
            w.resolution, w.visible
        );
        true
    });

    if !window_exists {
        warn!("No windows found in the world!");
    }

    // Spawn a screenshot entity with an observer to handle the capture
    let path_for_observer = absolute_path_str.clone();
    let entity = world
        .spawn((
            Screenshot::primary_window(),
            Name::new(format!("Screenshot_{absolute_path_str}")),
        ))
        .observe(move |trigger: Trigger<ScreenshotCaptured>| {
            info!("Screenshot captured! Attempting to save to: {path_for_observer}");
            let img = trigger.event().0.clone();
            match img.try_into_dynamic() {
                Ok(dyn_img) => {
                    match std::fs::create_dir_all(
                        std::path::Path::new(&path_for_observer)
                            .parent()
                            .unwrap_or_else(|| std::path::Path::new(".")),
                    ) {
                        Ok(()) => match dyn_img.save(&path_for_observer) {
                            Ok(()) => {
                                info!("Screenshot successfully saved to: {path_for_observer}");
                            }
                            Err(e) => {
                                error!("Failed to save screenshot to {path_for_observer}: {e}");
                            }
                        },
                        Err(e) => error!(
                            "Failed to create directory for screenshot {path_for_observer}: {e}"
                        ),
                    }
                }
                Err(e) => error!("Failed to convert screenshot to dynamic image: {e}"),
            }
        })
        .id();

    info!("Screenshot entity spawned with ID: {:?}", entity);

    Ok(json!({
        "success": true,
        "path": absolute_path_str,
        "working_directory": std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("unknown")).to_string_lossy(),
        "note": "Screenshot capture initiated. The file will be saved asynchronously."
    }))
}
