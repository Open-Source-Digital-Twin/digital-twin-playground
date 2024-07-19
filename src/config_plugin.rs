//! This file contains the implementation of a Bevy plugin for managing configuration settings.
//! It defines the `ConfigPlugin` struct and implements the `Plugin` trait for it.
//! The plugin adds systems for startup and setup to the Bevy application.
//! The `setup` function initializes the key bindings resource using the `Persistent` builder.
use bevy::prelude::*;
use bevy_persistent::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

/// Represents the key bindings configuration.
#[derive(Debug, Deserialize, Resource, Serialize)]
pub struct KeyBindings {
    pub rotate_clockwise: KeyCode,
    pub rotate_counter_clockwise: KeyCode,
}

/// Sets up the key bindings resource using the `Persistent` builder.
fn setup(mut commands: Commands) {
    let config_dir = dirs::config_dir()
        .map(|native_config_dir| native_config_dir.join(env!("CARGO_PKG_NAME")))
        .unwrap_or(Path::new("local").join("configuration")); // Fallback to `local/configuration` when using WebAssembly
    commands.insert_resource(
        Persistent::<KeyBindings>::builder()
            .name("key_bindings")
            .format(StorageFormat::Json)
            .path(config_dir.join("key_bindings.json"))
            .default(KeyBindings {
                rotate_clockwise: KeyCode::ArrowLeft,
                rotate_counter_clockwise: KeyCode::ArrowRight,
            })
            .build()
            .expect("Failed to initialize key bindings."),
    )
}
