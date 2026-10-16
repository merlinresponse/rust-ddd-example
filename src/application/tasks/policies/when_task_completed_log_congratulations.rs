use std::sync::Arc;

use crate::application::contracts::ports::{Logger, Policy};
use crate::domain::shared::{DomainEvent, Error, Result};
use crate::domain::task_management::tasks::events::TaskCompleted;

/// WhenTaskCompletedLogCongratulations — cheer the user on when they finish something.
///
/// DDD role: Policy (a reaction to a domain event, outside the aggregate that raised it).
/// When → Then: WHEN `TaskCompleted` was committed → THEN log "Well done! Task … is completed."
/// On failure: the listener logs the error; the task stays completed (nothing is rolled back).
/// Idempotent: logging the same line twice is harmless.
/// Why modelled this way: congratulating is not part of "completing a task", so it does not
///   belong in `Task::complete`. Swap the logger for an e-mail or push port later and nothing
///   else changes.
/// Pattern: Policy (Domain Events).
pub struct WhenTaskCompletedLogCongratulations {
    logger: Arc<dyn Logger>,
}

impl WhenTaskCompletedLogCongratulations {
    pub const NAME: &'static str = "WhenTaskCompletedLogCongratulations";

    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self { logger }
    }
}

impl Policy for WhenTaskCompletedLogCongratulations {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn handle(&self, event: &dyn DomainEvent) -> Result<()> {
        // POLICY FLOW P4 — WHAT: get the concrete event back from the trait object.
        // WHY an error and not a silent `return`: receiving the wrong event means the registry
        // is wired wrongly, and we want to see that in the logs.
        let Some(completed) = event.downcast_ref::<TaskCompleted>() else {
            return Err(Error::Infrastructure(format!(
                "{} received {} instead of {}",
                Self::NAME,
                event.fqdn(),
                TaskCompleted::FQDN
            )));
        };
        // POLICY FLOW P5 — WHAT: the reaction itself: one log line.
        self.logger.info(&format!(
            "Well done! Task {} is completed.",
            completed.task_id
        ));
        Ok(())
    }
}
