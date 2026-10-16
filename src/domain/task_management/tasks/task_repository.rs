use super::{Task, TaskId};
use crate::domain::shared::Result;

/// TaskRepository — a collection-like store of Task aggregates.
///
/// DDD role: Repository port. A *repository* gives the illusion of an in-memory collection of
///   aggregates; a *port* is an interface the domain owns and infrastructure implements.
/// Business meaning: "find the task with this ID" and "keep this task".
/// Rules: one repository per aggregate root. A repository never commits a transaction and never
///   publishes events — the Unit of Work does both.
/// Why modelled this way: the domain says *what* it needs; whether it is Postgres or a HashMap
///   is decided in the infrastructure layer.
/// Pattern: Repository (Eric Evans, Domain-Driven Design, Ch. 6).
pub trait TaskRepository {
    fn by_id(&self, id: &TaskId) -> Result<Option<Task>>;
    fn save(&self, task: &Task) -> Result<()>;
}
