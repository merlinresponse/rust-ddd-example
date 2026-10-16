//! Unit tests for the task policies (tests mirror src/application/tasks/policies).
#![allow(non_snake_case)]

mod common;

use std::time::SystemTime;

use common::RecordingLogger;
use task_manager::application::contracts::ports::Policy;
use task_manager::application::policies::policy_registry;
use task_manager::application::tasks::policies::{
    WhenTaskCompletedLogCongratulations, WhenTaskDescriptionChangedWriteAuditLog,
};
use task_manager::domain::shared::Error;
use task_manager::domain::task_management::tasks::events::{
    TaskCompleted, TaskCreated, TaskDescriptionChanged,
};

fn completed(task_id: &str) -> TaskCompleted {
    TaskCompleted {
        task_id: task_id.into(),
        occurred_on: SystemTime::now(),
    }
}

#[test]
fn when_task_completed_log_congratulations__task_completed__logs_congratulations() {
    // Arrange
    let logger = RecordingLogger::new();
    let policy = WhenTaskCompletedLogCongratulations::new(logger.clone());
    // Act
    let result = policy.handle(&completed("task-1"));
    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        logger.lines(),
        vec!["INFO Well done! Task task-1 is completed."]
    );
}

#[test]
fn when_task_completed_log_congratulations__same_event_twice__is_harmless() {
    // Arrange: a real message bus may deliver an event more than once.
    let logger = RecordingLogger::new();
    let policy = WhenTaskCompletedLogCongratulations::new(logger.clone());
    let event = completed("task-1");
    // Act
    policy.handle(&event).unwrap();
    let second = policy.handle(&event);
    // Assert
    assert_eq!(second, Ok(()));
    assert_eq!(logger.lines().len(), 2);
}

#[test]
fn when_task_completed_log_congratulations__wrong_event__returns_error_and_logs_nothing() {
    // Arrange
    let logger = RecordingLogger::new();
    let policy = WhenTaskCompletedLogCongratulations::new(logger.clone());
    let wrong = TaskCreated {
        task_id: "task-1".into(),
        description: "x".into(),
        occurred_on: SystemTime::now(),
    };
    // Act
    let result = policy.handle(&wrong);
    // Assert
    assert!(matches!(result, Err(Error::Infrastructure(_))));
    assert!(logger.lines().is_empty());
}

#[test]
fn when_task_description_changed_write_audit_log__description_changed__logs_old_and_new() {
    // Arrange
    let logger = RecordingLogger::new();
    let policy = WhenTaskDescriptionChangedWriteAuditLog::new(logger.clone());
    let event = TaskDescriptionChanged {
        task_id: "task-1".into(),
        old_description: "Buy milk".into(),
        new_description: "Buy oat milk".into(),
        occurred_on: SystemTime::now(),
    };
    // Act
    policy.handle(&event).unwrap();
    // Assert
    assert_eq!(
        logger.lines(),
        vec!["INFO [audit] task task-1 reworded from \"Buy milk\" to \"Buy oat milk\""]
    );
}

#[test]
fn policy_registry__lists_who_reacts_to_what() {
    // Arrange / Act
    let registry = policy_registry(RecordingLogger::new());
    let names = |fqdn: &str| -> Vec<&str> {
        registry
            .get(fqdn)
            .map(|ps| ps.iter().map(|p| p.name()).collect())
            .unwrap_or_default()
    };
    // Assert
    assert_eq!(
        names(TaskCompleted::FQDN),
        vec![WhenTaskCompletedLogCongratulations::NAME]
    );
    assert_eq!(
        names(TaskDescriptionChanged::FQDN),
        vec![WhenTaskDescriptionChangedWriteAuditLog::NAME]
    );
    assert!(
        names(TaskCreated::FQDN).is_empty(),
        "nobody reacts to TaskCreated"
    );
}
