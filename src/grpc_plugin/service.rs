use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use super::bridge::{JointRecord, MotorCommandMsg, SharedBridgeState};
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
        let joints = self.shared.joints.read().unwrap();
        let joints = joints
            .iter()
            .enumerate()
            .map(|(i, (name, record))| JointInfo {
                name: name.clone(),
                index: i as u32,
                joint_type: JointType::Revolute as i32,
                body1_name: String::new(),
                body2_name: String::new(),
                hinge_axis: Some(Vec3 {
                    x: record.hinge_axis.x,
                    y: record.hinge_axis.y,
                    z: record.hinge_axis.z,
                }),
                motor_controllable: record.motor_controllable,
            })
            .collect();

        Ok(Response::new(ListJointsResponse { joints }))
    }

    async fn get_joint_state(
        &self,
        request: Request<GetJointStateRequest>,
    ) -> Result<Response<JointState>, Status> {
        let req = request.into_inner();
        let joints = self.shared.joints.read().unwrap();
        let joint_name = resolve_joint_name(&joints, &req.joint).map_err(|status| *status)?;

        match joints.get(&joint_name) {
            Some(record) => Ok(Response::new(JointState {
                name: joint_name,
                angle: record.state.angle,
                angular_velocity: record.state.angular_velocity,
                motor_target_velocity: record.state.motor_target_velocity,
                motor_enabled: record.state.motor_enabled,
                timestamp: record.state.timestamp,
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
        let cmd = req
            .command
            .ok_or_else(|| Status::invalid_argument("MotorCommand is required"))?;

        let joint_name = {
            let joints = self.shared.joints.read().unwrap();
            let joint_name = resolve_joint_name(&joints, &req.joint).map_err(|status| *status)?;
            let Some(record) = joints.get(&joint_name) else {
                return Err(Status::not_found(format!(
                    "Joint '{}' not found",
                    joint_name
                )));
            };

            if !record.motor_controllable {
                return Err(Status::failed_precondition(format!(
                    "Joint '{}' is not motor controllable",
                    joint_name
                )));
            }

            joint_name
        };

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

        let requested_names = {
            let joints = self.shared.joints.read().unwrap();
            resolve_requested_joint_names(&joints, &req.joints).map_err(|status| *status)?
        };

        let shared = self.shared.clone();
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                // Collect snapshots while holding the lock, then drop it before awaiting sends.
                let snapshots: Vec<JointState> = {
                    let joints = shared.joints.read().unwrap();
                    joints
                        .iter()
                        .filter(|(name, _)| {
                            requested_names.is_empty() || requested_names.contains(*name)
                        })
                        .map(|(name, record)| JointState {
                            name: name.clone(),
                            angle: record.state.angle,
                            angular_velocity: record.state.angular_velocity,
                            motor_target_velocity: record.state.motor_target_velocity,
                            motor_enabled: record.state.motor_enabled,
                            timestamp: record.state.timestamp,
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

fn resolve_joint_name(
    joints: &BTreeMap<String, JointRecord>,
    joint_id: &Option<JointId>,
) -> Result<String, Box<Status>> {
    match joint_id {
        Some(JointId {
            id: Some(joint_id::Id::Name(name)),
        }) => {
            if joints.contains_key(name) {
                Ok(name.clone())
            } else {
                Err(Box::new(Status::not_found(format!(
                    "Joint '{}' not found",
                    name
                ))))
            }
        }
        Some(JointId {
            id: Some(joint_id::Id::Index(index)),
        }) => joints.keys().nth(*index as usize).cloned().ok_or_else(|| {
            Box::new(Status::not_found(format!(
                "Joint index {} not found",
                index
            )))
        }),
        _ => Err(Box::new(Status::invalid_argument("JointId is required"))),
    }
}

fn resolve_requested_joint_names(
    joints: &BTreeMap<String, JointRecord>,
    joint_ids: &[JointId],
) -> Result<BTreeSet<String>, Box<Status>> {
    joint_ids
        .iter()
        .map(|joint_id| resolve_joint_name(joints, &Some(joint_id.clone())))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::grpc_plugin::bridge::JointStateSnapshot;

    fn joint_record() -> JointRecord {
        JointRecord {
            hinge_axis: bevy::prelude::Vec3::Y,
            motor_controllable: true,
            state: JointStateSnapshot::default(),
        }
    }

    #[test]
    fn resolves_joint_name_by_index() {
        let joints = BTreeMap::from([
            (String::from("motor_joint"), joint_record()),
            (String::from("pendulum_joint"), joint_record()),
        ]);

        let joint_name = resolve_joint_name(
            &joints,
            &Some(JointId {
                id: Some(joint_id::Id::Index(1)),
            }),
        )
        .unwrap();

        assert_eq!(joint_name, "pendulum_joint");
    }

    #[test]
    fn rejects_out_of_range_joint_index() {
        let joints = BTreeMap::from([(String::from("motor_joint"), joint_record())]);

        let error = resolve_joint_name(
            &joints,
            &Some(JointId {
                id: Some(joint_id::Id::Index(9)),
            }),
        )
        .unwrap_err();

        assert_eq!(error.code(), tonic::Code::NotFound);
    }
}
