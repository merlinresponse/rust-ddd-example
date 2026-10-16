//! Hand-written test doubles, shared by the test files. See README.md in this folder.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use task_manager::application::contracts::ports::{Logger, UnitOfWork};
use task_manager::domain::shared::{
    AggregateRoot, DomainEvent, DomainEventPublisher, Entity, Error, Result,
};
use task_manager::domain::task_management::tasks::value_objects::{
    ChecklistItemTitle, TaskDescription, TaskStatus,
};
use task_manager::domain::task_management::tasks::{
    ChecklistItem, ChecklistItemId, Task, TaskId, TaskRepository,
};

/// What the mock saw being saved (a `Task` itself is not `Clone`, so we keep a snapshot).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedTask {
    pub id: TaskId,
    pub description: String,
    pub status: TaskStatus,
    /// (item id, title, checked) for every checklist item.
    pub checklist: Vec<(ChecklistItemId, String, bool)>,
    pub event_fqdns: Vec<&'static str>,
}

/// One stored task in the mock: (description, status, checklist).
pub type StoredTask = (String, TaskStatus, Vec<(ChecklistItemId, String, bool)>);

/// In-memory repository double: `stored` holds one `StoredTask` per ID.
#[derive(Default)]
pub struct MockTaskRepository {
    pub stored: Mutex<HashMap<TaskId, StoredTask>>,
}

fn checklist_of(task: &Task) -> Vec<(ChecklistItemId, String, bool)> {
    task.checklist()
        .iter()
        .map(|item| {
            (
                *item.id(),
                item.title().value().to_string(),
                item.is_checked(),
            )
        })
        .collect()
}

impl TaskRepository for MockTaskRepository {
    fn by_id(&self, id: &TaskId) -> Result<Option<Task>> {
        Ok(self
            .stored
            .lock()
            .unwrap()
            .get(id)
            .map(|(description, status, checklist)| {
                let items = checklist
                    .iter()
                    .map(|(item_id, title, checked)| {
                        ChecklistItem::reconstitute(
                            *item_id,
                            ChecklistItemTitle::create(title).unwrap(),
                            *checked,
                        )
                    })
                    .collect();
                Task::reconstitute(
                    *id,
                    TaskDescription::create(description).unwrap(),
                    *status,
                    items,
                )
            }))
    }

    fn save(&self, task: &Task) -> Result<()> {
        self.stored.lock().unwrap().insert(
            *task.id(),
            (
                task.description().value().to_string(),
                task.status(),
                checklist_of(task),
            ),
        );
        Ok(())
    }
}

/// Unit of Work double with tracking fields.
/// - `run_called`, `save_called`, `saved_task`: what the use case did;
/// - `fail_on_save`: simulate a database failure.
#[derive(Default)]
pub struct MockUnitOfWork {
    pub tasks: MockTaskRepository,
    pub run_called: Mutex<bool>,
    pub save_called: Mutex<bool>,
    pub saved_task: Mutex<Option<SavedTask>>,
    pub fail_on_save: bool,
}

impl MockUnitOfWork {
    pub fn failing() -> Self {
        Self {
            fail_on_save: true,
            ..Self::default()
        }
    }

    /// Puts an existing task in the mock repository and returns its ID.
    pub fn with_task(&self, description: &str, status: TaskStatus) -> TaskId {
        self.with_task_and_checklist(description, status, &[])
    }

    /// Like `with_task`, plus checklist items given as (title, checked).
    /// Returns the task ID and the generated item IDs, in order.
    pub fn with_task_and_checklist(
        &self,
        description: &str,
        status: TaskStatus,
        items: &[(&str, bool)],
    ) -> TaskId {
        let id = TaskId::generate();
        let checklist = items
            .iter()
            .map(|(title, checked)| (ChecklistItemId::generate(), title.to_string(), *checked))
            .collect();
        self.tasks
            .stored
            .lock()
            .unwrap()
            .insert(id, (description.to_string(), status, checklist));
        id
    }

    /// The checklist item IDs of a stored task, in order.
    pub fn item_ids(&self, task_id: &TaskId) -> Vec<ChecklistItemId> {
        self.tasks.stored.lock().unwrap()[task_id]
            .2
            .iter()
            .map(|(id, _, _)| *id)
            .collect()
    }

    pub fn saved(&self) -> Option<SavedTask> {
        self.saved_task.lock().unwrap().clone()
    }
}

impl UnitOfWork for MockUnitOfWork {
    fn run(&self, action: &mut dyn FnMut(&dyn UnitOfWork) -> Result<()>) -> Result<()> {
        *self.run_called.lock().unwrap() = true;
        action(self)
    }

    fn tasks(&self) -> &dyn TaskRepository {
        &self.tasks
    }

    fn save(&self, task: Task) -> Result<()> {
        *self.save_called.lock().unwrap() = true;
        if self.fail_on_save {
            return Err(Error::Infrastructure("database unavailable".into()));
        }
        self.tasks.save(&task)?;
        *self.saved_task.lock().unwrap() = Some(SavedTask {
            id: *task.id(),
            description: task.description().value().to_string(),
            status: task.status(),
            checklist: checklist_of(&task),
            event_fqdns: task.domain_events().iter().map(|e| e.fqdn()).collect(),
        });
        Ok(())
    }
}

/// Publisher double: remembers the FQDN of every published event, in order.
#[derive(Default)]
pub struct RecordingPublisher {
    pub published: Mutex<Vec<&'static str>>,
}

impl RecordingPublisher {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn fqdns(&self) -> Vec<&'static str> {
        self.published.lock().unwrap().clone()
    }
}

impl DomainEventPublisher for RecordingPublisher {
    fn publish(&self, event: Box<dyn DomainEvent>) {
        self.published.lock().unwrap().push(event.fqdn());
    }
}

/// Logger double: remembers every line, prefixed with its level.
#[derive(Default)]
pub struct RecordingLogger {
    pub lines: Mutex<Vec<String>>,
}

impl RecordingLogger {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn lines(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }

    /// True when some line contains `text`.
    pub fn contains(&self, text: &str) -> bool {
        self.lines().iter().any(|line| line.contains(text))
    }
}

impl Logger for RecordingLogger {
    fn info(&self, message: &str) {
        self.lines.lock().unwrap().push(format!("INFO {message}"));
    }

    fn error(&self, message: &str) {
        self.lines.lock().unwrap().push(format!("ERROR {message}"));
    }
}
