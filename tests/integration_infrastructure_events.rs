//! Integration tests for PolicyDomainEventListener: dispatching, background threads, failures.
#![allow(non_snake_case)]

mod common;

use std::sync::Arc;
use std::time::SystemTime;

use common::RecordingLogger;
use task_manager::application::contracts::ports::Policy;
use task_manager::application::policies::{policy_registry, PolicyRegistry};
use task_manager::domain::shared::{DomainEvent, DomainEventPublisher, Error, Result};
use task_manager::domain::task_management::tasks::events::{TaskCompleted, TaskCreated};
use task_manager::infrastructure::events::domain_events::PolicyDomainEventListener;

fn completed() -> Box<TaskCompleted> {
    Box::new(TaskCompleted {
        task_id: "task-1".into(),
        occurred_on: SystemTime::now(),
    })
}

/// A policy that always fails, or always panics, to prove the listener isolates it.
struct BrokenPolicy {
    panics: bool,
}

impl Policy for BrokenPolicy {
    fn name(&self) -> &'static str {
        if self.panics {
            "PanickingPolicy"
        } else {
            "FailingPolicy"
        }
    }

    fn handle(&self, _event: &dyn DomainEvent) -> Result<()> {
        if self.panics {
            panic!("boom");
        }
        Err(Error::Infrastructure("mail server down".into()))
    }
}

#[test]
fn policy_domain_event_listener__registered_event__logs_event_and_runs_policy() {
    // Arrange
    let logger = RecordingLogger::new();
    let listener = PolicyDomainEventListener::new(policy_registry(logger.clone()), logger.clone());
    // Act
    listener.publish(completed());
    listener.when_idle();
    // Assert
    assert!(logger.contains("[event] TaskApp.TaskManagement.TaskCompleted"));
    assert!(logger.contains("Well done! Task task-1 is completed."));
}

#[test]
fn policy_domain_event_listener__event_without_policy__is_only_logged() {
    // Arrange
    let logger = RecordingLogger::new();
    let listener = PolicyDomainEventListener::new(policy_registry(logger.clone()), logger.clone());
    // Act
    listener.publish(Box::new(TaskCreated {
        task_id: "task-1".into(),
        description: "Buy milk".into(),
        occurred_on: SystemTime::now(),
    }));
    listener.when_idle();
    // Assert
    assert_eq!(logger.lines().len(), 1);
    assert!(logger.contains("[event] TaskApp.TaskManagement.TaskCreated"));
}

#[test]
fn policy_domain_event_listener__failing_and_panicking_policies__are_logged_and_others_still_run() {
    // Arrange: three policies on the same event; two of them are broken.
    let logger = RecordingLogger::new();
    let mut registry: PolicyRegistry = policy_registry(logger.clone());
    let reactions = registry.entry(TaskCompleted::FQDN).or_default();
    reactions.push(Arc::new(BrokenPolicy { panics: false }));
    reactions.push(Arc::new(BrokenPolicy { panics: true }));
    let listener = PolicyDomainEventListener::new(registry, logger.clone());
    // Act: `publish` itself must not fail or panic.
    listener.publish(completed());
    listener.when_idle();
    // Assert
    assert!(logger.contains("Well done! Task task-1 is completed."));
    assert!(logger.contains("ERROR policy FailingPolicy failed for TaskApp.TaskManagement.TaskCompleted: Infrastructure failure: mail server down"));
    assert!(logger.contains(
        "ERROR policy PanickingPolicy panicked for TaskApp.TaskManagement.TaskCompleted"
    ));
}
