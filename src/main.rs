//! A application to control and emulate mechanical systems and play as a workbench for
//! physics simulations.
//!
//! Just run `cargo run --release`, and you should see a window with a basic example.

use bevy::{prelude::*, window::WindowPlugin};

#[cfg(feature = "blender")]
use bevy::{
    math::Vec3A,
    render::primitives::{Aabb, Sphere},
};

use bevy_infinite_grid::{InfiniteGrid, InfiniteGridBundle, InfiniteGridPlugin};
// use bevy_infinite_grid::{InfiniteGrid, InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
#[cfg(feature = "embedded-model")]
mod embedded_model;
#[cfg(feature = "blender")]
mod scene_viewer_plugin;

mod config_plugin;

use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
#[cfg(feature = "embedded-model")]
use embedded_model::EmbeddedModelPlugin;
#[cfg(feature = "blender")]
use scene_viewer_plugin::{SceneHandle, SceneViewerPlugin};

use config_plugin::ConfigPlugin;

fn main() {
    let mut app = App::new();
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0 / 5.0f32,
    })
    .add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "bevy scene viewer".to_string(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()),
                ..default()
            }),
        PanOrbitCameraPlugin,
        #[cfg(feature = "blender")]
        SceneViewerPlugin,
        #[cfg(feature = "embedded-model")]
        EmbeddedModelPlugin,
        WorldInspectorPlugin::new(),
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        ConfigPlugin,
        InfiniteGridPlugin,
    ))
    .add_systems(Startup, setup);

    #[cfg(feature = "blender")]
    app.add_systems(PreUpdate, setup_scene_after_load);

    app.run();
}

fn _parse_scene(scene_path: String) -> (String, usize) {
    if scene_path.contains('#') {
        let gltf_and_scene = scene_path.split('#').collect::<Vec<_>>();
        if let Some((last, path)) = gltf_and_scene.split_last() {
            if let Some(index) = last
                .strip_prefix("Scene")
                .and_then(|index| index.parse::<usize>().ok())
            {
                return (path.join("#"), index);
            }
        }
    }
    (scene_path, 0)
}

#[cfg(feature = "blender")]
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let scene_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "3d-models/rotary-inverted-pendulum/rotary_pendulum.glb".to_string());
    info!("Loading {}", scene_path);
    let (file_path, scene_index) = parse_scene(scene_path);
    commands.insert_resource(SceneHandle::new(asset_server.load(file_path), scene_index));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
            ..default()
        },
        PanOrbitCamera::default(),
        EnvironmentMapLight {
            diffuse_map: asset_server.load("assets/environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server
                .load("assets/environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 1.0,
        },
    ));
    commands.spawn(InfiniteGridBundle {
        grid: InfiniteGrid {
            // shadow_color: None,
            // ..default()
        },
        ..default()
    });
}

#[cfg(feature = "blender")]
fn setup_scene_after_load(
    mut commands: Commands,
    mut setup: Local<bool>,
    mut scene_handle: ResMut<SceneHandle>,
    asset_server: Res<AssetServer>,
    meshes: Query<(&GlobalTransform, Option<&Aabb>), With<Handle<Mesh>>>,
) {
    if scene_handle.is_loaded && !*setup {
        *setup = true;
        // Find an approximate bounding box of the scene from its meshes
        if meshes.iter().any(|(_, maybe_aabb)| maybe_aabb.is_none()) {
            return;
        }

        let mut min = Vec3A::splat(f32::MAX);
        let mut max = Vec3A::splat(f32::MIN);
        for (transform, maybe_aabb) in &meshes {
            let aabb = maybe_aabb.unwrap();
            // If the Aabb had not been rotated, applying the non-uniform scale would produce the
            // correct bounds. However, it could very well be rotated and so we first convert to
            // a Sphere, and then back to an Aabb to find the conservative min and max points.
            let sphere = Sphere {
                center: Vec3A::from(transform.transform_point(Vec3::from(aabb.center))),
                radius: transform.radius_vec3a(aabb.half_extents),
            };
            let aabb = Aabb::from(sphere);
            min = min.min(aabb.min());
            max = max.max(aabb.max());
        }

        // Display the controls of the scene viewer
        info!("{}", *scene_handle);

        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
                ..default()
            },
            PanOrbitCamera::default(),
            EnvironmentMapLight {
                diffuse_map: asset_server
                    .load("assets/environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
                specular_map: asset_server
                    .load("assets/environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            },
        ));

        // Spawn a default light if the scene does not have one
        if !scene_handle.has_light {
            info!("Spawning a directional light");
            commands.spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    shadows_enabled: false,
                    ..default()
                },
                ..default()
            });

            scene_handle.has_light = true;
        }

        commands.spawn(InfiniteGridBundle {
            grid: InfiniteGrid {
                // shadow_color: None,
                ..default()
            },
            ..default()
        });
    }
}
