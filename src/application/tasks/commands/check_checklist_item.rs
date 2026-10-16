use std::sync::Arc;

use crate::application::contracts::ports::{UnitOfWork, UseCase};
use crate::domain::shared::{DomainError, Result};
use crate::domain::task_management::tasks::{ChecklistItemId, TaskId};

/// CheckChecklistItemInput — the data of the command "tick off this checklist item".
///
/// DDD role: Command input (the data of a request to CHANGE the system). It is plain data
///   from the outside world and not yet validated, so the command can still be refused. Compare a
///   domain event, which is a fact in the past tense and can no longer be refused.
/// Handled by: `CheckChecklistItem` (the use case below), which turns it into domain calls.
/// Pattern: Command (CQRS-lite: commands change state, queries only read).
#[derive(Debug, Clone)]
pub struct CheckChecklistItemInput {
    pub task_id: String,
    pub item_id: String,
}

/// CheckChecklistItem — tick off one step of a task.
///
/// DDD role: Use Case = command handler (it receives a `CheckChecklistItemInput` and changes state
///   through exactly one aggregate, inside one Unit of Work).
/// Business rules relied on: the item must belong to this task; checking twice changes nothing
///   (`Task::check_checklist_item`).
/// Note: the item is found by its IDENTITY (`ChecklistItemId`), never by its title — that is
///   what makes it an entity.
/// @throws `InvalidTaskId` | `InvalidChecklistItemId` | `TaskNotFound` | `ChecklistItemNotFound` |
///   `Infrastructure`.
pub struct CheckChecklistItem {
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl CheckChecklistItem {
    pub fn new(unit_of_work: Arc<dyn UnitOfWork>) -> Self {
        Self { unit_of_work }
    }
}

impl UseCase<CheckChecklistItemInput> for CheckChecklistItem {
    fn execute(&self, input: CheckChecklistItemInput) -> Result<()> {
        let task_id = TaskId::parse(&input.task_id)?;
        let item_id = ChecklistItemId::parse(&input.item_id)?;
        self.unit_of_work.run(&mut |uow| {
            let Some(mut task) = uow.tasks().by_id(&task_id)? else {
                return Err(DomainError::TaskNotFound(input.task_id.clone()).into());
            };
            // Ask the ROOT to change its inner entity; we never touch the item directly.
            task.check_checklist_item(&item_id)?;
            uow.save(task)
        })
    }
}
