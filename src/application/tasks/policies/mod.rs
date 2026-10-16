//! Task policies: one file per reaction, named `When<Event><Reaction>`.

mod when_task_completed_log_congratulations;
mod when_task_description_changed_write_audit_log;

pub use when_task_completed_log_congratulations::WhenTaskCompletedLogCongratulations;
pub use when_task_description_changed_write_audit_log::WhenTaskDescriptionChangedWriteAuditLog;
