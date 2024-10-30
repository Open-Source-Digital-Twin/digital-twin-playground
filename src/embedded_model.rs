//! This module is an experiment of how to use ECS with Rapier3D.
//! It's main purpose is to check if it's possible to use physics with a model embedded in the scene.

use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_rapier3d::prelude::*;

use crate::config_plugin::KeyBindings;

pub struct EmbeddedModelPlugin;

impl Plugin for EmbeddedModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Motor>()
            // .add_systems(PreStartup, |mut rapier_config: ResMut<RapierConfiguration>| {
            //     rapier_config.physics_pipeline_active = false;
            // })
            .add_systems(Startup, add_rotary_interved_pendulum)
            // .add_systems(PostStartup, |mut rapier_config: ResMut<RapierConfiguration>| {
            //     rapier_config.physics_pipeline_active = true;
            // })
            .add_systems(
                Update,
                control_motor.run_if(resource_changed::<ButtonInput<KeyCode>>),
            )
            .add_systems(Update, get_pendulum_state);
    }
}

#[derive(Resource, Default)]
struct Motor {
    /// The entity of the joint. It's used to control the motor.
    joint_entity: Option<Entity>,
}

#[derive(Component)]
struct Name(String);

impl Name {
    fn new(name: &str) -> Self {
        Name(name.to_string())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}
/// This system is used to create the scene with embedded model.
fn add_rotary_interved_pendulum(
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

    let ground = commands
        .spawn((
            RigidBody::Fixed,
            TransformBundle::from(Transform::from_xyz(0.0, -GROUND_THICKNESS, 0.0)),
            Collider::cuboid(GROUND_SIDE_SIZE, GROUND_THICKNESS, GROUND_SIDE_SIZE),
        ))
        .id();

    let cube_1 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(CUBE_SIZE / 2.0, CUBE_SIZE / 2.0, CUBE_SIZE / 2.0),
            ColliderMassProperties::Mass(1.0),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE)),
                material: materials.add(Color::srgb_u8(124, 124, 124)),
                transform: Transform::from_xyz(0.0, CUBE_SIZE, 0.0),
                ..default()
            },
        ))
        .id();

    let fixed_joint_1 =
        FixedJointBuilder::new().local_anchor2(Vec3::new(0.0, CUBE_SIZE / 2.0, 0.0));

    commands
        .entity(ground)
        .insert(ImpulseJoint::new(cube_1, fixed_joint_1));

    let cylinder_1 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cylinder(CYLINDER_HEIGHT / 2.0, CYLINDER_RADIUS),
            ColliderMassProperties::Mass(1.0),
            LockedAxes::TRANSLATION_LOCKED
                | LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Z,
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cylinder {
                    radius: CYLINDER_RADIUS,
                    half_height: CYLINDER_HEIGHT / 2.0,
                })),
                material: materials.add(Color::srgb_u8(124, 124, 124)),
                transform: Transform::from_xyz(0.0, CUBE_SIZE + CYLINDER_HEIGHT / 2.0, 0.0),
                ..Default::default()
            },
        ))
        .id();

    let revolute_joint_1 = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(0.0, CUBE_SIZE / 2.0, 0.0))
        .local_anchor2(Vec3::new(0.0, CUBE_SIZE + CYLINDER_HEIGHT / 2.0, 0.0));

    let rev = commands
        .entity(cube_1)
        .insert(ImpulseJoint::new(cylinder_1, revolute_joint_1))
        .id();

    motor.joint_entity = Some(rev);

    let cube_2 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(CUBE_SIZE / 2.0, CUBE_SIZE / 2.0, CUBE_SIZE / 2.0),
            ColliderMassProperties::Mass(1.0),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE)),
                material: materials.add(Color::srgb_u8(124, 124, 124)),
                transform: Transform::from_xyz(
                    0.0,
                    CUBE_SIZE + CYLINDER_HEIGHT + CUBE_SIZE / 2.0,
                    0.0,
                ),
                ..Default::default()
            },
        ))
        .id();

    let fixed_joint_2 = FixedJointBuilder::new().local_anchor2(Vec3::new(
        0.0,
        CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0,
        0.0,
    ));

    commands
        .entity(cylinder_1)
        .insert(ImpulseJoint::new(cube_2, fixed_joint_2));

    let cylinder_2 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cylinder(CYLINDER_HEIGHT / 2.0, CYLINDER_RADIUS),
            LockedAxes::TRANSLATION_LOCKED_Y,
            ColliderMassProperties::Mass(1.0),
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cylinder {
                    radius: CYLINDER_RADIUS,
                    half_height: CYLINDER_HEIGHT / 2.0,
                })),
                material: materials.add(Color::srgb_u8(124, 124, 124)),
                transform: Transform::from_xyz(
                    0.0,
                    CUBE_SIZE + CYLINDER_HEIGHT + CUBE_SIZE / 2.0,
                    CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0,
                )
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
                ..Default::default()
            },
            Name::new("cylinder_2"),
        ))
        .id();

    let fixed_joint_3 = FixedJointBuilder::new()
        .local_basis2(Quat::from_rotation_x(std::f32::consts::PI / 2.0))
        .local_anchor2(Vec3::new(0.0, 0.0, CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0));

    commands
        .entity(cube_2)
        .insert(ImpulseJoint::new(cylinder_2, fixed_joint_3));

    let cube_3 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(CUBE_SIZE / 2.0, CUBE_SIZE / 2.0, CUBE_SIZE / 2.0),
            ColliderMassProperties::Mass(1.0),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE)),
                material: materials.add(Color::srgb_u8(124, 124, 124)),
                transform: Transform::from_xyz(
                    0.0,
                    CUBE_SIZE + CYLINDER_HEIGHT + CUBE_SIZE / 2.0,
                    CUBE_SIZE + CYLINDER_HEIGHT,
                ),
                ..Default::default()
            },
            Name::new("cube_3"),
        ))
        .id();

    let revolute_joint_2 = RevoluteJointBuilder::new(Vec3::Y).local_anchor2(Vec3::new(
        0.0,
        -(CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0),
        0.0,
    ));

    commands
        .entity(cube_3)
        .insert(ImpulseJoint::new(cylinder_2, revolute_joint_2));

    let cylinder_3 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cylinder(CYLINDER_HEIGHT / 2.0, CYLINDER_RADIUS),
            ColliderMassProperties::Mass(1.0),
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cylinder {
                    radius: CYLINDER_RADIUS,
                    half_height: CYLINDER_HEIGHT / 2.0,
                })),
                material: materials.add(Color::srgb_u8(124, 124, 124)),
                transform: Transform::from_xyz(
                    0.0,
                    CUBE_SIZE + CYLINDER_HEIGHT / 2.0,
                    CUBE_SIZE / 2.0 + CYLINDER_HEIGHT + CUBE_SIZE / 2.0,
                ),
                ..Default::default()
            },
            Name::new("cylinder_3"),
        ))
        .id();

    let fixed_joint_3 = FixedJointBuilder::new()
        .local_basis2(Quat::from_rotation_x(std::f32::consts::PI / 2.0))
        .local_anchor2(Vec3::new(
            0.0,
            -(CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0),
            0.0,
        ));

    commands
        .entity(cylinder_3)
        .insert(ImpulseJoint::new(cube_3, fixed_joint_3));
}

