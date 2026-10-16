use std::sync::Arc;

use crate::application::contracts::ports::{UnitOfWork, UseCase};
use crate::domain::shared::{Entity, Result};
use crate::domain::task_management::tasks::value_objects::TaskDescription;
use crate::domain::task_management::tasks::{Task, TaskId};

/// CreateTaskInput — the data of the command "add a new task".
///
/// DDD role: Command input (the data of a request to CHANGE the system). It is plain data
///   from the outside world and not yet validated, so the command can still be refused. Compare a
///   domain event, which is a fact in the past tense and can no longer be refused.
/// Handled by: `CreateTask` (the use case below), which turns it into domain calls.
/// Pattern: Command (CQRS-lite: commands change state, queries only read).
#[derive(Debug, Clone)]
pub struct CreateTaskInput {
    pub description: String,
}

/// CreateTask — add a new task to the list.
///
/// DDD role: Use Case = command handler (it receives a `CreateTaskInput` and changes state
///   through exactly one aggregate, inside one Unit of Work).
/// Business rules relied on: the description must be valid (`TaskDescription`); a new task
///   always starts as `Todo` and raises `TaskCreated` (`Task::create`).
/// @throws `EmptyTaskDescription` | `TaskDescriptionTooLong` | `Infrastructure`.
/// Returns: the ID of the new task.
pub struct CreateTask {
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl CreateTask {
    pub fn new(unit_of_work: Arc<dyn UnitOfWork>) -> Self {
        Self { unit_of_work }
    }
}

impl UseCase<CreateTaskInput, TaskId> for CreateTask {
    fn execute(&self, input: CreateTaskInput) -> Result<TaskId> {
        // Validate first: an invalid description never reaches the Unit of Work.
        let description = TaskDescription::create(&input.description)?;
        let task = Task::create(description);
        let id = *task.id();
        let mut task = Some(task);
        self.unit_of_work.run(&mut |uow| {
            // `take()` moves the task into the UoW; the closure type (`FnMut`) cannot move it directly.
            match task.take() {
                Some(task) => uow.save(task),
                None => Ok(()),
            }
        })?;
        Ok(id)
    }
}
