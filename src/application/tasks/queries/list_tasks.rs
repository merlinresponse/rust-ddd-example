use std::sync::Arc;

use crate::application::contracts::ports::queries::{ListTasksQuery, TaskView};
use crate::application::contracts::ports::UseCase;
use crate::domain::shared::Result;

/// ListTasks — show all tasks.
///
/// DDD role: Use Case (query: reads only, changes nothing, no Unit of Work).
/// @throws `Infrastructure`.
pub struct ListTasks {
    query: Arc<dyn ListTasksQuery>,
}

impl ListTasks {
    pub fn new(query: Arc<dyn ListTasksQuery>) -> Self {
        Self { query }
    }
}

impl UseCase<(), Vec<TaskView>> for ListTasks {
    fn execute(&self, _input: ()) -> Result<Vec<TaskView>> {
        self.query.fetch()
    }
}
