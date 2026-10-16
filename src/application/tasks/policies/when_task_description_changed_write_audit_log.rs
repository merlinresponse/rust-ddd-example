use std::sync::Arc;

use crate::application::contracts::ports::{Logger, Policy};
use crate::domain::shared::{DomainEvent, Error, Result};
use crate::domain::task_management::tasks::events::TaskDescriptionChanged;

/// WhenTaskDescriptionChangedWriteAuditLog — keep a trail of who-changed-what.
///
/// DDD role: Policy (a reaction to a domain event, outside the aggregate that raised it).
/// When → Then: WHEN `TaskDescriptionChanged` was committed → THEN write an audit line with the
///   old and the new description.
/// On failure: the listener logs the error; the new description stays saved.
/// Idempotent: a duplicate audit line is harmless (a real audit store would dedupe on event id).
/// Why modelled this way: the event already carries both versions, so the policy needs nothing
///   else — it never loads the task. This is why events carry the facts as plain values.
/// Pattern: Policy (Domain Events).
pub struct WhenTaskDescriptionChangedWriteAuditLog {
    logger: Arc<dyn Logger>,
}

impl WhenTaskDescriptionChangedWriteAuditLog {
    pub const NAME: &'static str = "WhenTaskDescriptionChangedWriteAuditLog";

    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self { logger }
    }
}

impl Policy for WhenTaskDescriptionChangedWriteAuditLog {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn handle(&self, event: &dyn DomainEvent) -> Result<()> {
        let Some(changed) = event.downcast_ref::<TaskDescriptionChanged>() else {
            return Err(Error::Infrastructure(format!(
                "{} received {} instead of {}",
                Self::NAME,
                event.fqdn(),
                TaskDescriptionChanged::FQDN
            )));
        };
        self.logger.info(&format!(
            "[audit] task {} reworded from \"{}\" to \"{}\"",
            changed.task_id, changed.old_description, changed.new_description
        ));
        Ok(())
    }
}
