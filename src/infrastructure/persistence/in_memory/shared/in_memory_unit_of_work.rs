use std::sync::{Arc, Mutex};

use super::InMemoryStore;
use crate::application::contracts::ports::UnitOfWork;
use crate::domain::shared::{AggregateRoot, DomainEventPublisher, Result};
use crate::domain::task_management::tasks::{Task, TaskRepository};
use crate::infrastructure::persistence::in_memory::repositories::InMemoryTaskRepository;
use crate::infrastructure::shared::lock;

/// InMemoryUnitOfWork — Unit of Work adapter for the in-memory store.
///
/// DDD role: Adapter (the concrete implementation of the `UnitOfWork` port).
/// Hexagonal role: DRIVEN adapter (secondary adapter): implements the driven port `UnitOfWork`.
///   The application calls it; swap it (e.g. for Postgres) without touching domain or application.
/// Guarantees (the same contract a Postgres-backed Unit of Work would keep):
/// 1. begin = take the transaction lock + snapshot; on error = restore the snapshot (rollback);
/// 2. events are pulled and published ONLY after a successful commit, after the lock is released;
/// 3. `save` stores through the repository and keeps the task to pull its events later.
pub struct InMemoryUnitOfWork {
    store: Arc<InMemoryStore>,
    tasks: InMemoryTaskRepository,
    publisher: Arc<dyn DomainEventPublisher>,
    tracked: Mutex<Vec<Task>>,
}

impl InMemoryUnitOfWork {
    pub fn new(store: Arc<InMemoryStore>, publisher: Arc<dyn DomainEventPublisher>) -> Self {
        Self {
            tasks: InMemoryTaskRepository::new(Arc::clone(&store)),
            store,
            publisher,
            tracked: Mutex::new(Vec::new()),
        }
    }
}

impl UnitOfWork for InMemoryUnitOfWork {
    fn run(&self, action: &mut dyn FnMut(&dyn UnitOfWork) -> Result<()>) -> Result<()> {
        let (outcome, tracked) = {
            // FLOW 4a — WHAT: BEGIN: take the lock and a snapshot to roll back to.
            // WHY: two requests changing the same task at once must not overwrite each other.
            let _transaction = self.store.begin(); // BEGIN
            let snapshot = self.store.snapshot();
            // FLOW 4b — WHAT: run the use case's closure; FLOW 5, 6 and 7 happen inside this call.
            let outcome = action(self);
            if outcome.is_err() {
                self.store.restore(snapshot); // ROLLBACK
            }
            // Taken while still holding the lock, so concurrent runs never mix their tasks.
            let tracked = std::mem::take(&mut *lock(&self.tracked));
            (outcome, tracked)
        }; // COMMIT: the lock is released here
           // FLOW 8 — WHAT: only now, after commit, pull the recorded events and publish them.
           // WHY: if we published first and the save then failed, listeners would react to
           // something that never happened. After a rollback, the events are simply dropped.
        if outcome.is_ok() {
            for mut task in tracked {
                for event in task.pull_domain_events() {
                    self.publisher.publish(event);
                }
            }
        }
        outcome // never swallow the error
    }

    fn tasks(&self) -> &dyn TaskRepository {
        &self.tasks
    }

    fn save(&self, task: Task) -> Result<()> {
        self.tasks.save(&task)?;
        lock(&self.tracked).push(task);
        Ok(())
    }
}
