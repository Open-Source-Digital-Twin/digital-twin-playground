use avian3d::prelude::*;
use bevy::prelude::*;

use super::bridge::GrpcBridge;

/// Publishes current joint states into the shared snapshot every frame.
///
/// For each `RevoluteJoint` entity with a `Name`, this reads the motor state
/// and computes the joint angle from the connected bodies' transforms.
pub fn publish_joint_states(
    bridge: Res<GrpcBridge>,
    joint_query: Query<(&RevoluteJoint, &Name)>,
    body_query: Query<(&GlobalTransform, Option<&AngularVelocity>)>,
    time: Res<Time>,
) {
    let mut states = bridge.shared.joint_states.write().unwrap();

    for (joint, name) in joint_query.iter() {
        let angle = compute_joint_angle(joint, &body_query);
        let ang_vel = compute_joint_angular_velocity(joint, &body_query);

        let snapshot = states.entry(name.to_string()).or_default();
        snapshot.angle = angle;
        snapshot.angular_velocity = ang_vel;
        snapshot.motor_target_velocity = joint.motor.target_velocity;
        snapshot.motor_enabled = joint.motor.enabled;
        snapshot.timestamp = time.elapsed_secs_f64();
    }
}

/// Drains motor commands from the gRPC channel and applies them to the corresponding joints.
pub fn apply_grpc_commands(
    bridge: Res<GrpcBridge>,
    mut joints: Query<(&mut RevoluteJoint, &Name)>,
) {
    let mut rx = bridge.shared.command_rx.lock().unwrap();
    while let Ok(cmd) = rx.try_recv() {
        for (mut joint, name) in joints.iter_mut() {
            if name.as_str() == cmd.joint_name {
                joint.motor.target_velocity = cmd.target_velocity;
                if cmd.max_torque > 0.0 {
                    joint.motor.max_torque = cmd.max_torque;
                }
                joint.motor.enabled = cmd.enabled;
                info!(
                    "gRPC: Applied motor command to '{}': vel={}, enabled={}",
                    cmd.joint_name, cmd.target_velocity, cmd.enabled
                );
            }
        }
    }
}

/// Computes the signed relative angle of a revolute joint around its hinge axis.
///
/// Uses twist decomposition: projects the quaternion's imaginary part onto the
/// hinge axis and computes the twist angle via `atan2`. Returns a value in (−π, π].
fn compute_joint_angle(
    joint: &RevoluteJoint,
    body_query: &Query<(&GlobalTransform, Option<&AngularVelocity>)>,
) -> f32 {
    let Ok((transform1, _)) = body_query.get(joint.body1) else {
        return 0.0;
    };
    let Ok((transform2, _)) = body_query.get(joint.body2) else {
        return 0.0;
    };

    let rot1 = transform1.compute_transform().rotation;
    let rot2 = transform2.compute_transform().rotation;
    let relative_rotation = rot2 * rot1.inverse();

    let axis: Vec3 = joint.hinge_axis.into();
    signed_angle_around_axis(relative_rotation, axis)
}

/// Extracts the signed rotation angle of `rotation` around `axis`.
///
/// Given a quaternion q = (w, x, y, z), the twist component around a unit
/// axis **a** has half-angle whose sine is dot((x,y,z), a) and whose cosine
/// is w.  Therefore the full twist angle is `2 * atan2(dot(xyz, a), w)`,
/// which naturally lies in (−π, π].
fn signed_angle_around_axis(rotation: Quat, axis: Vec3) -> f32 {
    let xyz = Vec3::new(rotation.x, rotation.y, rotation.z);
    let projection = xyz.dot(axis);
    2.0 * f32::atan2(projection, rotation.w)
}

/// Computes the relative angular velocity along the hinge axis.
fn compute_joint_angular_velocity(
    joint: &RevoluteJoint,
    body_query: &Query<(&GlobalTransform, Option<&AngularVelocity>)>,
) -> f32 {
    let vel1 = body_query
        .get(joint.body1)
        .ok()
        .and_then(|(_, v)| v)
        .map(|v| v.0)
        .unwrap_or(Vec3::ZERO);

    let vel2 = body_query
        .get(joint.body2)
        .ok()
        .and_then(|(_, v)| v)
        .map(|v| v.0)
        .unwrap_or(Vec3::ZERO);

    let relative_vel = vel2 - vel1;
    // Project onto the hinge axis
    relative_vel.dot(joint.hinge_axis)
}
