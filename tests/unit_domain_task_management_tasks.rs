//! Unit tests for the Task aggregate and its value objects (tests mirror src/domain/task_management/tasks).
#![allow(non_snake_case)]

use task_manager::domain::shared::{AggregateRoot, DomainError, Entity};
use task_manager::domain::task_management::tasks::events::{
    ChecklistItemAdded, ChecklistItemChecked, TaskCompleted, TaskCreated, TaskDescriptionChanged,
    TaskStarted,
};
use task_manager::domain::task_management::tasks::value_objects::{
    ChecklistItemTitle, TaskDescription, TaskStatus,
};
use task_manager::domain::task_management::tasks::{ChecklistItem, ChecklistItemId, Task, TaskId};

fn description(text: &str) -> TaskDescription {
    TaskDescription::create(text).unwrap()
}

fn fqdns(task: &Task) -> Vec<&'static str> {
    task.domain_events().iter().map(|e| e.fqdn()).collect()
}

// ---------- TaskDescription ----------

#[test]
fn task_description__valid_text__is_trimmed() {
    // Arrange / Act
    let result = TaskDescription::create("  Buy milk  ");
    // Assert
    assert_eq!(result.unwrap().value(), "Buy milk");
}

#[test]
fn task_description__blank_text__is_rejected() {
    // Arrange / Act
    let result = TaskDescription::create("   ");
    // Assert
    assert_eq!(result, Err(DomainError::EmptyTaskDescription));
}

#[test]
fn task_description__exactly_255_characters__is_accepted() {
    // Arrange: multi-byte characters prove we count characters, not bytes.
    let text = "é".repeat(255);
    // Act / Assert
    assert!(TaskDescription::create(&text).is_ok());
}

#[test]
fn task_description__256_characters__is_rejected() {
    // Arrange
    let text = "a".repeat(256);
    // Act
    let result = TaskDescription::create(&text);
    // Assert
    assert_eq!(
        result,
        Err(DomainError::TaskDescriptionTooLong {
            max: 255,
            actual: 256
        })
    );
}

// ---------- TaskId ----------

#[test]
fn task_id__parse_of_garbage__is_rejected() {
    // Arrange / Act
    let result = TaskId::parse("not-a-uuid");
    // Assert
    assert!(matches!(result, Err(DomainError::InvalidTaskId(_))));
}

#[test]
fn task_id__parse_of_generated__round_trips() {
    // Arrange
    let id = TaskId::generate();
    // Act
    let parsed = TaskId::parse(&id.to_string()).unwrap();
    // Assert
    assert_eq!(parsed, id);
}

// ---------- TaskStatus: every transition, allowed and forbidden ----------

#[test]
fn task_status__all_transitions__follow_the_table() {
    use TaskStatus::*;
    let allowed = [(Todo, InProgress), (Todo, Done), (InProgress, Done)];
    for from in [Todo, InProgress, Done] {
        for to in [Todo, InProgress, Done] {
            // Act
            let result = from.transition_to(to);
            // Assert
            if allowed.contains(&(from, to)) {
                assert_eq!(result, Ok(to), "{from:?} → {to:?} should be allowed");
            } else {
                assert_eq!(
                    result,
                    Err(DomainError::InvalidStatusTransition {
                        from: from.as_str(),
                        to: to.as_str()
                    }),
                    "{from:?} → {to:?} should be forbidden"
                );
            }
        }
    }
}

#[test]
fn task_status__parse_of_stored_value__round_trips() {
    for status in [TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done] {
        assert_eq!(TaskStatus::parse(status.as_str()), Ok(status));
    }
    assert!(matches!(
        TaskStatus::parse("archived"),
        Err(DomainError::UnknownTaskStatus(_))
    ));
}

// ---------- Task aggregate ----------

#[test]
fn task__create__starts_as_todo_and_raises_task_created() {
    // Arrange / Act
    let task = Task::create(description("Buy milk"));
    // Assert
    assert_eq!(task.status(), TaskStatus::Todo);
    assert_eq!(fqdns(&task), vec![TaskCreated::FQDN]);
}

#[test]
fn task__reconstitute__raises_no_events() {
    // Arrange / Act
    let task = Task::reconstitute(
        TaskId::generate(),
        description("Buy milk"),
        TaskStatus::InProgress,
        vec![],
    );
    // Assert
    assert_eq!(task.status(), TaskStatus::InProgress);
    assert!(task.domain_events().is_empty());
}

#[test]
fn task__start_when_todo__moves_to_in_progress_and_raises_task_started() {
    // Arrange
    let mut task = Task::create(description("Buy milk"));
    task.pull_domain_events();
    // Act
    task.start().unwrap();
    // Assert
    assert_eq!(task.status(), TaskStatus::InProgress);
    assert_eq!(fqdns(&task), vec![TaskStarted::FQDN]);
}

