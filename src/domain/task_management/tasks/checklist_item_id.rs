use std::fmt;

use uuid::Uuid;

use crate::domain::shared::DomainError;

/// ChecklistItemId — the identity of one checklist item.
///
/// DDD role: Typed ID (a Value Object that identifies an Entity).
/// Business meaning: lets a user say "tick off THIS item", even when two items share a title.
/// Invariants: always a valid UUID. Unique within its task (it is generated, never typed in).
/// Why modelled this way: its own type, so the compiler stops you from passing a `TaskId`
///   where a `ChecklistItemId` is expected — both are UUIDs underneath.
/// Pattern: Entity identity (Eric Evans, Domain-Driven Design, Ch. 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChecklistItemId(Uuid);

impl ChecklistItemId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        Uuid::parse_str(raw.trim())
            .map(Self)
            .map_err(|_| DomainError::InvalidChecklistItemId(raw.to_string()))
    }
}

impl fmt::Display for ChecklistItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
