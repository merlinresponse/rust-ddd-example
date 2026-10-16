use std::sync::Arc;

use crate::application::contracts::ports::queries::{FindTaskByIdQuery, TaskView};
use crate::application::contracts::ports::UseCase;
use crate::domain::shared::{DomainError, Result};
use crate::domain::task_management::tasks::TaskId;

/// Input of FindTaskById. Not a command: a query only asks, so it can never be "refused" by
/// a business rule (only answered with "not found").
#[derive(Debug, Clone)]
pub struct FindTaskByIdInput {
    pub id: String,
}

/// FindTaskById — show one task.
///
/// DDD role: Use Case (query: reads only, changes nothing, no Unit of Work).
/// @throws `InvalidTaskId` | `TaskNotFound` | `Infrastructure`.
pub struct FindTaskById {
    query: Arc<dyn FindTaskByIdQuery>,
}

impl FindTaskById {
    pub fn new(query: Arc<dyn FindTaskByIdQuery>) -> Self {
        Self { query }
    }
}

impl UseCase<FindTaskByIdInput, TaskView> for FindTaskById {
    fn execute(&self, input: FindTaskByIdInput) -> Result<TaskView> {
        let id = TaskId::parse(&input.id)?;
        self.query
            .fetch(&id)?
            .ok_or_else(|| DomainError::TaskNotFound(input.id).into())
    }
}
