use std::fmt;

use uuid::Uuid;

use crate::domain::shared::DomainError;

/// TaskId — the identity of one task.
///
/// DDD role: Typed ID (a Value Object that identifies an Entity).
/// Business meaning: what makes a task "the same task" over time, whatever its description or
///   status becomes.
/// Invariants: always a valid UUID.
/// Why modelled this way: a dedicated type instead of a bare `Uuid` or `String`, so the compiler
///   stops you from passing, say, a user ID where a task ID is expected.
/// Pattern: Entity identity (Eric Evans, Domain-Driven Design, Ch. 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(Uuid);

impl TaskId {
    /// A brand-new identity, used when a task is created.
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parses an identity that came from outside (a URL, the database).
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        Uuid::parse_str(raw.trim())
            .map(Self)
            .map_err(|_| DomainError::InvalidTaskId(raw.to_string()))
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
