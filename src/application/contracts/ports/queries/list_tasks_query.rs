use super::TaskView;
use crate::domain::shared::Result;

/// ListTasksQuery — port: "give me all tasks".
///
/// DDD role: Query port (read side). No side effects, no events, no Unit of Work.
/// Pattern: CQRS-lite (beyond Evans).
pub trait ListTasksQuery: Send + Sync {
    fn fetch(&self) -> Result<Vec<TaskView>>;
}
