//! Composition root: the ONE place that knows every concrete type and wires them together.
//! Everything else depends only on traits (ports).

use std::sync::Arc;

use crate::application::contracts::ports::{Logger, UnitOfWork};
use crate::application::policies::policy_registry;
use crate::application::tasks::commands::{
    AddChecklistItem, ChangeTaskDescription, CheckChecklistItem, CompleteTask, CreateTask,
    StartTask,
};
use crate::application::tasks::queries::{FindTaskById, ListTasks};
use crate::infrastructure::events::domain_events::PolicyDomainEventListener;
use crate::infrastructure::persistence::in_memory::queries::{
    InMemoryFindTaskByIdQuery, InMemoryListTasksQuery,
};
use crate::infrastructure::persistence::in_memory::shared::{InMemoryStore, InMemoryUnitOfWork};
use crate::infrastructure::shared::ConsoleLogger;
use crate::infrastructure::web_api::shared::AppState;

/// Everything `main` (or a test) needs: the use cases for HTTP and the event listener, so
/// tests can wait for background policies with `listener.when_idle()`.
pub struct Container {
    pub state: AppState,
    pub listener: Arc<PolicyDomainEventListener>,
}

/// Builds the whole application, from the bottom (logger, storage) up (use cases).
///
/// To switch to Postgres you would change only this function: build a Postgres Unit of Work
/// and query adapters instead of the in-memory ones. Connect lazily, on the first request that
/// needs data, and run migrations as a separate deliberate step — not here on every start.
pub fn bootstrap() -> Container {
    bootstrap_with(Arc::new(ConsoleLogger))
}

/// Same as [`bootstrap`], with a chosen logger (tests use a recording one).
pub fn bootstrap_with(logger: Arc<dyn Logger>) -> Container {
    // Events: registry (which policy reacts to what) → listener (runs them after commit).
    let listener = Arc::new(PolicyDomainEventListener::new(
        policy_registry(Arc::clone(&logger)),
        logger,
    ));

    // Persistence: the Unit of Work publishes committed events to the listener.
    let store = Arc::new(InMemoryStore::default());
    let unit_of_work: Arc<dyn UnitOfWork> = Arc::new(InMemoryUnitOfWork::new(
        Arc::clone(&store),
        listener.clone(),
    ));

    let state = AppState {
        create_task: Arc::new(CreateTask::new(Arc::clone(&unit_of_work))),
        start_task: Arc::new(StartTask::new(Arc::clone(&unit_of_work))),
        complete_task: Arc::new(CompleteTask::new(Arc::clone(&unit_of_work))),
        change_task_description: Arc::new(ChangeTaskDescription::new(Arc::clone(&unit_of_work))),
        add_checklist_item: Arc::new(AddChecklistItem::new(Arc::clone(&unit_of_work))),
        check_checklist_item: Arc::new(CheckChecklistItem::new(unit_of_work)),
        find_task_by_id: Arc::new(FindTaskById::new(Arc::new(InMemoryFindTaskByIdQuery::new(
            Arc::clone(&store),
        )))),
        list_tasks: Arc::new(ListTasks::new(Arc::new(InMemoryListTasksQuery::new(store)))),
    };
    Container { state, listener }
}
