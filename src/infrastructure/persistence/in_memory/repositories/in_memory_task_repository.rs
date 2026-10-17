use std::sync::Arc;

use super::mappers::{reconstitute, to_record};
use crate::domain::shared::Result;
use crate::domain::task_management::tasks::{Task, TaskId, TaskRepository};
use crate::infrastructure::persistence::in_memory::shared::InMemoryStore;

/// InMemoryTaskRepository — stores tasks in the in-memory table.
///
/// DDD role: Adapter for the `TaskRepository` port.
/// Hexagonal role: DRIVEN adapter (secondary adapter): implements the driven port `TaskRepository`.
///   The application calls it; swap it (e.g. for Postgres) without touching domain or application.
/// Rules: no commit, no event publishing — the Unit of Work does that.
pub struct InMemoryTaskRepository {
    store: Arc<InMemoryStore>,
}

impl InMemoryTaskRepository {
    pub fn new(store: Arc<InMemoryStore>) -> Self {
        Self { store }
    }
}

impl TaskRepository for InMemoryTaskRepository {
    fn by_id(&self, id: &TaskId) -> Result<Option<Task>> {
        // FLOW 5a — WHAT: find the row and turn it (plain strings) into a `Task` via the mapper.
        // WHY: the rest of the code only ever sees valid aggregates, never raw rows.
        // `transpose` flips Option<Result<Task>> into Result<Option<Task>>, i.e. the three
        // outcomes "found", "not found" and "row is broken".
        self.store
            .rows()
            .get(&id.to_string())
            .map(reconstitute)
            .transpose()
    }

    fn save(&self, task: &Task) -> Result<()> {
        // FLOW 7a — WHAT: `Task` → row via the mapper, then write it.
        // WHY no commit or events here: the Unit of Work owns the transaction, so several saves
        // (in a bigger app) succeed or fail together.
        let record = to_record(task);
        self.store.rows().insert(record.id.clone(), record);
        Ok(())
    }
}
