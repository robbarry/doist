use core::fmt;
use std::fmt::Display;

use crate::api::tree::Treeable;
use chrono::Utc;
use owo_colors::{OwoColorize, Stream};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::{ProjectID, SectionID};

/// TaskID describes the unique ID of a [`Task`].
pub type TaskID = String;
/// UserID is the unique ID of a User.
pub type UserID = String;

/// Task describes a Task from the Todoist API v1.
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/api/v1/#tasks).
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Task {
    /// Unique ID of a Task.
    pub id: TaskID,
    /// The user who owns this task.
    #[serde(default)]
    pub user_id: Option<UserID>,
    /// Shows which [`super::Project`] the Task belongs to.
    pub project_id: ProjectID,
    /// Set if the Task is also in a subsection of a Project.
    pub section_id: Option<SectionID>,
    /// If set, this Task is a subtask of another.
    pub parent_id: Option<TaskID>,
    /// The user who added this task.
    #[serde(default)]
    pub added_by_uid: Option<UserID>,
    /// The user who assigned this task.
    #[serde(default)]
    pub assigned_by_uid: Option<UserID>,
    /// The user responsible for this task.
    #[serde(default)]
    pub responsible_uid: Option<UserID>,
    /// The main content of the Task, also known as Task name.
    pub content: String,
    /// Description is the description found under the content.
    pub description: String,
    /// All associated [`super::Label`]s to this Task. Just label names are used here.
    pub labels: Vec<String>,
    /// Whether this task is checked (completed).
    #[serde(default)]
    pub checked: bool,
    /// Whether this task has been deleted.
    #[serde(default)]
    pub is_deleted: bool,
    /// When the task was added.
    #[serde(default)]
    pub added_at: Option<String>,
    /// When the task was completed, if ever.
    #[serde(default)]
    pub completed_at: Option<String>,
    /// Who completed the task.
    #[serde(default)]
    pub completed_by_uid: Option<UserID>,
    /// When the task was last updated.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Order within subtasks of a parent or within a project/section.
    #[serde(default)]
    pub child_order: isize,
    /// Priority is how urgent the task is.
    pub priority: Priority,
    /// The due date of the Task.
    pub due: Option<DueDate>,
    /// Number of comments/notes on this task.
    #[serde(default)]
    pub note_count: usize,
    /// Day order for today/upcoming views.
    #[serde(default)]
    pub day_order: isize,
    /// Whether subtasks are collapsed in the UI.
    #[serde(default)]
    pub is_collapsed: bool,
    /// Deadline information (opaque for now).
    #[serde(default)]
    pub deadline: Option<serde_json::Value>,
    /// Duration information (opaque for now).
    #[serde(default)]
    pub duration: Option<serde_json::Value>,
}

impl Treeable for Task {
    type ID = TaskID;

    fn id(&self) -> TaskID {
        self.id.clone()
    }

    fn parent_id(&self) -> Option<TaskID> {
        self.parent_id.clone()
    }

    fn reset_parent(&mut self) {
        self.parent_id = None;
    }
}

impl Ord for Task {
    /// Sorts on a best-attempt to make it sort similar to the Todoist UI.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Tasks with a timezone have an exact time — sort those first
        // TODO: In v1, the exact time derivation from date+timezone is unclear.
        // For now, we treat tasks with timezone as having exact times and sort
        // them by date string, which should be correct for date-only comparisons.
        match (
            self.due
                .as_ref()
                .and_then(|d| d.timezone.as_ref().map(|_| &d.date)),
            other
                .due
                .as_ref()
                .and_then(|d| d.timezone.as_ref().map(|_| &d.date)),
        ) {
            (Some(left), Some(right)) => match left.cmp(right) {
                std::cmp::Ordering::Equal => {}
                ord => return ord,
            },
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }

        // Lower priority in API is lower in list
        match self.priority.cmp(&other.priority).reverse() {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.child_order.cmp(&other.child_order) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Priority as is given from the Todoist API.
///
/// 1 for Normal up to 4 for Urgent.
#[derive(
    Default, Debug, Copy, Clone, Serialize_repr, Deserialize_repr, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(u8)]
pub enum Priority {
    /// p1 in the Todoist UI.
    #[default]
    Normal = 1,
    /// p3 in the Todoist UI.
    High = 2,
    /// p2 in the Todoist UI.
    VeryHigh = 3,
    /// p1 in the Todoist UI.
    Urgent = 4,
}

impl Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The priority display is reversed as in the actual desktop client compared to the API.
        match self {
            Priority::Normal => write!(f, "p4"),
            Priority::High => write!(
                f,
                "{}",
                "p3".if_supports_color(Stream::Stdout, |text| text.blue())
            ),
            Priority::VeryHigh => write!(
                f,
                "{}",
                "p2".if_supports_color(Stream::Stdout, |text| text.yellow())
            ),
            Priority::Urgent => write!(
                f,
                "{}",
                "p1".if_supports_color(Stream::Stdout, |text| text.red())
            ),
        }
    }
}

/// DueDate is the Due object from the Todoist API v1.
///
/// In v1, the due object has `date`, `timezone`, `string`, `lang`, and `is_recurring`.
/// When `timezone` is Some, it indicates an exact-time task. The exact datetime
/// derivation from date + timezone is a TODO — for now we display the date and
/// timezone separately.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DueDate {
    /// Human-readable form of the due date.
    #[serde(rename = "string")]
    pub string: String,
    /// The date on which the Task is due (date-only string like "2022-08-27").
    pub date: String,
    /// Lets us know if it is recurring (reopens after close).
    pub is_recurring: bool,
    /// If set, this task has an exact time. The timezone name (e.g. "Europe/Athens").
    #[serde(default)]
    pub timezone: Option<String>,
    /// Language hint for the human-readable string.
    #[serde(default)]
    pub lang: Option<String>,
}

