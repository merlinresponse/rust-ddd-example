use std::sync::Arc;

use crate::application::contracts::ports::queries::{FindTaskByIdQuery, TaskView};
use crate::domain::shared::Result;
use crate::domain::task_management::tasks::TaskId;
use crate::infrastructure::persistence::in_memory::repositories::mappers::to_view;
use crate::infrastructure::persistence::in_memory::shared::InMemoryStore;

/// InMemoryFindTaskByIdQuery — adapter for the `FindTaskByIdQuery` port; reads rows directly.
/// Hexagonal role: DRIVEN adapter (secondary adapter): implements the driven port `FindTaskByIdQuery`.
///   The application calls it; swap it (e.g. for Postgres) without touching domain or application.
pub struct InMemoryFindTaskByIdQuery {
    store: Arc<InMemoryStore>,
}

impl InMemoryFindTaskByIdQuery {
    pub fn new(store: Arc<InMemoryStore>) -> Self {
        Self { store }
    }
}

impl FindTaskByIdQuery for InMemoryFindTaskByIdQuery {
    fn fetch(&self, id: &TaskId) -> Result<Option<TaskView>> {
        Ok(self.store.rows().get(&id.to_string()).map(to_view))
    }
}