#[test]
fn task__complete_when_in_progress__moves_to_done_and_raises_task_completed() {
    // Arrange
    let mut task = Task::reconstitute(
        TaskId::generate(),
        description("Buy milk"),
        TaskStatus::InProgress,
        vec![],
    );
    // Act
    task.complete().unwrap();
    // Assert
    assert_eq!(task.status(), TaskStatus::Done);
    assert_eq!(fqdns(&task), vec![TaskCompleted::FQDN]);
}

#[test]
fn task__complete_when_done__fails_and_raises_nothing() {
    // Arrange
    let mut task = Task::reconstitute(
        TaskId::generate(),
        description("Buy milk"),
        TaskStatus::Done,
        vec![],
    );
    // Act
    let result = task.complete();
    // Assert
    assert!(matches!(
        result,
        Err(DomainError::InvalidStatusTransition { .. })
    ));
    assert_eq!(task.status(), TaskStatus::Done);
    assert!(task.domain_events().is_empty());
}

#[test]
fn task__start_when_in_progress__fails_and_raises_nothing() {
    // Arrange
    let mut task = Task::reconstitute(
        TaskId::generate(),
        description("Buy milk"),
        TaskStatus::InProgress,
        vec![],
    );
    // Act
    let result = task.start();
    // Assert
    assert!(matches!(
        result,
        Err(DomainError::InvalidStatusTransition { .. })
    ));
    assert!(task.domain_events().is_empty());
}

#[test]
fn task__change_description__updates_text_and_raises_event_with_both_versions() {
    // Arrange
    let mut task = Task::reconstitute(
        TaskId::generate(),
        description("Buy milk"),
        TaskStatus::Todo,
        vec![],
    );
    // Act
    task.change_description(description("Buy oat milk"))
        .unwrap();
    // Assert
    assert_eq!(task.description().value(), "Buy oat milk");
    assert_eq!(fqdns(&task), vec![TaskDescriptionChanged::FQDN]);
    let event = format!("{:?}", task.domain_events()[0]);
    assert!(event.contains("Buy milk") && event.contains("Buy oat milk"));
}

#[test]
fn task__change_description_to_same_text__raises_nothing() {
    // Arrange
    let mut task = Task::reconstitute(
        TaskId::generate(),
        description("Buy milk"),
        TaskStatus::Todo,
        vec![],
    );
    // Act
    task.change_description(description("Buy milk")).unwrap();
    // Assert
    assert!(task.domain_events().is_empty());
}

#[test]
fn task__change_description_when_done__fails_and_raises_nothing() {
    // Arrange
    let mut task = Task::reconstitute(
        TaskId::generate(),
        description("Buy milk"),
        TaskStatus::Done,
        vec![],
    );
    // Act
    let result = task.change_description(description("Buy oat milk"));
    // Assert
    assert_eq!(result, Err(DomainError::CompletedTaskCannotBeChanged));
    assert_eq!(task.description().value(), "Buy milk");
    assert!(task.domain_events().is_empty());
}

#[test]
fn task__pull_domain_events__empties_the_list() {
    // Arrange
    let mut task = Task::create(description("Buy milk"));
    // Act
    let pulled = task.pull_domain_events();
    // Assert
    assert_eq!(pulled.len(), 1);
    assert!(task.domain_events().is_empty());
}

// ---------- ChecklistItemTitle (Value Object) ----------

#[test]
fn checklist_item_title__same_text__is_equal_because_values_compare_by_value() {
    // Arrange / Act
    let a = ChecklistItemTitle::create("Buy eggs").unwrap();
    let b = ChecklistItemTitle::create("  Buy eggs ").unwrap();
    // Assert: a value object has no identity; same value = same thing.
    assert_eq!(a, b);
}

#[test]
fn checklist_item_title__blank_or_too_long__is_rejected() {
    assert_eq!(
        ChecklistItemTitle::create(" "),
        Err(DomainError::EmptyChecklistItemTitle)
    );
    assert_eq!(
        ChecklistItemTitle::create(&"a".repeat(101)),
        Err(DomainError::ChecklistItemTitleTooLong {
            max: 100,
            actual: 101
        })
    );
    assert!(ChecklistItemTitle::create(&"a".repeat(100)).is_ok());
}

// ---------- ChecklistItem (Entity) ----------

#[test]
fn checklist_item__same_title_different_id__are_different_entities() {
    // Arrange
    let title = || ChecklistItemTitle::create("Call Bob").unwrap();
    let first = ChecklistItem::reconstitute(ChecklistItemId::generate(), title(), false);
    let second = ChecklistItem::reconstitute(ChecklistItemId::generate(), title(), false);
    // Act / Assert: an entity is compared by identity, not by its fields.
    assert!(!first.same_identity_as(&second));
}

