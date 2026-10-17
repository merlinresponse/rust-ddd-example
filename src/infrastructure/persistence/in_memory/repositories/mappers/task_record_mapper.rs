use crate::application::contracts::ports::queries::{ChecklistItemView, TaskView};
use crate::domain::shared::{Entity, Error, Result};
use crate::domain::task_management::tasks::value_objects::{
    ChecklistItemTitle, TaskDescription, TaskStatus,
};
use crate::domain::task_management::tasks::{ChecklistItem, ChecklistItemId, Task, TaskId};

/// TaskRecord — one row of the "tasks" table: plain strings, exactly what a database stores.
/// The checklist is stored WITH its task (in SQL: a `checklist_items` table with a `task_id`
/// column, always read and written together with the task row).
/// Hexagonal role: part of the DRIVEN persistence adapter (translates rows ⇄ aggregates); not a port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRecord {
    pub id: String,
    pub description: String,
    pub status: String,
    pub checklist: Vec<ChecklistItemRecord>,
}

/// ChecklistItemRecord — one stored checklist item. Its `id` is stored because it is an
/// entity: after loading, it must still be *the same item* the user ticked before.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItemRecord {
    pub id: String,
    pub title: String,
    pub checked: bool,
}

/// Aggregate → row. The whole aggregate (root + inner entities) is saved as one unit.
pub fn to_record(task: &Task) -> TaskRecord {
    TaskRecord {
        id: task.id().to_string(),
        description: task.description().value().to_string(),
        status: task.status().as_str().to_string(),
        checklist: task
            .checklist()
            .iter()
            .map(|item| ChecklistItemRecord {
                id: item.id().to_string(),
                title: item.title().value().to_string(),
                checked: item.is_checked(),
            })
            .collect(),
    }
}

/// Row → aggregate, through `Task::reconstitute` (so no "created" event is raised again).
/// Stored data is re-validated; a corrupt row becomes an `Infrastructure` error instead of a
/// panic (the article's `expect` here would crash the server on one bad row).
pub fn reconstitute(record: &TaskRecord) -> Result<Task> {
    let corrupt = |e| Error::Infrastructure(format!("corrupt task row {}: {e}", record.id));
    let checklist = record
        .checklist
        .iter()
        .map(|item| {
            Ok(ChecklistItem::reconstitute(
                ChecklistItemId::parse(&item.id).map_err(corrupt)?,
                ChecklistItemTitle::create(&item.title).map_err(corrupt)?,
                item.checked,
            ))
        })
        .collect::<Result<Vec<_>>>()?; // stops at the first corrupt item
    Ok(Task::reconstitute(
        TaskId::parse(&record.id).map_err(corrupt)?,
        TaskDescription::create(&record.description).map_err(corrupt)?,
        TaskStatus::parse(&record.status).map_err(corrupt)?,
        checklist,
    ))
}

/// Row → read model, for queries. Skips the aggregate entirely.
pub fn to_view(record: &TaskRecord) -> TaskView {
    TaskView {
        id: record.id.clone(),
        description: record.description.clone(),
        status: record.status.clone(),
        checklist: record
            .checklist
            .iter()
            .map(|item| ChecklistItemView {
                id: item.id.clone(),
                title: item.title.clone(),
                checked: item.checked,
            })
            .collect(),
    }
}
