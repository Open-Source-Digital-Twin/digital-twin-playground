use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::Vector};

pub struct EmbeddedModelPlugin;

impl Plugin for EmbeddedModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_rotary_interved_pendulum);
    }
}

fn add_rotary_interved_pendulum(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const GROUND_THICKNESS: f32 = 0.01;
    const GROUND_SIDE_SIZE: f32 = 100.0;
    const CUBE_SIZE: f32 = 1.0;
    const CYLINDER_RADIUS: f32 = 0.25;
    const CYLINDER_HEIGHT: f32 = 3.0;
    const SPHERE_RADIUS: f32 = 1.0;

    // let mut body_entities: Vec<_> = Vec::new();

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
                mesh: meshes.add(Mesh::from(shape::Cube { size: CUBE_SIZE })),
                material: materials.add(Color::GRAY.into()),
                transform: Transform::from_xyz(0.0, CUBE_SIZE, 0.0),
                ..Default::default()
            },
        ))
        .id();

    let fixed_joint_1 =
        FixedJointBuilder::new().local_anchor2(Vec3::new(0.0, CUBE_SIZE / 2.0, 0.0));

    commands.entity(ground).with_children(|children| {
        children.spawn(ImpulseJoint::new(cube_1, fixed_joint_1));
    });

    let cylinder_1 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cylinder(CYLINDER_HEIGHT / 2.0, CYLINDER_RADIUS),
            ColliderMassProperties::Mass(1.0),
            LockedAxes::TRANSLATION_LOCKED | LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: CYLINDER_RADIUS,
                    height: CYLINDER_HEIGHT,
                    ..Default::default()
                })),
                material: materials.add(Color::GRAY.into()),
                transform: Transform::from_xyz(0.0, CUBE_SIZE + CYLINDER_HEIGHT / 2.0, 0.0),
                ..Default::default()
            },
        ))
        .id();

    let revolute_joint_1 = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(0.0, CUBE_SIZE / 2.0, 0.0))
        .local_anchor2(Vec3::new(0.0, CUBE_SIZE + CYLINDER_HEIGHT / 2.0, 0.0))
        .motor_velocity(1000.0, 0.5);

    commands.entity(cube_1).with_children(|children| {
        children.spawn(ImpulseJoint::new(cylinder_1, revolute_joint_1));
    });

    let cube_2 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(CUBE_SIZE / 2.0, CUBE_SIZE / 2.0, CUBE_SIZE / 2.0),
            ColliderMassProperties::Mass(1.0),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: CUBE_SIZE })),
                material: materials.add(Color::GRAY.into()),
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

    commands.entity(cylinder_1).with_children(|children| {
        children.spawn(ImpulseJoint::new(cube_2, fixed_joint_2));
    });

    let cylinder_2 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cylinder(CYLINDER_HEIGHT / 2.0, CYLINDER_RADIUS),
            ColliderMassProperties::Mass(1.0),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: CYLINDER_RADIUS,
                    height: CYLINDER_HEIGHT,
                    ..Default::default()
                })),
                material: materials.add(Color::GRAY.into()),
                transform: Transform::from_xyz(
                    0.0,
                    CUBE_SIZE + CYLINDER_HEIGHT + CUBE_SIZE / 2.0,
                    CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0,
                )
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
                ..Default::default()
            },
        ))
        .id();

    let fixed_joint_3 = FixedJointBuilder::new()
        .local_basis2(Quat::from_rotation_x(std::f32::consts::PI / 2.0))
        .local_anchor2(Vec3::new(0.0, 0.0, CUBE_SIZE / 2.0 + CYLINDER_HEIGHT / 2.0));

    commands.entity(cube_2).with_children(|children| {
        children.spawn(ImpulseJoint::new(cylinder_2, fixed_joint_3));
    });

    let cube_3 = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(CUBE_SIZE / 2.0, CUBE_SIZE / 2.0, CUBE_SIZE / 2.0),
            ColliderMassProperties::Mass(1.0),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: CUBE_SIZE })),
                material: materials.add(Color::GRAY.into()),
                transform: Transform::from_xyz(
                    0.0,
                    CUBE_SIZE + CYLINDER_HEIGHT + CUBE_SIZE / 2.0,
                    CUBE_SIZE + CYLINDER_HEIGHT,
                ),
                ..Default::default()
            },
        ))
        .id();

    let revolute_joint_2 = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor2(Vec3::new(
            0.0,
            -(CUBE_SIZE / 2.0  + CYLINDER_HEIGHT / 2.0) ,
            0.0,
        ));

    commands.entity(cube_3).with_children(|children| {
        children.spawn(ImpulseJoint::new(cylinder_2, revolute_joint_2));
    });
}

// fn add_ground(mut commands: Commands) {

// }
