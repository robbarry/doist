use crate::api::tree::Treeable;
use owo_colors::{OwoColorize, Stream};
use serde::{Deserialize, Serialize};

/// ProjectID is the unique ID of a [`Project`]
pub type ProjectID = String;
/// ProjectSyncID is an identifier to mark between copies of shared projects.
pub type ProjectSyncID = String;

/// Project as described by the Todoist API v1.
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/api/v1/#projects).
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Project {
    /// ID of the Project.
    pub id: ProjectID,
    /// The direct parent of the project if it exists.
    #[serde(default)]
    pub parent_id: Option<ProjectID>,
    /// The name of the Project. Displayed in the project list in the UI.
    pub name: String,
    /// Color as used by the Todoist UI.
    #[serde(default)]
    pub color: String,
    /// Whether the project is shared with someone else.
    #[serde(default)]
    pub is_shared: bool,
    /// Project order under the same parent.
    #[serde(default)]
    pub child_order: isize,
    /// This marks the project as the initial Inbox project if it exists.
    #[serde(default)]
    pub inbox_project: bool,
    /// Toggle to mark this project as a favorite.
    #[serde(default)]
    pub is_favorite: bool,
    /// View style to show in todoist clients.
    #[serde(default)]
    pub view_style: ViewStyle,
    /// Whether tasks can be assigned in this project.
    #[serde(default)]
    pub can_assign_tasks: bool,
    /// The user who created this project.
    #[serde(default)]
    pub creator_uid: Option<String>,
    /// When the project was created.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Whether the project is archived.
    #[serde(default)]
    pub is_archived: bool,
    /// Whether the project is deleted.
    #[serde(default)]
    pub is_deleted: bool,
    /// Whether the project is frozen.
    #[serde(default)]
    pub is_frozen: bool,
    /// Default ordering.
    #[serde(default)]
    pub default_order: isize,
    /// Project description.
    #[serde(default)]
    pub description: String,
    /// Whether the project has public access.
    #[serde(default)]
    pub public_access: bool,
    /// Public key for the project.
    #[serde(default)]
    pub public_key: Option<String>,
    /// Access configuration (opaque for now).
    #[serde(default)]
    pub access: Option<serde_json::Value>,
    /// User's role in this project.
    #[serde(default)]
    pub role: Option<String>,
    /// Whether the project is collapsed in the UI.
    #[serde(default)]
    pub is_collapsed: bool,
    /// When the project was last updated.
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl Ord for Project {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.child_order.cmp(&other.child_order) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Project {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// ViewStyle for viewing of the project in different clients.
///
/// Taken from the [Developer Documentation](https://developer.todoist.com/api/v1/#projects).
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ViewStyle {
    /// Project as list view (default).
    #[default]
    List,
    /// Project as board view.
    Board,
    /// Project as calendar view.
    Calendar,
}

impl Treeable for Project {
    type ID = ProjectID;

    fn id(&self) -> ProjectID {
        self.id.clone()
    }

    fn parent_id(&self) -> Option<ProjectID> {
        self.parent_id.clone()
    }

    fn reset_parent(&mut self) {
        self.parent_id = None;
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.id
                .if_supports_color(Stream::Stdout, |text| text.bright_yellow()),
            self.name
        )
    }
}

/// Command used with [`super::Gateway::create_project`] to create a new [`Project`].
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateProject {
    /// Name of the project to create.
    pub name: String,
    /// Makes the newly created project a child of this parent project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ProjectID>,
    /// Color of the project icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Mark as favorite or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorite: Option<bool>,
    /// Sets the view style of the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_style: Option<ViewStyle>,
}

#[cfg(test)]
impl Project {
    /// This is initializer is used for tests, as in general the tool relies on the API and not
    /// local state.
    pub fn new(id: &str, name: &str) -> Project {
        Project {
            id: id.to_string(),
            name: name.to_string(),
            parent_id: None,
            color: "".to_string(),
            is_shared: false,
            child_order: 0,
            inbox_project: false,
            is_favorite: false,
            view_style: Default::default(),
            can_assign_tasks: false,
            creator_uid: None,
            created_at: None,
            is_archived: false,
            is_deleted: false,
            is_frozen: false,
            default_order: 0,
            description: String::new(),
            public_access: false,
            public_key: None,
            access: None,
            role: None,
            is_collapsed: false,
            updated_at: None,
        }
    }
}
