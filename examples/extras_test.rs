//! BRP extras test example with keyboard input display
//!
//! This example demonstrates:
//! - Format discovery capabilities of `bevy_brp_extras`
//! - Keyboard input tracking and display
//!
//! Run with: cargo run --example `extras_test`
//!
//! Once running, you can:
//! - Test format discovery: curl -X POST <http://localhost:15702/brp_extras/discover_format>
//! - Send keyboard input: curl -X POST <http://localhost:15702/brp_extras/send_keys>
//!
//! The app displays the last keyboard inputs on screen.

use std::time::Instant;

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::ui::{AlignItems, JustifyContent, Val};
use bevy_brp_extras::BrpExtrasPlugin;

/// Resource to track keyboard input history
#[derive(Resource, Default)]
struct KeyboardInputHistory {
    /// Currently pressed keys
    active_keys:      Vec<String>,
    /// Last pressed keys (for display after release)
    last_keys:        Vec<String>,
    /// Active modifier keys
    modifiers:        Vec<String>,
    /// Time when the last key was pressed
    press_time:       Option<Instant>,
    /// Duration between press and release in milliseconds
    last_duration_ms: Option<u64>,
    /// Whether the last key press has completed
    completed:        bool,
}

/// Marker component for the keyboard input display text
#[derive(Component)]
struct KeyboardDisplayText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::window::WindowPlugin {
            primary_window: Some(bevy::window::Window {
                title: "BRP Extras Test - Keyboard Input Display".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BrpExtrasPlugin::new())
        .init_resource::<KeyboardInputHistory>()
        .add_systems(Startup, (setup_test_entities, setup_ui))
        .add_systems(
            Update,
            (keep_running, track_keyboard_input, update_keyboard_display),
        )
        .run();
}

/// Setup various test entities with different component types
fn setup_test_entities(mut commands: Commands) {
    print_startup_info();
    spawn_test_entities(&mut commands);
    print_usage_instructions();
}

/// Print startup information
fn print_startup_info() {
    info!("=== BRP Extras Test Example ===");
    info!("This example demonstrates:");
    info!("1. Format discovery for component types");
    info!("2. Keyboard input tracking with duration calculation");
    info!("");
    info!("Setting up test entities for format discovery...");
}

/// Spawn test entities for format discovery
fn spawn_test_entities(commands: &mut Commands) {
    // Entity with Transform and Name
    commands.spawn((Transform::from_xyz(1.0, 2.0, 3.0), Name::new("TestEntity1")));

    // Entity with scaled transform
    commands.spawn((
        Transform::from_scale(Vec3::splat(2.0)),
        Name::new("ScaledEntity"),
    ));

    // Entity with more complex Transform
    commands.spawn((
        Transform {
            translation: Vec3::new(10.0, 20.0, 30.0),
            rotation:    Quat::from_rotation_y(std::f32::consts::PI / 4.0),
            scale:       Vec3::new(0.5, 1.5, 2.0),
        },
        Name::new("ComplexTransformEntity"),
    ));

    // Entity with visibility component
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("VisibleEntity"),
        Visibility::default(),
    ));

    info!("Test entities spawned successfully!");
}

