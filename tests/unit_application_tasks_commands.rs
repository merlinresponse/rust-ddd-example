//! Unit tests for the task COMMAND use cases (tests mirror src/application/tasks/commands), using the
//! hand-written mocks in tests/common.
#![allow(non_snake_case)]

mod common;

use std::sync::Arc;

use common::MockUnitOfWork;
use task_manager::application::contracts::ports::UseCase;
use task_manager::application::tasks::commands::*;
use task_manager::domain::shared::{DomainError, Error};
use task_manager::domain::task_management::tasks::events::{
    ChecklistItemAdded, ChecklistItemChecked, TaskCompleted, TaskCreated, TaskDescriptionChanged,
    TaskStarted,
};
use task_manager::domain::task_management::tasks::value_objects::TaskStatus;
use task_manager::domain::task_management::tasks::{ChecklistItemId, TaskId};

// ---------- CreateTask ----------

#[test]
fn create_task__valid_description__saves_todo_task_with_task_created() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let use_case = CreateTask::new(uow.clone());
    // Act
    let id = use_case
        .execute(CreateTaskInput {
            description: "Buy milk".into(),
        })
        .unwrap();
    // Assert
    let saved = uow.saved().expect("task saved");
    assert_eq!(saved.id, id);
    assert_eq!(saved.status, TaskStatus::Todo);
    assert_eq!(saved.event_fqdns, vec![TaskCreated::FQDN]);
}

#[test]
fn create_task__empty_description__fails_before_the_unit_of_work() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    // Act
    let result = CreateTask::new(uow.clone()).execute(CreateTaskInput {
        description: " ".into(),
    });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::EmptyTaskDescription))
    );
    assert!(!*uow.run_called.lock().unwrap());
}

#[test]
fn create_task__unit_of_work_fails__returns_infrastructure_error() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::failing());
    // Act
    let result = CreateTask::new(uow).execute(CreateTaskInput {
        description: "Buy milk".into(),
    });
    // Assert
    assert!(matches!(result, Err(Error::Infrastructure(_))));
}

// ---------- StartTask ----------

#[test]
fn start_task__todo_task__saves_in_progress_with_task_started() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let id = uow.with_task("Buy milk", TaskStatus::Todo);
    // Act
    StartTask::new(uow.clone())
        .execute(StartTaskInput { id: id.to_string() })
        .unwrap();
    // Assert
    let saved = uow.saved().unwrap();
    assert_eq!(saved.status, TaskStatus::InProgress);
    assert_eq!(saved.event_fqdns, vec![TaskStarted::FQDN]);
}

#[test]
fn start_task__unknown_id__returns_task_not_found() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let id = TaskId::generate().to_string();
    // Act
    let result = StartTask::new(uow.clone()).execute(StartTaskInput { id: id.clone() });
    // Assert
    assert_eq!(result, Err(Error::Domain(DomainError::TaskNotFound(id))));
    assert!(!*uow.save_called.lock().unwrap());
}

// ---------- CompleteTask: happy path, not found, rule violation, UoW failure ----------

#[test]
fn complete_task__in_progress_task__saves_done_with_task_completed() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let id = uow.with_task("Buy milk", TaskStatus::InProgress);
    // Act
    CompleteTask::new(uow.clone())
        .execute(CompleteTaskInput { id: id.to_string() })
        .unwrap();
    // Assert
    let saved = uow.saved().unwrap();
    assert_eq!(saved.status, TaskStatus::Done);
    assert_eq!(saved.event_fqdns, vec![TaskCompleted::FQDN]);
}

#[test]
fn complete_task__unknown_id__returns_task_not_found() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    // Act
    let result = CompleteTask::new(uow).execute(CompleteTaskInput {
        id: TaskId::generate().to_string(),
    });
    // Assert
    assert!(matches!(
        result,
        Err(Error::Domain(DomainError::TaskNotFound(_)))
    ));
}

