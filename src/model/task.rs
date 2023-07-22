use serde::Serialize;
use uuid::Uuid;
use strum_macros::{EnumString, Display};

#[derive(Debug, Serialize, EnumString, Display, Eq, PartialEq)]
pub enum TaskState {
    NotStarted,
    InProgress,
    Completed,
    Paused,
    Failed
}

#[derive(Serialize)]
pub struct Task {
    pub user_uuid: String,
    pub task_uuid: String,
    pub task_type: String,
    pub state: TaskState,
    pub source_file: String,
    pub result_file: Option<String>,
    pub title: String,
    pub description: String,
}

impl Task {
    pub fn new(
        user_uuid: String,
        task_type: String,
        source_file: String,
        title: String,
        description: String,
    ) -> Task {
        Task {
            user_uuid,
            task_uuid: Uuid::new_v4().to_string(),
            task_type,
            state: TaskState::Pending,
            source_file,
            result_file: None,
            title,
            description,
        }
    }

    pub fn get_global_id(&self) -> String {
        format!("{}-{}", self.user_uuid, self.task_uuid)
    }

    pub fn can_transition_to(&self, state: &TaskState) -> bool {
        self.state != *state
    }
}
