//! Command side (write): every file holds a `<UseCase>Input` (the command's data) and the
//! `<UseCase>` that handles it. Recipe: parse input → `run` { load → ONE domain method → save }.

mod add_checklist_item;
mod change_task_description;
mod check_checklist_item;
mod complete_task;
mod create_task;
mod start_task;

pub use add_checklist_item::{AddChecklistItem, AddChecklistItemInput};
pub use change_task_description::{ChangeTaskDescription, ChangeTaskDescriptionInput};
pub use check_checklist_item::{CheckChecklistItem, CheckChecklistItemInput};
pub use complete_task::{CompleteTask, CompleteTaskInput};
pub use create_task::{CreateTask, CreateTaskInput};
pub use start_task::{StartTask, StartTaskInput};
