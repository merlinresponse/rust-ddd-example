use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use crate::infrastructure::persistence::in_memory::repositories::mappers::TaskRecord;
use crate::infrastructure::shared::lock;

/// InMemoryStore — the "database": a table of task rows plus a transaction lock.
///
/// Why modelled this way: rows are stored as `TaskRecord` (plain strings, like a DB row), not as
/// `Task` aggregates, so the mapper is exercised exactly as it would be with a real database.
/// Hexagonal role: part of the in-memory DRIVEN adapters (their "database"); not a port.
#[derive(Debug, Default)]
pub struct InMemoryStore {
    transaction: Mutex<()>,
    rows: Mutex<HashMap<String, TaskRecord>>,
}

pub type Rows = HashMap<String, TaskRecord>;

impl InMemoryStore {
    /// "BEGIN": only one transaction at a time. Dropping the guard is "COMMIT".
    pub fn begin(&self) -> MutexGuard<'_, ()> {
        lock(&self.transaction)
    }

    pub fn rows(&self) -> MutexGuard<'_, Rows> {
        lock(&self.rows)
    }

    pub fn snapshot(&self) -> Rows {
        self.rows().clone()
    }

    /// "ROLLBACK": put the table back the way it was.
    pub fn restore(&self, snapshot: Rows) {
        *self.rows() = snapshot;
    }
}
