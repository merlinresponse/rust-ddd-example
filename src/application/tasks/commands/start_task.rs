use std::sync::Arc;

use crate::application::contracts::ports::{UnitOfWork, UseCase};
use crate::domain::shared::{DomainError, Result};
use crate::domain::task_management::tasks::TaskId;

/// StartTaskInput — the data of the command "start this task".
///
/// DDD role: Command input (the data of a request to CHANGE the system). It is plain data
///   from the outside world and not yet validated, so the command can still be refused. Compare a
///   domain event, which is a fact in the past tense and can no longer be refused.
/// Handled by: `StartTask` (the use case below), which turns it into domain calls.
/// Pattern: Command (CQRS-lite: commands change state, queries only read).
#[derive(Debug, Clone)]
pub struct StartTaskInput {
    pub id: String,
}

/// StartTask — someone begins working on a task.
///
/// DDD role: Use Case = command handler (it receives a `StartTaskInput` and changes state
///   through exactly one aggregate, inside one Unit of Work).
/// Business rules relied on: only a `Todo` task can be started (`TaskStatus` table).
/// @throws `InvalidTaskId` | `TaskNotFound` | `InvalidStatusTransition` | `Infrastructure`.
pub struct StartTask {
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl StartTask {
    pub fn new(unit_of_work: Arc<dyn UnitOfWork>) -> Self {
        Self { unit_of_work }
    }
}

impl UseCase<StartTaskInput> for StartTask {
    fn execute(&self, input: StartTaskInput) -> Result<()> {
        let id = TaskId::parse(&input.id)?;
        self.unit_of_work.run(&mut |uow| {
            // 1. load
            let Some(mut task) = uow.tasks().by_id(&id)? else {
                return Err(DomainError::TaskNotFound(input.id.clone()).into());
            };
            // 2. ask the aggregate to do it (the rule lives THERE, not here)
            task.start()?;
            // 3. save through the Unit of Work (events are published after commit)
            uow.save(task)
        })
    }
}
