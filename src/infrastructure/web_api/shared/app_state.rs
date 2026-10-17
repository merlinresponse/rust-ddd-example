use std::sync::Arc;

use crate::application::tasks::commands::{
    AddChecklistItem, ChangeTaskDescription, CheckChecklistItem, CompleteTask, CreateTask,
    StartTask,
};
use crate::application::tasks::queries::{FindTaskById, ListTasks};

/// Hexagonal role: part of the HTTP DRIVING adapter: the driving ports (use cases) it may call.
///
/// The use cases every controller can reach. Built once in `main_modules`, cloned per request
/// (cloning only copies the `Arc` pointers).
#[derive(Clone)]
pub struct AppState {
    pub create_task: Arc<CreateTask>,
    pub start_task: Arc<StartTask>,
    pub complete_task: Arc<CompleteTask>,
    pub change_task_description: Arc<ChangeTaskDescription>,
    pub find_task_by_id: Arc<FindTaskById>,
    pub list_tasks: Arc<ListTasks>,
    pub add_checklist_item: Arc<AddChecklistItem>,
    pub check_checklist_item: Arc<CheckChecklistItem>,
}
