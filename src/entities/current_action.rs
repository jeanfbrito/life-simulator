/// CurrentAction component - tracks what action an entity is currently performing
///
/// This is used for debugging and visualization in the web viewer
use bevy::prelude::*;

/// Component that stores the name of the current action being executed
#[derive(Component, Debug, Clone)]
pub struct CurrentAction {
    pub action_name: String,
}

impl CurrentAction {
    pub fn new(action_name: impl Into<String>) -> Self {
        Self {
            action_name: action_name.into(),
        }
    }

    pub fn none() -> Self {
        Self {
            action_name: "Idle".to_string(),
        }
    }
}

impl Default for CurrentAction {
    fn default() -> Self {
        Self::none()
    }
}
