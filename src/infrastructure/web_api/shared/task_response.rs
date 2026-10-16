use serde::Serialize;

use crate::application::contracts::ports::queries::{ChecklistItemView, TaskView};

/// JSON shape of a task in HTTP responses (a DTO: Data Transfer Object).
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub description: String,
    pub status: String,
    pub checklist: Vec<ChecklistItemResponse>,
}

#[derive(Debug, Serialize)]
pub struct ChecklistItemResponse {
    pub id: String,
    pub title: String,
    pub checked: bool,
}

impl From<ChecklistItemView> for ChecklistItemResponse {
    fn from(view: ChecklistItemView) -> Self {
        Self {
            id: view.id,
            title: view.title,
            checked: view.checked,
        }
    }
}

impl From<TaskView> for TaskResponse {
    fn from(view: TaskView) -> Self {
        Self {
            id: view.id,
            description: view.description,
            status: view.status,
            checklist: view
                .checklist
                .into_iter()
                .map(ChecklistItemResponse::from)
                .collect(),
        }
    }
}
