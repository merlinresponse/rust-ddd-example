use crate::domain::shared::{DomainEvent, Result};

/// Policy — "WHEN this event happened, THEN do that".
///
/// DDD role: Policy (a reaction to a domain event that lives outside the aggregate that raised it).
/// Business meaning: a business reaction such as "when a task is completed, congratulate the
///   user" or "when a task is reworded, write an audit line".
/// Rules:
/// - named `When<Event><Reaction>`;
/// - runs AFTER the change that raised the event was committed, in the background;
/// - its failure never undoes that change (the task stays completed even if logging fails);
/// - it may run twice for the same event in a real message system, so make it safe to repeat
///   (idempotent).
///
/// Why modelled this way: `Task::complete` stays about tasks. New reactions are added as new
///   policies plus one line in the registry; no existing use case or aggregate changes.
/// Pattern: Policy / event handler (Domain Events, after Evans; "Event Storming" vocabulary).
pub trait Policy: Send + Sync {
    /// Used in logs, e.g. "policy WhenTaskCompletedLogCongratulations failed".
    fn name(&self) -> &'static str;
    fn handle(&self, event: &dyn DomainEvent) -> Result<()>;
}