#[test]
fn checklist_item__same_id_different_state__is_the_same_entity() {
    // Arrange: the same item, once before and once after it was checked.
    let id = ChecklistItemId::generate();
    let before =
        ChecklistItem::reconstitute(id, ChecklistItemTitle::create("Call Bob").unwrap(), false);
    let after =
        ChecklistItem::reconstitute(id, ChecklistItemTitle::create("Call Bob").unwrap(), true);
    // Act / Assert
    assert!(before.same_identity_as(&after));
}

// ---------- Task + its checklist (the aggregate) ----------

fn task_with_items(titles: &[&str]) -> (Task, Vec<ChecklistItemId>) {
    let mut task = Task::create(description("Bake a cake"));
    let ids = titles
        .iter()
        .map(|t| {
            task.add_checklist_item(ChecklistItemTitle::create(t).unwrap())
                .unwrap()
        })
        .collect();
    task.pull_domain_events();
    (task, ids)
}

#[test]
fn task__add_checklist_item__adds_unchecked_item_and_raises_checklist_item_added() {
    // Arrange
    let mut task = Task::create(description("Bake a cake"));
    task.pull_domain_events();
    // Act
    let item_id = task
        .add_checklist_item(ChecklistItemTitle::create("Buy eggs").unwrap())
        .unwrap();
    // Assert
    assert_eq!(task.checklist().len(), 1);
    assert_eq!(task.checklist()[0].id(), &item_id);
    assert!(!task.checklist()[0].is_checked());
    assert_eq!(fqdns(&task), vec![ChecklistItemAdded::FQDN]);
}

#[test]
fn task__add_two_items_with_the_same_title__gives_two_identities() {
    // Arrange / Act
    let (task, ids) = task_with_items(&["Call Bob", "Call Bob"]);
    // Assert
    assert_ne!(ids[0], ids[1]);
    assert_eq!(task.checklist().len(), 2);
}

#[test]
fn task__add_checklist_item_when_done__fails_and_raises_nothing() {
    // Arrange
    let mut task = Task::reconstitute(
        TaskId::generate(),
        description("Buy milk"),
        TaskStatus::Done,
        vec![],
    );
    // Act
    let result = task.add_checklist_item(ChecklistItemTitle::create("Buy eggs").unwrap());
    // Assert
    assert_eq!(result, Err(DomainError::CompletedTaskCannotBeChanged));
    assert!(task.checklist().is_empty());
    assert!(task.domain_events().is_empty());
}

#[test]
fn task__check_checklist_item__checks_only_that_item_and_raises_event() {
    // Arrange: two items with the SAME title — identity decides which one is checked.
    let (mut task, ids) = task_with_items(&["Call Bob", "Call Bob"]);
    // Act
    task.check_checklist_item(&ids[1]).unwrap();
    // Assert
    assert!(!task.checklist()[0].is_checked());
    assert!(task.checklist()[1].is_checked());
    assert_eq!(fqdns(&task), vec![ChecklistItemChecked::FQDN]);
}

#[test]
fn task__check_already_checked_item__raises_nothing() {
    // Arrange
    let (mut task, ids) = task_with_items(&["Buy eggs"]);
    task.check_checklist_item(&ids[0]).unwrap();
    task.pull_domain_events();
    // Act
    let result = task.check_checklist_item(&ids[0]);
    // Assert
    assert_eq!(result, Ok(()));
    assert!(task.domain_events().is_empty());
}

#[test]
fn task__check_unknown_item__returns_checklist_item_not_found() {
    // Arrange
    let (mut task, _) = task_with_items(&["Buy eggs"]);
    let unknown = ChecklistItemId::generate();
    // Act
    let result = task.check_checklist_item(&unknown);
    // Assert
    assert_eq!(
        result,
        Err(DomainError::ChecklistItemNotFound(unknown.to_string()))
    );
}

#[test]
fn task__complete_with_open_items__fails_stays_todo_and_raises_nothing() {
    // Arrange
    let (mut task, ids) = task_with_items(&["Buy eggs", "Buy flour"]);
    task.check_checklist_item(&ids[0]).unwrap();
    task.pull_domain_events();
    // Act
    let result = task.complete();
    // Assert
    assert_eq!(
        result,
        Err(DomainError::TaskHasOpenChecklistItems { open: 1 })
    );
    assert_eq!(task.status(), TaskStatus::Todo);
    assert!(task.domain_events().is_empty());
}

#[test]
fn task__complete_with_all_items_checked__succeeds() {
    // Arrange
    let (mut task, ids) = task_with_items(&["Buy eggs", "Buy flour"]);
    for id in &ids {
        task.check_checklist_item(id).unwrap();
    }
    task.pull_domain_events();
    // Act
    task.complete().unwrap();
    // Assert
    assert_eq!(task.status(), TaskStatus::Done);
    assert_eq!(fqdns(&task), vec![TaskCompleted::FQDN]);
}
