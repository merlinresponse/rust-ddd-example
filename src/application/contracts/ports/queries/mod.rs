//! Read side (CQRS-lite): queries return plain data and never go through aggregates.

mod find_task_by_id_query;
mod list_tasks_query;
mod task_view;

pub use find_task_by_id_query::FindTaskByIdQuery;
pub use list_tasks_query::ListTasksQuery;
pub use task_view::{ChecklistItemView, TaskView};