#[test]
fn complete_task__already_done__returns_invalid_transition_and_saves_nothing() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let id = uow.with_task("Buy milk", TaskStatus::Done);
    // Act
    let result = CompleteTask::new(uow.clone()).execute(CompleteTaskInput { id: id.to_string() });
    // Assert
    assert!(matches!(
        result,
        Err(Error::Domain(DomainError::InvalidStatusTransition { .. }))
    ));
    assert!(!*uow.save_called.lock().unwrap());
}

#[test]
fn complete_task__unit_of_work_fails__returns_infrastructure_error() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::failing());
    let id = uow.with_task("Buy milk", TaskStatus::Todo);
    // Act
    let result = CompleteTask::new(uow).execute(CompleteTaskInput { id: id.to_string() });
    // Assert
    assert!(matches!(result, Err(Error::Infrastructure(_))));
}

#[test]
fn complete_task__malformed_id__returns_invalid_task_id() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    // Act
    let result = CompleteTask::new(uow).execute(CompleteTaskInput { id: "42".into() });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::InvalidTaskId("42".into())))
    );
}

// ---------- ChangeTaskDescription ----------

#[test]
fn change_task_description__todo_task__saves_new_text_with_event() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let id = uow.with_task("Buy milk", TaskStatus::Todo);
    // Act
    ChangeTaskDescription::new(uow.clone())
        .execute(ChangeTaskDescriptionInput {
            id: id.to_string(),
            description: "Buy oat milk".into(),
        })
        .unwrap();
    // Assert
    let saved = uow.saved().unwrap();
    assert_eq!(saved.description, "Buy oat milk");
    assert_eq!(saved.event_fqdns, vec![TaskDescriptionChanged::FQDN]);
}

#[test]
fn change_task_description__done_task__returns_completed_task_cannot_be_changed() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let id = uow.with_task("Buy milk", TaskStatus::Done);
    // Act
    let result = ChangeTaskDescription::new(uow.clone()).execute(ChangeTaskDescriptionInput {
        id: id.to_string(),
        description: "Buy oat milk".into(),
    });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::CompletedTaskCannotBeChanged))
    );
    assert!(!*uow.save_called.lock().unwrap());
}

#[test]
fn change_task_description__unknown_id__returns_task_not_found() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    // Act
    let result = ChangeTaskDescription::new(uow).execute(ChangeTaskDescriptionInput {
        id: TaskId::generate().to_string(),
        description: "Buy oat milk".into(),
    });
    // Assert
    assert!(matches!(
        result,
        Err(Error::Domain(DomainError::TaskNotFound(_)))
    ));
}

// ---------- AddChecklistItem ----------

#[test]
fn add_checklist_item__todo_task__saves_task_with_new_unchecked_item() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let task_id = uow.with_task("Bake a cake", TaskStatus::Todo);
    // Act
    let item_id = AddChecklistItem::new(uow.clone())
        .execute(AddChecklistItemInput {
            task_id: task_id.to_string(),
            title: "Buy eggs".into(),
        })
        .unwrap();
    // Assert: the TASK was saved (the item has no repository of its own).
    let saved = uow.saved().unwrap();
    assert_eq!(saved.id, task_id);
    assert_eq!(
        saved.checklist,
        vec![(item_id, "Buy eggs".to_string(), false)]
    );
    assert_eq!(saved.event_fqdns, vec![ChecklistItemAdded::FQDN]);
}

#[test]
fn add_checklist_item__empty_title__fails_before_the_unit_of_work() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let task_id = uow.with_task("Bake a cake", TaskStatus::Todo);
    // Act
    let result = AddChecklistItem::new(uow.clone()).execute(AddChecklistItemInput {
        task_id: task_id.to_string(),
        title: "".into(),
    });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::EmptyChecklistItemTitle))
    );
    assert!(!*uow.run_called.lock().unwrap());
}

