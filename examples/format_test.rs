//! Format discovery test example
//!
//! This example demonstrates the format discovery capabilities of bevy_brp_extras.
//! It spawns various entities with different components and enables the BRP extras
//! plugin for testing format discovery functionality.
//!
//! Run with: cargo run --example format_test
//!
//! Once running, you can test the format discovery by calling:
//! curl -X POST http://localhost:15702/brp_extras/discover_format

use bevy::prelude::*;
use bevy_brp_extras::BrpExtrasPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::window::WindowPlugin {
            primary_window: Some(bevy::window::Window {
                title: "Format Discovery Test".to_string(),
                resolution: (400.0, 300.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BrpExtrasPlugin::new())
        .add_systems(Startup, setup_test_entities)
        .add_systems(Update, keep_running)
        .run();
}

/// Setup various test entities with different component types
fn setup_test_entities(mut commands: Commands) {
    info!("Setting up test entities for format discovery...");

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
    info!("BRP server should be running on http://localhost:15702");
    info!("Test format discovery with:");
    info!("  curl -X POST http://localhost:15702/brp_extras/discover_format \\");
    info!(r#"    -H "Content-Type: application/json" \"#);
    info!(r#"    -d '{{"types": ["bevy_transform::components::transform::Transform"]}}'""#);
    info!("Or with multiple types:");
    info!("  curl -X POST http://localhost:15702/brp_extras/discover_format \\");
    info!(r#"    -H "Content-Type: application/json" \"#);
    info!(
        r#"    -d '{{"types": ["bevy_transform::components::transform::Transform", "bevy_core::name::Name"]}}'""#
    );
}

/// Simple system to keep the app running and print periodic status
fn keep_running(time: Res<Time>) {
    // Print periodic status to show the app is alive
    if time.elapsed_secs() % 10.0 < time.delta_secs() {
        info!("App running... BRP server available at http://localhost:15702");
    }
}
