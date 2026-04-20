//! This module implements the Furuta pendulum using Avian physics.
//! It spawns a pendulum with revolute joints and motor control.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::config_plugin::KeyBindings;

pub struct EmbeddedModelPlugin;

impl Plugin for EmbeddedModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Motor>()
            .add_systems(Startup, add_rotary_inverted_pendulum)
            .add_systems(Update, control_motor)
            .add_systems(Update, get_pendulum_state);
    }
}

#[derive(Resource, Default)]
struct Motor {
    /// The entity of the revolute joint. Used to control the motor.
    joint_entity: Option<Entity>,
}

/// This system creates the Furuta pendulum scene using Avian physics.
///
/// The pendulum has only 3 rigid bodies and 2 revolute joints (no fixed joints):
///
///   Body 0: cube_1 (static base)          — center (0, 0.5, 0)
///   Body 1: motor_arm (compound body)     — center (0, 2.5, 0)
///           ├─ cylinder_1 (parent collider) — vertical shaft, y ∈ [1, 4]
///           ├─ cube_2     (child collider)  — connector block at y=4.5
///           └─ cylinder_2 (child collider)  — horizontal arm, rotated 90° X, extends +Z
///   Body 2: pendulum (compound body)      — center (0, 4.5, 4.0)
///           ├─ cube_3     (parent collider) — pivot block at (0, 4.5, 4.0)
///           ├─ cylinder_3 (child collider)  — pendulum rod, hangs down
///           └─ sphere     (child collider)  — weighted tip at bottom of rod
///
/// Joints:
///   Revolute + motor:  cube_1 ↔ motor_arm   (hinge Y, pivot at (0, 1, 0))
///   Revolute (passive): motor_arm ↔ pendulum (hinge Z, pivot at (0, 4.5, 3.5))
///
/// Using compound bodies eliminates fixed joints, which are soft XPBD
/// constraints that bend under motor torque.
fn add_rotary_inverted_pendulum(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut motor: ResMut<Motor>,
) {
    const GROUND_THICKNESS: f32 = 0.01;
    const GROUND_SIDE_SIZE: f32 = 100.0;
    const CUBE_SIZE: f32 = 1.0;
    const CYLINDER_RADIUS: f32 = 0.25;
    const CYLINDER_HEIGHT: f32 = 3.0;
    const SPHERE_RADIUS: f32 = 0.5;

    let grey = materials.add(Color::srgb_u8(124, 124, 124));

    // Ground — static collision floor
    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(0.0, -GROUND_THICKNESS, 0.0),
        Collider::cuboid(
            2.0 * GROUND_SIDE_SIZE,
            2.0 * GROUND_THICKNESS,
            2.0 * GROUND_SIDE_SIZE,
        ),
    ));

    // Cube 1 — static base support (center y=0.5)
    let cube_1 = commands
        .spawn((
            RigidBody::Static,
            Collider::cuboid(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
            Mesh3d(meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE))),
            MeshMaterial3d(grey.clone()),
            Transform::from_xyz(0.0, CUBE_SIZE / 2.0, 0.0),
        ))
        .id();

    // Motor arm — compound body: cylinder_1 + cube_2 + cylinder_2
    // Parent center at (0, 2.5, 0) = cylinder_1 center
    let motor_arm = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cylinder(CYLINDER_RADIUS, CYLINDER_HEIGHT),
            Mesh3d(meshes.add(Cylinder::new(CYLINDER_RADIUS, CYLINDER_HEIGHT))),
            MeshMaterial3d(grey.clone()),
            Transform::from_xyz(0.0, CUBE_SIZE + CYLINDER_HEIGHT / 2.0, 0.0),
            Name::new("motor_arm"),
            SleepingDisabled,
            children![
                // Cube 2 — connector block, relative offset (0, 2.0, 0) from parent
                (
                    Collider::cuboid(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
                    Mesh3d(meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE))),
                    MeshMaterial3d(grey.clone()),
                    Transform::from_xyz(0.0, CYLINDER_HEIGHT / 2.0 + CUBE_SIZE / 2.0, 0.0),
                ),
                // Cylinder 2 — horizontal arm, relative offset (0, 2.0, 2.0) rotated 90° X
                (
                    Collider::cylinder(CYLINDER_RADIUS, CYLINDER_HEIGHT),
                    Mesh3d(meshes.add(Cylinder::new(CYLINDER_RADIUS, CYLINDER_HEIGHT))),
                    MeshMaterial3d(grey.clone()),
                    Transform::from_xyz(
                        0.0,
                        CYLINDER_HEIGHT / 2.0 + CUBE_SIZE / 2.0,
                        CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0,
                    )
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    Name::new("cylinder_2"),
                ),
            ],
        ))
        .id();

    // Revolute joint with motor: cube_1 ↔ motor_arm
    // Pivot at world (0, 1, 0) — top of cube_1 / bottom of motor_arm
    let joint_entity = commands
        .spawn((
            RevoluteJoint::new(cube_1, motor_arm)
                .with_hinge_axis(Vec3::Y)
                .with_local_anchor1(Vec3::new(0.0, CUBE_SIZE / 2.0, 0.0))
                .with_local_anchor2(Vec3::new(0.0, -CYLINDER_HEIGHT / 2.0, 0.0))
                .with_motor(AngularMotor {
                    target_velocity: 0.0,
                    max_torque: 10_000.0,
                    motor_model: MotorModel::AccelerationBased {
                        stiffness: 0.0,
                        damping: 10.0,
                    },
                    ..default()
                }),
            JointCollisionDisabled,
            Name::new("motor_joint"),
        ))
        .id();
    motor.joint_entity = Some(joint_entity);

    // Pendulum — compound body: cube_3 + cylinder_3
    // Parent center at (0, 4.5, 4.0) = cube_3 center
    let pendulum = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
            Mesh3d(meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE))),
            MeshMaterial3d(grey.clone()),
            Transform::from_xyz(
                0.0,
                CUBE_SIZE + CYLINDER_HEIGHT + CUBE_SIZE / 2.0,
                CUBE_SIZE + CYLINDER_HEIGHT,
            ),
            Name::new("pendulum"),
            SleepingDisabled,
            children![
                // Cylinder 3 — pendulum rod, relative offset (0, -2.0, 0)
                (
                    Collider::cylinder(CYLINDER_RADIUS, CYLINDER_HEIGHT),
                    Mesh3d(meshes.add(Cylinder::new(CYLINDER_RADIUS, CYLINDER_HEIGHT))),
                    MeshMaterial3d(grey.clone()),
                    Transform::from_xyz(0.0, -(CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0), 0.0),
                ),
                // Sphere — weighted tip at the bottom of the pendulum rod
                (
                    Collider::sphere(SPHERE_RADIUS),
                    Mesh3d(meshes.add(Sphere::new(SPHERE_RADIUS))),
                    MeshMaterial3d(grey.clone()),
                    Transform::from_xyz(
                        0.0,
                        -(CUBE_SIZE / 2.0 + CYLINDER_HEIGHT + SPHERE_RADIUS),
                        0.0,
                    ),
                    Name::new("pendulum_weight"),
                ),
            ],
        ))
        .id();

    // Revolute joint: motor_arm ↔ pendulum (passive pendulum pivot)
    // Pivot at world (0, 4.5, 3.5) — far end of horizontal arm / -Z face of pendulum
    // Both bodies have identity rotation, so hinge_axis Z is directly in local space.
    commands.spawn((
        RevoluteJoint::new(motor_arm, pendulum)
            .with_hinge_axis(Vec3::Z)
            .with_local_anchor1(Vec3::new(
                0.0,
                CYLINDER_HEIGHT / 2.0 + CUBE_SIZE / 2.0,
                CUBE_SIZE / 2.0 + CYLINDER_HEIGHT - CUBE_SIZE / 2.0,
            ))
            .with_local_anchor2(Vec3::new(0.0, 0.0, -CUBE_SIZE / 2.0)),
        JointCollisionDisabled,
        Name::new("pendulum_joint"),
    ));
}

