use std::sync::Arc;

use crate::application::contracts::ports::{UnitOfWork, UseCase};
use crate::domain::shared::{DomainError, Error, Result};
use crate::domain::task_management::tasks::value_objects::ChecklistItemTitle;
use crate::domain::task_management::tasks::{ChecklistItemId, TaskId};

/// AddChecklistItemInput — the data of the command "add this step to the task's checklist".
///
/// DDD role: Command input (the data of a request to CHANGE the system). It is plain data
///   from the outside world and not yet validated, so the command can still be refused. Compare a
///   domain event, which is a fact in the past tense and can no longer be refused.
/// Handled by: `AddChecklistItem` (the use case below), which turns it into domain calls.
/// Pattern: Command (CQRS-lite: commands change state, queries only read).
#[derive(Debug, Clone)]
pub struct AddChecklistItemInput {
    pub task_id: String,
    pub title: String,
}

/// AddChecklistItem — add a step to a task's checklist.
///
/// DDD role: Use Case = command handler (it receives a `AddChecklistItemInput` and changes state
///   through exactly one aggregate, inside one Unit of Work).
/// Business rules relied on: the title must be valid (`ChecklistItemTitle`); a `Done` task gets
///   no new items (`Task::add_checklist_item`).
/// Note: we load and save the TASK, never the item on its own. The item is an entity *inside*
///   the Task aggregate, so it has no repository; it is reached only through its root.
/// @throws `InvalidTaskId` | `EmptyChecklistItemTitle` | `ChecklistItemTitleTooLong` |
///   `TaskNotFound` | `CompletedTaskCannotBeChanged` | `Infrastructure`.
/// Returns: the ID of the new checklist item.
pub struct AddChecklistItem {
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl AddChecklistItem {
    pub fn new(unit_of_work: Arc<dyn UnitOfWork>) -> Self {
        Self { unit_of_work }
    }
}

impl UseCase<AddChecklistItemInput, ChecklistItemId> for AddChecklistItem {
    fn execute(&self, input: AddChecklistItemInput) -> Result<ChecklistItemId> {
        let task_id = TaskId::parse(&input.task_id)?;
        let title = ChecklistItemTitle::create(&input.title)?;
        // The closure cannot return a value, so it writes the new ID into this variable.
        let mut new_item_id = None;
        self.unit_of_work.run(&mut |uow| {
            let Some(mut task) = uow.tasks().by_id(&task_id)? else {
                return Err(DomainError::TaskNotFound(input.task_id.clone()).into());
            };
            new_item_id = Some(task.add_checklist_item(title.clone())?);
            uow.save(task)
        })?;
        new_item_id.ok_or_else(|| Error::Infrastructure("unit of work did not run".into()))
    }
}
