use std::any::Any;
use std::time::SystemTime;

use crate::domain::shared::DomainEvent;

/// TaskCreated — a new task was added to the list.
///
/// DDD role: Domain Event (a past-tense fact the business cares about).
/// Business meaning: someone has something new to do.
/// Published: by the Unit of Work, only after the task was saved successfully.
/// Reacted to by: see `src/application/policies.rs` (the policy registry).
/// Pattern: Domain Events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCreated {
    pub task_id: String,
    pub description: String,
    pub occurred_on: SystemTime,
}

impl TaskCreated {
    pub const FQDN: &'static str = "TaskApp.TaskManagement.TaskCreated";
}

impl DomainEvent for TaskCreated {
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
