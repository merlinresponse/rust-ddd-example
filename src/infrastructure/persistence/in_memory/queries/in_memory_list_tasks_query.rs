use std::sync::Arc;

use crate::application::contracts::ports::queries::{ListTasksQuery, TaskView};
use crate::domain::shared::Result;
use crate::infrastructure::persistence::in_memory::repositories::mappers::to_view;
use crate::infrastructure::persistence::in_memory::shared::InMemoryStore;

/// InMemoryListTasksQuery — adapter for the `ListTasksQuery` port; reads rows directly.
/// Hexagonal role: DRIVEN adapter (secondary adapter): implements the driven port `ListTasksQuery`.
///   The application calls it; swap it (e.g. for Postgres) without touching domain or application.
pub struct InMemoryListTasksQuery {
    store: Arc<InMemoryStore>,
}

impl InMemoryListTasksQuery {
    pub fn new(store: Arc<InMemoryStore>) -> Self {
        Self { store }
    }
}

impl ListTasksQuery for InMemoryListTasksQuery {
    fn fetch(&self) -> Result<Vec<TaskView>> {
        let mut views: Vec<TaskView> = self.store.rows().values().map(to_view).collect();
        views.sort_by(|a, b| a.id.cmp(&b.id)); // HashMap order is random; make output stable
        Ok(views)
    }
}