/// Print usage instructions for the example
fn print_usage_instructions() {
    info!("");
    info!("=== BRP server running on http://localhost:15702 ===");
    info!("");
    info!("1. Test format discovery:");
    info!("  curl -X POST http://localhost:15702/brp_extras/discover_format \\");
    info!(r#"    -H "Content-Type: application/json" \"#);
    info!(r#"    -d '{{"types": ["bevy_transform::components::transform::Transform"]}}'""#);
    info!("");
    info!("2. Send keyboard input:");
    info!("  curl -X POST http://localhost:15702/brp_extras/send_keys \\");
    info!(r#"    -H "Content-Type: application/json" \"#);
    info!(r#"    -d '{{"keys": ["KeyA", "Space"]}}'""#);
    info!("");
    info!("  # Keys are automatically released after 100ms (default)");
    info!("  # Or specify a custom duration:");
    info!("  curl -X POST http://localhost:15702/brp_extras/send_keys \\");
    info!(r#"    -H "Content-Type: application/json" \"#);
    info!(r#"    -d '{{"keys": ["KeyA", "Space"], "duration_ms": 500}}'""#);
    info!("");
    info!("The UI will display the last pressed keys, modifiers, and duration.");
}

/// Simple system to keep the app running and print periodic status
#[allow(clippy::needless_pass_by_value)] // Bevy systems require owned Res<T>
fn keep_running(time: Res<Time>) {
    // Print periodic status to show the app is alive
    if time.elapsed_secs() % 10.0 < time.delta_secs() {
        info!("App running... BRP server available at http://localhost:15702");
    }
}

/// Setup UI for keyboard input display
fn setup_ui(mut commands: Commands) {
    // Spawn a camera to see the UI
    commands.spawn(Camera2d);

    // Root UI node that fills the entire screen with a dark background
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        ))
        .with_children(|parent| {
            // Container for the text with some padding
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|parent| {
                    // Text node for displaying keyboard input
                    parent.spawn((
                        Text::new("Waiting for keyboard input...\n\nUse curl to send keys:\ncurl -X POST http://localhost:15702/brp_extras/send_keys \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\"keys\": [\"KeyA\", \"Space\"]}'"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        KeyboardDisplayText,
                    ));
                });
        });
}

/// Helper function to update modifiers based on key press/release
fn update_modifiers_from_key(key_str: &str, modifiers: &mut Vec<String>, is_pressed: bool) {
    const MODIFIER_MAPPINGS: &[(&str, &str)] = &[
        ("Control", "Ctrl"),
        ("Shift", "Shift"),
        ("Alt", "Alt"),
        ("Super", "Super"),
    ];

    for &(pattern, display_name) in MODIFIER_MAPPINGS {
        if key_str.contains(pattern) {
            if is_pressed {
                if !modifiers.contains(&display_name.to_string()) {
                    modifiers.push(display_name.to_string());
                }
            } else {
                modifiers.retain(|m| m != display_name);
            }
        }
    }
}

/// Track keyboard input events and update history
fn track_keyboard_input(
    mut events: EventReader<KeyboardInput>,
    mut history: ResMut<KeyboardInputHistory>,
) {
    for event in events.read() {
        // Convert key code to string representation
        let key_str = format!("{:?}", event.key_code);

        match event.state {
            bevy::input::ButtonState::Pressed => {
                info!("Key pressed: {key_str}");

                // Mark as not completed since we have new input
                history.completed = false;

                // Track press time
                history.press_time = Some(Instant::now());

                // Update active key list
                if !history.active_keys.contains(&key_str) {
                    history.active_keys.push(key_str.clone());
                }

                // Track modifiers
                update_modifiers_from_key(&key_str, &mut history.modifiers, true);
            }
            bevy::input::ButtonState::Released => {
                info!("Key released: {key_str}");

                // Calculate duration if we have a press time
                if let Some(press_time) = history.press_time {
                    let duration = Instant::now().duration_since(press_time);
                    history.last_duration_ms = duration.as_millis().try_into().ok();
                }

                // Remove from active keys
                history.active_keys.retain(|k| k != &key_str);

                // Update modifiers
                update_modifiers_from_key(&key_str, &mut history.modifiers, false);

                // If all keys are released, mark as completed and save the last keys
                if history.active_keys.is_empty() && !history.last_keys.is_empty() {
                    history.completed = true;
                }
            }
        }

        // Always update last_keys to show what was pressed
        if !history.active_keys.is_empty() {
            let active = history.active_keys.clone();
            history.last_keys = active;
        }
    }
}

/// Update the keyboard display text
#[allow(clippy::needless_pass_by_value)] // Bevy systems require owned Res<T>
fn update_keyboard_display(
    history: Res<KeyboardInputHistory>,
    mut query: Query<&mut Text, With<KeyboardDisplayText>>,
) {
    // Only update if the history has changed
    if !history.is_changed() {
        return;
    }

    info!(
        "Updating keyboard display - Keys: {:?}, Modifiers: {:?}, Duration: {:?}, Completed: {}",
        history.last_keys, history.modifiers, history.last_duration_ms, history.completed
    );

    for mut text in &mut query {
        let keys_display = if history.last_keys.is_empty() {
            "None".to_string()
        } else {
            history.last_keys.join(", ")
        };

        let modifiers_display = if history.modifiers.is_empty() {
            "None".to_string()
        } else {
            history.modifiers.join(", ")
        };

        let duration_display = if let Some(ms) = history.last_duration_ms {
            format!("{ms}ms")
        } else if history.active_keys.is_empty() {
            "N/A".to_string()
        } else {
            "In progress...".to_string()
        };

        let status = if history.completed {
            "Completed"
        } else if !history.active_keys.is_empty() {
            "Keys pressed"
        } else {
            "Ready"
        };

        text.0 = format!(
            "Last keys: [{keys_display}]\nModifiers: [{modifiers_display}]\nDuration: {duration_display}\nStatus: {status}\n\nUse curl to send keys:\ncurl -X POST http://localhost:15702/brp_extras/send_keys \\\n  -H \"Content-Type: application/json\" \\\n  -d '{{\"keys\": [\"KeyA\", \"Space\"]}}'"
        );
    }
}
