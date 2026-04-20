use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};

use bevy::prelude::*;
use tokio::sync::mpsc;

/// Snapshot of a single joint's state, written by Bevy, read by gRPC.
#[derive(Clone, Default, Debug)]
pub struct JointStateSnapshot {
    pub angle: f32,
    pub angular_velocity: f32,
    pub motor_target_velocity: f32,
    pub motor_enabled: bool,
    pub timestamp: f64,
}

/// Joint metadata and last published state.
#[derive(Clone, Debug, Default)]
pub struct JointRecord {
    pub hinge_axis: Vec3,
    pub motor_controllable: bool,
    pub state: JointStateSnapshot,
}

/// A motor command sent from gRPC to Bevy.
#[derive(Debug)]
pub struct MotorCommandMsg {
    pub joint_name: String,
    pub target_velocity: f32,
    pub max_torque: f32,
    pub enabled: bool,
}

/// Marker for joints that may be actuated via the gRPC API.
#[derive(Component)]
pub struct GrpcControllableJoint;

/// Shared state between the Bevy main loop and the gRPC server thread.
pub struct SharedBridgeState {
    /// Latest joint metadata and states, keyed by stable joint name.
    pub joints: RwLock<BTreeMap<String, JointRecord>>,
    /// Sender for motor commands — gRPC sends, Bevy receives.
    pub command_tx: mpsc::Sender<MotorCommandMsg>,
    /// Receiver for motor commands — Bevy drains each frame.
    pub command_rx: Mutex<mpsc::Receiver<MotorCommandMsg>>,
}

/// Bevy resource that holds an `Arc` to the shared bridge state.
#[derive(Resource)]
pub struct GrpcBridge {
    pub shared: Arc<SharedBridgeState>,
}
