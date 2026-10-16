use std::any::Any;
use std::time::SystemTime;

use crate::domain::shared::DomainEvent;

/// ChecklistItemAdded — a step was added to a task's checklist.
///
/// DDD role: Domain Event (a past-tense fact the business cares about).
/// Business meaning: the task got a new, unchecked box. Raised by the Task (the root), not by the item.
/// Published: by the Unit of Work, only after the task was saved successfully.
/// Reacted to by: see `src/application/policies.rs` (the policy registry).
/// Pattern: Domain Events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItemAdded {
    pub task_id: String,
    pub item_id: String,
    pub title: String,
    pub occurred_on: SystemTime,
}

impl ChecklistItemAdded {
    pub const FQDN: &'static str = "TaskApp.TaskManagement.ChecklistItemAdded";
}

impl DomainEvent for ChecklistItemAdded {
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
