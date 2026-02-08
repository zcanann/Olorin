use serde::{Deserialize, Serialize};

/// Stateless trackable task command request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessTrackableTasksRequest {
    List(TrackableTasksListRequest),
    Cancel(TrackableTasksCancelRequest),
}

/// Stateless trackable task command response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessTrackableTasksResponse {
    List(TrackableTasksListResponse),
    Cancel(TrackableTasksCancelResponse),
}

/// Stateless request for listing currently tracked tasks.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackableTasksListRequest {}

/// Stateless response for listing currently tracked tasks.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackableTasksListResponse {}

/// Stateless request for canceling a tracked task.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackableTasksCancelRequest {
    pub task_id: String,
}

/// Stateless response for canceling a tracked task.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackableTasksCancelResponse {}
