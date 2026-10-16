use std::sync::Arc;

use crate::application::contracts::ports::{UnitOfWork, UseCase};
use crate::domain::shared::{DomainError, Result};
use crate::domain::task_management::tasks::TaskId;

/// CompleteTaskInput — the data of the command "complete this task".
///
/// DDD role: Command input (the data of a request to CHANGE the system). It is plain data
///   from the outside world and not yet validated, so the command can still be refused. Compare a
///   domain event, which is a fact in the past tense and can no longer be refused.
/// Handled by: `CompleteTask` (the use case below), which turns it into domain calls.
/// Pattern: Command (CQRS-lite: commands change state, queries only read).
#[derive(Debug, Clone)]
pub struct CompleteTaskInput {
    pub id: String,
}

/// CompleteTask — mark a task as finished.
///
/// DDD role: Use Case = command handler (it receives a `CompleteTaskInput` and changes state
///   through exactly one aggregate, inside one Unit of Work).
/// Business rules relied on: `Todo` and `InProgress` tasks can be completed; `Done` is final
///   (`TaskStatus` table); every checklist item must be checked first (`Task::complete`).
/// @throws `InvalidTaskId` | `TaskNotFound` | `InvalidStatusTransition` |
///   `TaskHasOpenChecklistItems` | `Infrastructure`.
pub struct CompleteTask {
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl CompleteTask {
    pub fn new(unit_of_work: Arc<dyn UnitOfWork>) -> Self {
        Self { unit_of_work }
    }
}

impl UseCase<CompleteTaskInput> for CompleteTask {
    fn execute(&self, input: CompleteTaskInput) -> Result<()> {
        // FLOW 3 — WHAT: turn the raw string into a `TaskId`. A malformed id stops here (→ 400).
        // WHY: validate at the edge, so nothing deeper ever handles a string that "might" be an id.
        let id = TaskId::parse(&input.id)?;
        // FLOW 4 — WHAT: everything inside this closure is one all-or-nothing transaction.
        // WHY a closure: the Unit of Work decides when to begin, commit or roll back around it,
        // so this use case cannot forget to do so.
        self.unit_of_work.run(&mut |uow| {
            // FLOW 5 — WHAT: load the aggregate through the repository (a port: we don't know
            // or care whether it is a HashMap or Postgres).
            let Some(mut task) = uow.tasks().by_id(&id)? else {
                return Err(DomainError::TaskNotFound(input.id.clone()).into());
            };
            // FLOW 6 — WHAT: ask the aggregate to complete itself.
            // WHY not `if status == Done { … }` here: then the rule would be copied into every
            // use case that completes tasks. In the aggregate it exists exactly once.
            task.complete()?;
            // FLOW 7 — WHAT: save through the Unit of Work. It stores the task AND keeps it, so
            // after commit it can pull and publish the recorded events (FLOW 8).
            uow.save(task)
        })
    }
}
