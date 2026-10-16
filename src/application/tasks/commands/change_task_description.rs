use std::sync::Arc;

use crate::application::contracts::ports::{UnitOfWork, UseCase};
use crate::domain::shared::{DomainError, Result};
use crate::domain::task_management::tasks::value_objects::TaskDescription;
use crate::domain::task_management::tasks::TaskId;

/// ChangeTaskDescriptionInput — the data of the command "reword this task".
///
/// DDD role: Command input (the data of a request to CHANGE the system). It is plain data
///   from the outside world and not yet validated, so the command can still be refused. Compare a
///   domain event, which is a fact in the past tense and can no longer be refused.
/// Handled by: `ChangeTaskDescription` (the use case below), which turns it into domain calls.
/// Pattern: Command (CQRS-lite: commands change state, queries only read).
#[derive(Debug, Clone)]
pub struct ChangeTaskDescriptionInput {
    pub id: String,
    pub description: String,
}

/// ChangeTaskDescription — reword an existing task.
///
/// DDD role: Use Case = command handler (it receives a `ChangeTaskDescriptionInput` and changes state
///   through exactly one aggregate, inside one Unit of Work).
/// Business rules relied on: the new description must be valid (`TaskDescription`); a `Done`
///   task cannot be reworded (`Task::change_description`).
/// @throws `InvalidTaskId` | `EmptyTaskDescription` | `TaskDescriptionTooLong` | `TaskNotFound` |
///   `CompletedTaskCannotBeChanged` | `Infrastructure`.
pub struct ChangeTaskDescription {
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl ChangeTaskDescription {
    pub fn new(unit_of_work: Arc<dyn UnitOfWork>) -> Self {
        Self { unit_of_work }
    }
}

impl UseCase<ChangeTaskDescriptionInput> for ChangeTaskDescription {
    fn execute(&self, input: ChangeTaskDescriptionInput) -> Result<()> {
        let id = TaskId::parse(&input.id)?;
        let description = TaskDescription::create(&input.description)?;
        self.unit_of_work.run(&mut |uow| {
            let Some(mut task) = uow.tasks().by_id(&id)? else {
                return Err(DomainError::TaskNotFound(input.id.clone()).into());
            };
            task.change_description(description.clone())?;
            uow.save(task)
        })
    }
}
