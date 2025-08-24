use serde::{Deserialize, Serialize};

use super::{ProjectID, TaskID};

/// CommentID describes the unique ID of a [`Comment`].
pub type CommentID = String;

/// ThreadID is the ID of the location where the comment is posted.
///
/// Used for creating comments — the request body uses `project_id` or `task_id`.
/// The v1 API response uses `item_id` for task comments instead.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ThreadID {
    /// The ID of the project this comment is attached to.
    Project {
        /// The ID of the [`super::Project`].
        project_id: ProjectID,
    },
    /// The ID of the task this comment is attached to.
    Task {
        /// The ID of the [`super::Task`].
        task_id: TaskID,
    },
}

/// Comment describes a Comment from the Todoist API v1.
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/api/v1/#comments).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    /// The unique ID of a comment.
    pub id: CommentID,
    /// The task this comment is attached to (v1 uses `item_id`).
    #[serde(default)]
    pub item_id: Option<TaskID>,
    /// The project this comment is attached to (if it's a project comment).
    #[serde(default)]
    pub project_id: Option<ProjectID>,
    /// The user who posted this comment.
    #[serde(default)]
    pub posted_uid: Option<String>,
    /// The date when the comment was posted.
    #[serde(default)]
    pub posted_at: Option<String>,
    /// Contains the comment text with markdown.
    pub content: String,
    /// Optional attachment file description.
    #[serde(default)]
    pub file_attachment: Option<serde_json::Value>,
    /// User IDs to notify about this comment.
    #[serde(default)]
    pub uids_to_notify: Option<serde_json::Value>,
    /// Whether this comment has been deleted.
    #[serde(default)]
    pub is_deleted: bool,
    /// Reactions on this comment (opaque for now).
    #[serde(default)]
    pub reactions: Option<serde_json::Value>,
}

/// CreateComment allows to create a new comment through the API.
///
/// The create request still uses `task_id`/`project_id` in the body.
#[derive(Debug, Serialize)]
pub struct CreateComment {
    /// The thread to attach the comment to.
    #[serde(flatten)]
    pub thread: ThreadID,
    /// The text of the comment. Supports markdown.
    pub content: String,
}
