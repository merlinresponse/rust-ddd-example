use super::TaskView;
use crate::domain::shared::Result;
use crate::domain::task_management::tasks::TaskId;

/// FindTaskByIdQuery — port: "give me the data of this one task".
///
/// DDD role: Query port (read side). No side effects, no events, no Unit of Work.
/// Pattern: CQRS-lite (beyond Evans).
pub trait FindTaskByIdQuery: Send + Sync {
    fn fetch(&self, id: &TaskId) -> Result<Option<TaskView>>;
}
