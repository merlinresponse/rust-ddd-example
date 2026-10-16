//! The Task aggregate: everything a novice needs to understand tasks lives in this folder.

mod checklist_item;
mod checklist_item_id;
pub mod events;
mod task;
mod task_id;
mod task_repository;
pub mod value_objects;

pub use checklist_item::ChecklistItem;
pub use checklist_item_id::ChecklistItemId;
pub use task::Task;
pub use task_id::TaskId;
pub use task_repository::TaskRepository;
