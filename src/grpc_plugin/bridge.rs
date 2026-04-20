use std::collections::HashMap;
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

/// A motor command sent from gRPC to Bevy.
#[derive(Debug)]
pub struct MotorCommandMsg {
    pub joint_name: String,
    pub target_velocity: f32,
    pub max_torque: f32,
    pub enabled: bool,
}

/// Shared state between the Bevy main loop and the gRPC server thread.
pub struct SharedBridgeState {
    /// Latest joint states — Bevy writes, gRPC reads.
    pub joint_states: RwLock<HashMap<String, JointStateSnapshot>>,
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
