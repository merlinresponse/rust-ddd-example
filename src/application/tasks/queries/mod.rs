//! Query side (read): handlers that return plain data (`TaskView`) through a query port from
//! `application/contracts/ports/queries`. No Unit of Work, no aggregate, no events.
//!
//! Note the two `queries` folders: `contracts/ports/queries` holds the PORTS (the trait the
//! storage implements); this folder holds the USE CASES that call them.

mod find_task_by_id;
mod list_tasks;

pub use find_task_by_id::{FindTaskById, FindTaskByIdInput};
pub use list_tasks::ListTasks;
