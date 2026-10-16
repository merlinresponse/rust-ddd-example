use crate::domain::shared::Result;
use crate::domain::task_management::tasks::{Task, TaskRepository};

/// UnitOfWork — runs one use case as one all-or-nothing transaction.
///
/// DDD role: Application port (an interface the application layer owns).
/// The implementation guarantees:
/// - `run(action)`: begin → action → commit → THEN publish the domain events of every saved
///   aggregate. If `action` fails: roll back, publish nothing, return the error unchanged.
/// - `save(task)`: store the task through its repository and remember it, so its events can be
///   published after commit. Always save through the Unit of Work, never directly.
///
/// Rules: change ONE aggregate per Unit of Work. Do not call `run` inside `run`.
/// Why modelled this way: transactions and event publishing are technical concerns; keeping
///   them out of the domain and out of every use case means no use case can forget them.
/// Pattern: Unit of Work (Martin Fowler, Patterns of Enterprise Application Architecture);
///   Evans Ch. 6: "leave transaction control to the client".
pub trait UnitOfWork: Send + Sync {
    /// `&mut dyn FnMut` (instead of a generic closure) keeps the trait usable as `Arc<dyn UnitOfWork>`.
    fn run(&self, action: &mut dyn FnMut(&dyn UnitOfWork) -> Result<()>) -> Result<()>;
    fn tasks(&self) -> &dyn TaskRepository;
    fn save(&self, task: Task) -> Result<()>;
}
