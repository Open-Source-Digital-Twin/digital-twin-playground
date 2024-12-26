//! This module provides a plugin for drawing a 2D grid on XZ plane.
use bevy::{color::palettes::css::GRAY, prelude::*};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

fn update(mut gizmos: Gizmos) {
    gizmos
        .grid_3d(Quat::IDENTITY, UVec3::new(10, 0, 10), Vec3::splat(1.), GRAY)
        .outer_edges();
}
