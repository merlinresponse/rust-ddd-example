use super::TaskView;
use crate::domain::shared::Result;
use crate::domain::task_management::tasks::TaskId;

/// FindTaskByIdQuery — port: "give me the data of this one task".
///
/// DDD role: Query port (read side). No side effects, no events, no Unit of Work.
/// Hexagonal role: DRIVEN port (secondary port): something the application NEEDS from the
///   outside world. Defined on the inside, implemented by a driven adapter outside:
///   `InMemoryFindTaskByIdQuery` (and a hand-written double in tests/common).
/// Pattern: CQRS-lite (beyond Evans).
pub trait FindTaskByIdQuery: Send + Sync {
    fn fetch(&self, id: &TaskId) -> Result<Option<TaskView>>;
}
