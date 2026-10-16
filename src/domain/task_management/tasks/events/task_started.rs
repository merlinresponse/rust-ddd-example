use std::any::Any;
use std::time::SystemTime;

use crate::domain::shared::DomainEvent;

/// TaskStarted — work on a task has begun.
///
/// DDD role: Domain Event (a past-tense fact the business cares about).
/// Business meaning: the task moved from todo to in progress.
/// Published: by the Unit of Work, only after the task was saved successfully.
/// Reacted to by: see `src/application/policies.rs` (the policy registry).
/// Pattern: Domain Events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStarted {
    pub task_id: String,
    pub occurred_on: SystemTime,
}

impl TaskStarted {
    pub const FQDN: &'static str = "TaskApp.TaskManagement.TaskStarted";
}

impl DomainEvent for TaskStarted {
    fn fqdn(&self) -> &'static str {
        Self::FQDN
    }

    fn occurred_on(&self) -> SystemTime {
        self.occurred_on
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