/// Controls the motor velocity via keyboard input.
fn control_motor(
    key: Res<ButtonInput<KeyCode>>,
    motor: Res<Motor>,
    mut query: Query<&mut RevoluteJoint>,
    key_bindings: Res<Persistent<KeyBindings>>,
) {
    match motor.joint_entity {
        Some(entity) => {
            let velocity = 5.0;
            let Ok(mut joint) = query.get_mut(entity) else {
                warn!("Motor joint entity not found in query");
                return;
            };
            if key.just_pressed(key_bindings.rotate_clockwise) {
                joint.motor.target_velocity = velocity;
                joint.motor.enabled = true;
            } else if key.just_pressed(key_bindings.rotate_counter_clockwise) {
                joint.motor.target_velocity = -velocity;
                joint.motor.enabled = true;
            } else if key.just_pressed(KeyCode::ArrowDown) {
                debug!("Stop");
                joint.motor.target_velocity = 0.0;
            }
        }
        _ => {
            warn!("No joint entity");
        }
    }
}

fn get_pendulum_state(query: Query<(&GlobalTransform, &Name)>) {
    let mut pendulum_transform = None;
    let mut motor_arm_transform = None;

    for (transform, name) in query.iter() {
        match name.as_str() {
            "pendulum" => pendulum_transform = Some(transform.compute_transform()),
            "motor_arm" => motor_arm_transform = Some(transform.compute_transform()),
            _ => {}
        }
    }

    if let (Some(pendulum), Some(motor_arm)) = (pendulum_transform, motor_arm_transform) {
        // Calculate the relative rotation
        let relative_rotation = pendulum.rotation * motor_arm.rotation.inverse();

        // The angle of the relative rotation is between 0 and 2*PI.
        let (_axis, angle) = relative_rotation.to_axis_angle();

        println!("Relative angle: {angle:?}");
    } else {
        println!("pendulum or motor_arm not found");
    }
}