/// This system is used to control the motor.
fn control_motor(
    key: Res<ButtonInput<KeyCode>>,
    motor: ResMut<Motor>,
    mut query: Query<&mut ImpulseJoint>,
    key_bindings: Res<Persistent<KeyBindings>>,
) {
    match motor.joint_entity {
        Some(entity) => {
            let velocity = 10.0;
            let factor = 10000.0;
            let mut joint = query.get_mut(entity).unwrap();
            if key.just_pressed(key_bindings.rotate_clockwise) {
                joint
                    .data
                    .as_mut()
                    .as_revolute_mut()
                    .unwrap()
                    .set_motor_velocity(velocity, factor);
            } else if key.just_pressed(key_bindings.rotate_counter_clockwise) {
                joint
                    .data
                    .as_mut()
                    .as_revolute_mut()
                    .unwrap()
                    .set_motor_velocity(-velocity, factor);
            } else if key.just_pressed(KeyCode::ArrowDown) {
                debug!("Stop");
                joint
                    .data
                    .as_mut()
                    .as_revolute_mut()
                    .unwrap()
                    .set_motor_velocity(0.0, factor);
            }
        }
        _ => {
            warn!("No joint entity");
        }
    }
}

fn get_pendulum_state(query: Query<(&Transform, &Name)>) {
    let mut cube_3_transform = None;
    let mut cylinder_2_transform = None;

    for (transform, name) in query.iter() {
        match name.as_str() {
            "cube_3" => cube_3_transform = Some(*transform),
            "cylinder_2" => cylinder_2_transform = Some(*transform),
            _ => {}
        }
    }

    if let (Some(cube_3_transform), Some(cylinder_2_transform)) =
        (cube_3_transform, cylinder_2_transform)
    {
        // Calculate the relative rotation
        let relative_rotation = cube_3_transform.rotation * cylinder_2_transform.rotation.inverse();

        // The angle of the relative rotation is between 0 and 2*PI.
        let (_axis, angle) = relative_rotation.to_axis_angle();

        println!("Relative angle: {:?}", angle);
    } else {
        println!("cube_3 or cylinder_2 not found");
    }
}
