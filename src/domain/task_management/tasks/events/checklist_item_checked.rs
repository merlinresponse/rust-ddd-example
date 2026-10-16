use std::any::Any;
use std::time::SystemTime;

use crate::domain::shared::DomainEvent;

/// ChecklistItemChecked — a checklist step was ticked off.
///
/// DDD role: Domain Event (a past-tense fact the business cares about).
/// Business meaning: one step of the task is done. Raised by the Task (the root), not by the item.
/// Published: by the Unit of Work, only after the task was saved successfully.
/// Reacted to by: see `src/application/policies.rs` (the policy registry).
/// Pattern: Domain Events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItemChecked {
    pub task_id: String,
    pub item_id: String,
    pub occurred_on: SystemTime,
}

impl ChecklistItemChecked {
    pub const FQDN: &'static str = "TaskApp.TaskManagement.ChecklistItemChecked";
}

impl DomainEvent for ChecklistItemChecked {
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
