//! A application to control and emulate mechanical systems and play as a workbench for
//! physics simulations.
//!
//! Just run `cargo run --release`, and you should see a window with a basic example.
use bevy::{prelude::*, window::WindowPlugin};

use avian3d::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(feature = "embedded-model")]
mod embedded_model;
#[cfg(feature = "blender-model")]
mod scene_viewer_plugin;

mod config_plugin;
mod grid_plugin;
#[cfg(feature = "grpc")]
mod grpc_plugin;

use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
#[cfg(feature = "embedded-model")]
use embedded_model::EmbeddedModelPlugin;
use grid_plugin::GridPlugin;

use config_plugin::ConfigPlugin;
#[cfg(feature = "grpc")]
use grpc_plugin::GrpcPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "digital twin playground".to_string(),
                ..default()
            }),
            ..default()
        }),
        PanOrbitCameraPlugin,
        #[cfg(feature = "embedded-model")]
        EmbeddedModelPlugin,
        EguiPlugin::default(),
        WorldInspectorPlugin::new(),
        PhysicsPlugins::default(),
        ConfigPlugin,
        GridPlugin,
        #[cfg(feature = "grpc")]
        GrpcPlugin {
            addr: "0.0.0.0:50051".to_string(),
        },
    ))
    .insert_resource(SubstepCount(12))
    .add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera::default(),
        Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
    ));

    // Ambient light
    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 2_000.0,
        ..default()
    });
}
