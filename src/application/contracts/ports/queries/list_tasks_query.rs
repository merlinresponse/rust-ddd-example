use super::TaskView;
use crate::domain::shared::Result;

/// ListTasksQuery — port: "give me all tasks".
///
/// DDD role: Query port (read side). No side effects, no events, no Unit of Work.
/// Hexagonal role: DRIVEN port (secondary port): something the application NEEDS from the
///   outside world. Defined on the inside, implemented by a driven adapter outside:
///   `InMemoryListTasksQuery` (and a hand-written double in tests/common).
/// Pattern: CQRS-lite (beyond Evans).
pub trait ListTasksQuery: Send + Sync {
    fn fetch(&self) -> Result<Vec<TaskView>>;
}
