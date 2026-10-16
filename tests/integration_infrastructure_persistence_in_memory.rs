//! Integration tests for the in-memory adapters: Unit of Work, repository, mapper, queries.
#![allow(non_snake_case)]

mod common;

use std::sync::Arc;

use common::RecordingPublisher;
use task_manager::application::contracts::ports::queries::{FindTaskByIdQuery, ListTasksQuery};
use task_manager::application::contracts::ports::UnitOfWork;
use task_manager::domain::shared::{AggregateRoot, DomainError, Entity, Error};
use task_manager::domain::task_management::tasks::events::{TaskCompleted, TaskCreated};
use task_manager::domain::task_management::tasks::value_objects::{
    ChecklistItemTitle, TaskDescription, TaskStatus,
};
use task_manager::domain::task_management::tasks::Task;
use task_manager::infrastructure::persistence::in_memory::queries::{
    InMemoryFindTaskByIdQuery, InMemoryListTasksQuery,
};
use task_manager::infrastructure::persistence::in_memory::repositories::mappers::TaskRecord;
use task_manager::infrastructure::persistence::in_memory::shared::{
    InMemoryStore, InMemoryUnitOfWork,
};

fn setup() -> (
    Arc<InMemoryStore>,
    Arc<RecordingPublisher>,
    InMemoryUnitOfWork,
) {
    let store = Arc::new(InMemoryStore::default());
    let publisher = RecordingPublisher::new();
    let uow = InMemoryUnitOfWork::new(store.clone(), publisher.clone());
    (store, publisher, uow)
}

#[test]
fn in_memory_unit_of_work__successful_run__stores_task_and_publishes_events_after_commit() {
    // Arrange
    let (store, publisher, uow) = setup();
    let mut task = Some(Task::create(TaskDescription::create("Buy milk").unwrap()));
    let id = *task.as_ref().unwrap().id();
    // Act
    uow.run(&mut |uow| uow.save(task.take().unwrap())).unwrap();
    // Assert
    assert_eq!(publisher.fqdns(), vec![TaskCreated::FQDN]);
    let reloaded = uow.tasks().by_id(&id).unwrap().unwrap();
    assert_eq!(reloaded.description().value(), "Buy milk");
    assert!(
        reloaded.domain_events().is_empty(),
        "reconstitute must not replay events"
    );
    assert_eq!(store.rows().len(), 1);
}

#[test]
fn in_memory_unit_of_work__failing_run__rolls_back_and_publishes_nothing() {
    // Arrange
    let (store, publisher, uow) = setup();
    let mut task = Some(Task::create(TaskDescription::create("Buy milk").unwrap()));
    // Act: save succeeds, then a later step fails → the whole run must be undone.
    let result = uow.run(&mut |uow| {
        uow.save(task.take().unwrap())?;
        Err(DomainError::CompletedTaskCannotBeChanged.into())
    });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::CompletedTaskCannotBeChanged))
    );
    assert!(store.rows().is_empty());
    assert!(publisher.fqdns().is_empty());
}

#[test]
fn in_memory_unit_of_work__load_change_save__publishes_only_the_new_event() {
    // Arrange
    let (_store, publisher, uow) = setup();
    let mut task = Some(Task::create(TaskDescription::create("Buy milk").unwrap()));
    let id = *task.as_ref().unwrap().id();
    uow.run(&mut |uow| uow.save(task.take().unwrap())).unwrap();
    // Act
    uow.run(&mut |uow| {
        let mut task = uow.tasks().by_id(&id)?.unwrap();
        task.complete()?;
        uow.save(task)
    })
    .unwrap();
    // Assert
    assert_eq!(
        publisher.fqdns(),
        vec![TaskCreated::FQDN, TaskCompleted::FQDN]
    );
    assert_eq!(
        uow.tasks().by_id(&id).unwrap().unwrap().status(),
        TaskStatus::Done
    );
}

#[test]
fn task_record_mapper__corrupt_row__becomes_infrastructure_error_not_panic() {
    // Arrange
    let (store, _publisher, uow) = setup();
    let id = task_manager::domain::task_management::tasks::TaskId::generate();
    store.rows().insert(
        id.to_string(),
        TaskRecord {
            id: id.to_string(),
            description: "Buy milk".into(),
            status: "archived".into(),
            checklist: vec![],
        },
    );
    // Act
    let result = uow.tasks().by_id(&id);
    // Assert
    assert!(matches!(result, Err(Error::Infrastructure(_))));
}

#[test]
fn in_memory_queries__after_save__return_plain_views() {
    // Arrange
    let (store, _publisher, uow) = setup();
    let mut task = Some(Task::create(TaskDescription::create("Buy milk").unwrap()));
    let id = *task.as_ref().unwrap().id();
    uow.run(&mut |uow| uow.save(task.take().unwrap())).unwrap();
    // Act
    let one = InMemoryFindTaskByIdQuery::new(store.clone())
        .fetch(&id)
        .unwrap()
        .unwrap();
    let all = InMemoryListTasksQuery::new(store).fetch().unwrap();
    // Assert
    assert_eq!(one.description, "Buy milk");
    assert_eq!(one.status, "todo");
    assert_eq!(all, vec![one]);
}

#[test]
fn in_memory_task_repository__task_with_checklist__round_trips_with_same_item_identities() {
    // Arrange
    let (_store, _publisher, uow) = setup();
    let mut task = Task::create(TaskDescription::create("Bake a cake").unwrap());
    let id = *task.id();
    let eggs = task
        .add_checklist_item(ChecklistItemTitle::create("Buy eggs").unwrap())
        .unwrap();
    task.check_checklist_item(&eggs).unwrap();
    let mut task = Some(task);
    // Act
    uow.run(&mut |uow| uow.save(task.take().unwrap())).unwrap();
    let reloaded = uow.tasks().by_id(&id).unwrap().unwrap();
    // Assert: after storage it is still THE SAME item (same identity), still checked.
    assert_eq!(reloaded.checklist().len(), 1);
    assert_eq!(reloaded.checklist()[0].id(), &eggs);
    assert!(reloaded.checklist()[0].is_checked());
}