#[test]
fn add_checklist_item__done_task__returns_completed_task_cannot_be_changed() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let task_id = uow.with_task("Bake a cake", TaskStatus::Done);
    // Act
    let result = AddChecklistItem::new(uow.clone()).execute(AddChecklistItemInput {
        task_id: task_id.to_string(),
        title: "Buy eggs".into(),
    });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::CompletedTaskCannotBeChanged))
    );
    assert!(!*uow.save_called.lock().unwrap());
}

#[test]
fn add_checklist_item__unit_of_work_fails__returns_infrastructure_error() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::failing());
    let task_id = uow.with_task("Bake a cake", TaskStatus::Todo);
    // Act
    let result = AddChecklistItem::new(uow).execute(AddChecklistItemInput {
        task_id: task_id.to_string(),
        title: "Buy eggs".into(),
    });
    // Assert
    assert!(matches!(result, Err(Error::Infrastructure(_))));
}

// ---------- CheckChecklistItem ----------

#[test]
fn check_checklist_item__open_item__saves_it_checked_with_event() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let task_id =
        uow.with_task_and_checklist("Bake a cake", TaskStatus::Todo, &[("Buy eggs", false)]);
    let item_id = uow.item_ids(&task_id)[0];
    // Act
    CheckChecklistItem::new(uow.clone())
        .execute(CheckChecklistItemInput {
            task_id: task_id.to_string(),
            item_id: item_id.to_string(),
        })
        .unwrap();
    // Assert
    let saved = uow.saved().unwrap();
    assert_eq!(
        saved.checklist,
        vec![(item_id, "Buy eggs".to_string(), true)]
    );
    assert_eq!(saved.event_fqdns, vec![ChecklistItemChecked::FQDN]);
}

#[test]
fn check_checklist_item__item_of_another_task__returns_checklist_item_not_found() {
    // Arrange: the item exists, but on a DIFFERENT task.
    let uow = Arc::new(MockUnitOfWork::default());
    let task_id = uow.with_task("Bake a cake", TaskStatus::Todo);
    let other = uow.with_task_and_checklist("Clean up", TaskStatus::Todo, &[("Wash bowl", false)]);
    let foreign_item = uow.item_ids(&other)[0];
    // Act
    let result = CheckChecklistItem::new(uow.clone()).execute(CheckChecklistItemInput {
        task_id: task_id.to_string(),
        item_id: foreign_item.to_string(),
    });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::ChecklistItemNotFound(
            foreign_item.to_string()
        )))
    );
    assert!(!*uow.save_called.lock().unwrap());
}

#[test]
fn check_checklist_item__unknown_task__returns_task_not_found() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    // Act
    let result = CheckChecklistItem::new(uow).execute(CheckChecklistItemInput {
        task_id: TaskId::generate().to_string(),
        item_id: ChecklistItemId::generate().to_string(),
    });
    // Assert
    assert!(matches!(
        result,
        Err(Error::Domain(DomainError::TaskNotFound(_)))
    ));
}

#[test]
fn check_checklist_item__malformed_item_id__returns_invalid_checklist_item_id() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    // Act
    let result = CheckChecklistItem::new(uow).execute(CheckChecklistItemInput {
        task_id: TaskId::generate().to_string(),
        item_id: "7".into(),
    });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::InvalidChecklistItemId(
            "7".into()
        )))
    );
}

#[test]
fn complete_task__open_checklist_items__returns_task_has_open_checklist_items() {
    // Arrange
    let uow = Arc::new(MockUnitOfWork::default());
    let task_id = uow.with_task_and_checklist(
        "Bake a cake",
        TaskStatus::InProgress,
        &[("Buy eggs", true), ("Buy flour", false)],
    );
    // Act
    let result = CompleteTask::new(uow.clone()).execute(CompleteTaskInput {
        id: task_id.to_string(),
    });
    // Assert
    assert_eq!(
        result,
        Err(Error::Domain(DomainError::TaskHasOpenChecklistItems {
            open: 1
        }))
    );
    assert!(!*uow.save_called.lock().unwrap());
}
