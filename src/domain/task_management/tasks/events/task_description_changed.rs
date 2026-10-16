use std::any::Any;
use std::time::SystemTime;

use crate::domain::shared::DomainEvent;

/// TaskDescriptionChanged — a task was reworded.
///
/// DDD role: Domain Event (a past-tense fact the business cares about).
/// Business meaning: the user changed what the task says; both versions are kept in the event.
/// Published: by the Unit of Work, only after the task was saved successfully.
/// Reacted to by: see `src/application/policies.rs` (the policy registry).
/// Pattern: Domain Events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDescriptionChanged {
    pub task_id: String,
    pub old_description: String,
    pub new_description: String,
    pub occurred_on: SystemTime,
}

impl TaskDescriptionChanged {
    pub const FQDN: &'static str = "TaskApp.TaskManagement.TaskDescriptionChanged";
}

impl DomainEvent for TaskDescriptionChanged {
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
