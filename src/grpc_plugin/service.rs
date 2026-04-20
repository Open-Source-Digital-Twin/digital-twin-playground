use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use super::bridge::{MotorCommandMsg, SharedBridgeState};
use super::proto::joint_control_server::JointControl;
use super::proto::*;

pub struct JointControlService {
    pub shared: Arc<SharedBridgeState>,
}

#[tonic::async_trait]
impl JointControl for JointControlService {
    async fn list_joints(
        &self,
        _request: Request<ListJointsRequest>,
    ) -> Result<Response<ListJointsResponse>, Status> {
        let states = self.shared.joint_states.read().unwrap();
        let joints = states
            .keys()
            .enumerate()
            .map(|(i, name)| JointInfo {
                name: name.clone(),
                index: i as u32,
                joint_type: JointType::Revolute as i32,
                body1_name: String::new(),
                body2_name: String::new(),
                hinge_axis: None,
            })
            .collect();

        Ok(Response::new(ListJointsResponse { joints }))
    }

    async fn get_joint_state(
        &self,
        request: Request<GetJointStateRequest>,
    ) -> Result<Response<JointState>, Status> {
        let req = request.into_inner();
        let joint_name = extract_joint_name(&req.joint).map_err(|e| *e)?;

        let states = self.shared.joint_states.read().unwrap();
        match states.get(&joint_name) {
            Some(s) => Ok(Response::new(JointState {
                name: joint_name,
                angle: s.angle,
                angular_velocity: s.angular_velocity,
                motor_target_velocity: s.motor_target_velocity,
                motor_enabled: s.motor_enabled,
                timestamp: s.timestamp,
            })),
            None => Err(Status::not_found(format!(
                "Joint '{}' not found",
                joint_name
            ))),
        }
    }

    async fn set_motor_command(
        &self,
        request: Request<SetMotorCommandRequest>,
    ) -> Result<Response<SetMotorCommandResponse>, Status> {
        let req = request.into_inner();
        let joint_name = extract_joint_name(&req.joint).map_err(|e| *e)?;
        let cmd = req
            .command
            .ok_or_else(|| Status::invalid_argument("MotorCommand is required"))?;

        self.shared
            .command_tx
            .send(MotorCommandMsg {
                joint_name: joint_name.clone(),
                target_velocity: cmd.target_velocity,
                max_torque: cmd.max_torque,
                enabled: cmd.enabled,
            })
            .await
            .map_err(|_| Status::internal("Command channel closed"))?;

        Ok(Response::new(SetMotorCommandResponse {
            success: true,
            message: format!("Command sent to '{}'", joint_name),
        }))
    }

    type StreamJointStatesStream = ReceiverStream<Result<JointState, Status>>;

    async fn stream_joint_states(
        &self,
        request: Request<StreamJointStatesRequest>,
    ) -> Result<Response<Self::StreamJointStatesStream>, Status> {
        let req = request.into_inner();
        let rate_hz = if req.rate_hz > 0.0 { req.rate_hz } else { 60.0 };
        let interval = std::time::Duration::from_secs_f32(1.0 / rate_hz);

        // Collect requested joint names (empty = all joints).
        let requested_names: Vec<String> = req
            .joints
            .iter()
            .filter_map(|jid| match &jid.id {
                Some(joint_id::Id::Name(n)) => Some(n.clone()),
                _ => None,
            })
            .collect();

        let shared = self.shared.clone();
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                // Collect snapshots while holding the lock, then drop it before awaiting sends.
                let snapshots: Vec<JointState> = {
                    let states = shared.joint_states.read().unwrap();
                    states
                        .iter()
                        .filter(|(name, _)| {
                            requested_names.is_empty() || requested_names.contains(name)
                        })
                        .map(|(name, s)| JointState {
                            name: name.clone(),
                            angle: s.angle,
                            angular_velocity: s.angular_velocity,
                            motor_target_velocity: s.motor_target_velocity,
                            motor_enabled: s.motor_enabled,
                            timestamp: s.timestamp,
                        })
                        .collect()
                };

                for state in snapshots {
                    if tx.send(Ok(state)).await.is_err() {
                        return; // Client disconnected.
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

/// Extract the joint name from a `JointId` option.
fn extract_joint_name(joint_id: &Option<JointId>) -> Result<String, Box<Status>> {
    match joint_id {
        Some(JointId {
            id: Some(joint_id::Id::Name(name)),
        }) => Ok(name.clone()),
        Some(JointId {
            id: Some(joint_id::Id::Index(_)),
        }) => Err(Box::new(Status::unimplemented(
            "Index-based joint lookup not yet implemented",
        ))),
        _ => Err(Box::new(Status::invalid_argument("JointId is required"))),
    }
}
