use std::time::SystemTime;

use super::events::{
    ChecklistItemAdded, ChecklistItemChecked, TaskCompleted, TaskCreated, TaskDescriptionChanged,
    TaskStarted,
};
use super::value_objects::{ChecklistItemTitle, TaskDescription, TaskStatus};
use super::{ChecklistItem, ChecklistItemId, TaskId};
use crate::domain::shared::{AggregateRoot, DomainError, DomainEvent, DomainEvents, Entity};

/// Task — one thing a user has to do.
///
/// DDD role: Aggregate Root (an Entity, i.e. something with an identity that lasts over time,
///   which is also the single entry point for every change to the task).
/// Business meaning: an item on a to-do list with a description, a status and an optional
///   checklist of smaller steps.
///
/// What it is made of (the aggregate):
/// - `Task` itself — the root entity, identified by `TaskId`;
/// - `ChecklistItem` — inner entities, each identified by a `ChecklistItemId`;
/// - `TaskDescription`, `TaskStatus`, `ChecklistItemTitle` — value objects.
///
/// Invariants:
/// - the description is always valid (guaranteed by `TaskDescription`);
/// - the status only moves along the table in `TaskStatus`;
/// - a `Done` task can no longer be reworded or get new checklist items;
/// - a task cannot be completed while any checklist item is still open. This rule needs the
///   task AND all its items at once — the reason items live inside this aggregate.
///
/// Events raised: `TaskCreated`, `TaskStarted`, `TaskCompleted`, `TaskDescriptionChanged`,
///   `ChecklistItemAdded`, `ChecklistItemChecked`.
///   They are only recorded here; the Unit of Work publishes them after the save commits.
/// Why modelled this way: fields are private and there are no setters. You cannot write
///   `task.status = Done`; you must call `complete()`, which checks the rules first. The
///   article this project is based on had a `Task` struct with public fields — the rules then
///   live wherever someone remembers to check them.
/// Pattern: Aggregate + Entity (Eric Evans, Domain-Driven Design, Ch. 5–6).
#[derive(Debug)]
pub struct Task {
    id: TaskId,
    description: TaskDescription,
    status: TaskStatus,
    checklist: Vec<ChecklistItem>,
    events: DomainEvents,
}

impl Task {
    /// Creates a NEW task (status `Todo`) and raises `TaskCreated`.
    pub fn create(description: TaskDescription) -> Self {
        let mut task = Self {
            id: TaskId::generate(),
            description,
            status: TaskStatus::Todo,
            checklist: Vec::new(),
            events: DomainEvents::default(),
        };
        task.events.raise(TaskCreated {
            task_id: task.id.to_string(),
            description: task.description.value().to_string(),
            occurred_on: SystemTime::now(),
        });
        task
    }

    /// Rebuilds an EXISTING task from storage. Raises no events: nothing new happened.
    pub fn reconstitute(
        id: TaskId,
        description: TaskDescription,
        status: TaskStatus,
        checklist: Vec<ChecklistItem>,
    ) -> Self {
        Self {
            id,
            description,
            status,
            checklist,
            events: DomainEvents::default(),
        }
    }

    /// Someone starts working on the task.
    pub fn start(&mut self) -> Result<(), DomainError> {
        self.status = self.status.transition_to(TaskStatus::InProgress)?;
        self.events.raise(TaskStarted {
            task_id: self.id.to_string(),
            occurred_on: SystemTime::now(),
        });
        Ok(())
    }

    /// The task is finished.
    pub fn complete(&mut self) -> Result<(), DomainError> {
        // FLOW 6a — WHAT: the status value object checks its transition table.
        // WHY first: `?` returns early on a forbidden move, so the status stays unchanged and
        // NO event is recorded. Check, then change, then record — always in that order.
        let next = self.status.transition_to(TaskStatus::Done)?;
        // A rule that spans the root AND its inner entities: only the aggregate can check it,
        // because only the aggregate sees all items at once.
        let open = self
            .checklist
            .iter()
            .filter(|item| !item.is_checked())
            .count();
        if open > 0 {
            return Err(DomainError::TaskHasOpenChecklistItems { open });
        }
        self.status = next;
        // FLOW 6b — WHAT: record (not publish!) the fact "task completed".
        // WHY not publish now: the save might still fail; see FLOW 8.
        self.events.raise(TaskCompleted {
            task_id: self.id.to_string(),
            occurred_on: SystemTime::now(),
        });
        Ok(())
    }

    /// Rewords the task. Rewording with the same text is allowed and changes nothing.
    pub fn change_description(
        &mut self,
        new_description: TaskDescription,
    ) -> Result<(), DomainError> {
        if self.status == TaskStatus::Done {
            return Err(DomainError::CompletedTaskCannotBeChanged);
        }
        if new_description == self.description {
            return Ok(()); // nothing happened, so no event
        }
        let old = std::mem::replace(&mut self.description, new_description);
        self.events.raise(TaskDescriptionChanged {
            task_id: self.id.to_string(),
            old_description: old.value().to_string(),
            new_description: self.description.value().to_string(),
            occurred_on: SystemTime::now(),
        });
        Ok(())
    }

    /// Adds an unchecked step to the checklist and returns its new identity.
    pub fn add_checklist_item(
        &mut self,
        title: ChecklistItemTitle,
    ) -> Result<ChecklistItemId, DomainError> {
        if self.status == TaskStatus::Done {
            return Err(DomainError::CompletedTaskCannotBeChanged);
        }
        // The ROOT creates the inner entity (ChecklistItem::new is not reachable from outside).
        let item = ChecklistItem::new(title);
        let item_id = *item.id();
        self.events.raise(ChecklistItemAdded {
            task_id: self.id.to_string(),
            item_id: item_id.to_string(),
            title: item.title().value().to_string(),
            occurred_on: SystemTime::now(),
        });
        self.checklist.push(item);
        Ok(item_id)
    }

    /// Ticks off one checklist item, found by its identity (not by its title: titles may repeat).
    /// Checking an already-checked item is allowed and changes nothing (no event).
    pub fn check_checklist_item(&mut self, item_id: &ChecklistItemId) -> Result<(), DomainError> {
        let item = self
            .checklist
            .iter_mut()
            .find(|item| item.id() == item_id)
            .ok_or_else(|| DomainError::ChecklistItemNotFound(item_id.to_string()))?;
        if item.check() {
            self.events.raise(ChecklistItemChecked {
                task_id: self.id.to_string(),
                item_id: item_id.to_string(),
                occurred_on: SystemTime::now(),
            });
        }
        Ok(())
    }

    /// Read-only view of the inner entities: outsiders can look, but only `Task` can change them.
    pub fn checklist(&self) -> &[ChecklistItem] {
        &self.checklist
    }

    pub fn description(&self) -> &TaskDescription {
        &self.description
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }
}

impl Entity for Task {
    type Id = TaskId;

    fn id(&self) -> &TaskId {
        &self.id
    }
}

impl AggregateRoot for Task {
    fn domain_events(&self) -> &[Box<dyn DomainEvent>] {
        self.events.as_slice()
    }

    fn pull_domain_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        self.events.pull()
    }
}