impl DueDate {
    /// Returns the date portion parsed from Todoist's v1 due-date string, if valid.
    ///
    /// Todoist v1 returns date-only values (`YYYY-MM-DD`), floating date-times
    /// (`YYYY-MM-DDTHH:MM:SS[.ffffff]`) and fixed-zone date-times
    /// (`YYYY-MM-DDTHH:MM:SS[.ffffff]Z`) in the same `date` field.
    pub fn naive_date(&self) -> Option<chrono::NaiveDate> {
        self.date
            .get(..10)
            .and_then(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
    }

    /// Returns a display string for the due date including timezone info if present.
    pub fn display_date(&self) -> String {
        if let Some(tz_str) = &self.timezone {
            // Exact-time task — show date + timezone
            // TODO: When the v1 API date+timezone exact-time format is clarified,
            // parse and display the actual local time here.
            format!("{} {}", self.date, tz_str)
        } else {
            self.date.clone()
        }
    }
}

/// Formats a [`DueDate`] using the given [`DateTime`], by coloring the output based on if it's
/// too late or too soon.
pub struct DueDateFormatter<'a>(pub &'a DueDate, pub &'a chrono::DateTime<Utc>);

impl Display for DueDateFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_recurring {
            write!(
                f,
                "{}",
                "[REPEAT] ".if_supports_color(Stream::Stdout, |_| "🔁 ")
            )?;
        }
        if self.0.timezone.is_some() {
            // Exact-time task — for now just show the date string
            // TODO: parse date+timezone into a proper datetime for comparison
            let display = self.0.display_date();
            if let Some(naive) = self.0.naive_date() {
                if naive >= self.1.date_naive() {
                    write!(
                        f,
                        "{}",
                        display.if_supports_color(Stream::Stdout, |text| text.bright_green())
                    )
                } else {
                    write!(
                        f,
                        "{}",
                        display.if_supports_color(Stream::Stdout, |text| text.bright_red())
                    )
                }
            } else {
                write!(f, "{display}")
            }
        } else if let Some(naive) = self.0.naive_date() {
            if naive >= self.1.date_naive() {
                write!(
                    f,
                    "{}",
                    self.0
                        .string
                        .if_supports_color(Stream::Stdout, |text| text.bright_green())
                )
            } else {
                write!(
                    f,
                    "{}",
                    self.0
                        .string
                        .if_supports_color(Stream::Stdout, |text| text.bright_red())
                )
            }
        } else {
            write!(f, "{}", self.0.string)
        }
    }
}

/// Human representation of the due date.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskDue {
    /// Human readable representation of the date.
    #[serde(rename = "due_string")]
    String(String),
    /// Loose target date with no exact time.
    #[serde(rename = "due_date")]
    Date(String),
    /// Exact DateTime in UTC for the due date.
    #[serde(rename = "due_datetime")]
    DateTime(chrono::DateTime<Utc>),
}
/// Command used with [`super::Gateway::create`] to create a new Task.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateTask {
    /// Sets the [`Task::content`] on the new [`Task`].
    pub content: String,
    /// Sets the [`Task::description`] on the new [`Task`].
    pub description: Option<String>,
    /// Sets the [`Task::project_id`] on the new [`Task`].
    pub project_id: Option<ProjectID>,
    /// Sets the [`Task::section_id`] on the new [`Task`].
    pub section_id: Option<SectionID>,
    /// Sets the [`Task::parent_id`] on the new [`Task`].
    pub parent_id: Option<TaskID>,
    /// Sets the [`Task::child_order`] on the new [`Task`].
    pub order: Option<isize>,
    /// Sets the [`Task::labels`] on the new [`Task`].
    pub labels: Vec<String>,
    /// Sets the [`Task::priority`] on the new [`Task`].
    pub priority: Option<Priority>,
    /// Sets the [`Task::due`] on the new [`Task`].
    #[serde(flatten)]
    pub due: Option<TaskDue>,
    /// If due is [TaskDue::String], this two-letter code optionally specifies the language if it's not english.
    pub due_lang: Option<String>,
    /// Sets the assignee on the new [`Task`].
    pub assignee: Option<UserID>,
}

/// Command used with [`super::Gateway::update`] to update a [`Task`].
///
/// Each field is optional, so if something exists, that part of the [`Task`] will get overwritten.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct UpdateTask {
    /// Overwrites [`Task::content`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Overwrites [`Task::description`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Overwrites [`Task::labels`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    /// Overwrites [`Task::priority`] if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    /// Overwrites [`Task::due`] if set.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub due: Option<TaskDue>,
    /// If due is [TaskDue::String], this two-letter code optionally specifies the language if it's not english.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_lang: Option<String>,
    /// Overwrites the assignee if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<UserID>,
}

#[cfg(test)]
impl Task {
    /// This is initializer is used for tests, as in general the tool relies on the API and not
    /// local state.
    pub fn new(id: &str, content: &str) -> Task {
        Task {
            id: id.to_string(),
            user_id: None,
            project_id: "".to_string(),
            section_id: None,
            parent_id: None,
            added_by_uid: None,
            assigned_by_uid: None,
            responsible_uid: None,
            content: content.to_string(),
            description: String::new(),
            labels: Vec::new(),
            checked: false,
            is_deleted: false,
            added_at: None,
            completed_at: None,
            completed_by_uid: None,
            updated_at: None,
            child_order: 0,
            priority: Priority::default(),
            due: None,
            note_count: 0,
            day_order: -1,
            is_collapsed: false,
            deadline: None,
            duration: None,
        }
    }
}
