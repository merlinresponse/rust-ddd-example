use crate::domain::shared::DomainError;

/// TaskStatus — where a task is in its life.
///
/// DDD role: Value Object with a state machine (a fixed list of states plus the moves allowed
///   between them).
/// Business meaning: a task is waiting (`Todo`), being worked on (`InProgress`) or finished (`Done`).
/// Allowed transitions:
///
/// | from         | to                  |
/// |--------------|---------------------|
/// | `Todo`       | `InProgress`, `Done`|
/// | `InProgress` | `Done`              |
/// | `Done`       | — (final)           |
///
/// Why modelled this way: the whole rulebook is one `match` in `allowed_next`, so a reader sees
///   every legal move in one place, and adding a status forces the compiler to ask you about it.
/// Pattern: Value Object (Eric Evans, Domain-Driven Design, Ch. 5); state transition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl TaskStatus {
    pub fn allowed_next(self) -> &'static [TaskStatus] {
        match self {
            Self::Todo => &[Self::InProgress, Self::Done],
            Self::InProgress => &[Self::Done],
            Self::Done => &[],
        }
    }

    /// Returns the new status, or an error if this move is not in the table above.
    pub fn transition_to(self, next: TaskStatus) -> Result<TaskStatus, DomainError> {
        if self.allowed_next().contains(&next) {
            Ok(next)
        } else {
            Err(DomainError::InvalidStatusTransition {
                from: self.as_str(),
                to: next.as_str(),
            })
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Done => "done",
        }
    }

    /// Reads a status back from storage.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "todo" => Ok(Self::Todo),
            "in_progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            other => Err(DomainError::UnknownTaskStatus(other.to_string())),
        }
    }
}
