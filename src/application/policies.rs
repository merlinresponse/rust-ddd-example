use std::collections::HashMap;
use std::sync::Arc;

use crate::application::contracts::ports::{Logger, Policy};
use crate::application::tasks::policies::{
    WhenTaskCompletedLogCongratulations, WhenTaskDescriptionChangedWriteAuditLog,
};
use crate::domain::task_management::tasks::events::{TaskCompleted, TaskDescriptionChanged};

/// Event FQDN → the policies that react to it.
pub type PolicyRegistry = HashMap<&'static str, Vec<Arc<dyn Policy>>>;

/// The policy registry: the ONE place that says which policy reacts to which event.
///
/// WHAT: a lookup table read by `PolicyDomainEventListener` for every published event.
/// WHY one table: you can see every reaction in the system at a glance, and adding one is a
///   new policy file plus one line here. Events without an entry (e.g. `TaskCreated`) are
///   simply logged and nobody reacts.
pub fn policy_registry(logger: Arc<dyn Logger>) -> PolicyRegistry {
    let mut registry = PolicyRegistry::new();
    registry
        .entry(TaskCompleted::FQDN)
        .or_default()
        .push(Arc::new(WhenTaskCompletedLogCongratulations::new(
            Arc::clone(&logger),
        )));
    registry
        .entry(TaskDescriptionChanged::FQDN)
        .or_default()
        .push(Arc::new(WhenTaskDescriptionChangedWriteAuditLog::new(
            logger,
        )));
    registry
}
