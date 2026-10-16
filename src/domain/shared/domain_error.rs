use std::fmt;

/// DomainError — every business rule a user can break, as one typed enum.
///
/// DDD role: typed domain error (the "exceptions" of other languages).
/// Business meaning: each variant is one rule; the message is a full sentence a user can read.
/// Why modelled this way: callers `match` on the *variant* (e.g. `TaskNotFound` → HTTP 404),
///   never on the message text. The article this project is based on uses `Box<dyn Error>`,
///   which throws that type information away.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InvalidTaskId(String),
    EmptyTaskDescription,
    TaskDescriptionTooLong {
        max: usize,
        actual: usize,
    },
    UnknownTaskStatus(String),
    TaskNotFound(String),
    InvalidStatusTransition {
        from: &'static str,
        to: &'static str,
    },
    CompletedTaskCannotBeChanged,
    TaskHasOpenChecklistItems {
        open: usize,
    },
    InvalidChecklistItemId(String),
    EmptyChecklistItemTitle,
    ChecklistItemTitleTooLong {
        max: usize,
        actual: usize,
    },
    ChecklistItemNotFound(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTaskId(raw) => write!(f, "\"{raw}\" is not a valid task ID."),
            Self::EmptyTaskDescription => f.write_str("A task description cannot be empty."),
            Self::TaskDescriptionTooLong { max, actual } => write!(
                f,
                "A task description can have at most {max} characters, but has {actual}."
            ),
            Self::UnknownTaskStatus(raw) => write!(f, "\"{raw}\" is not a known task status."),
            Self::TaskNotFound(id) => write!(f, "Task with ID \"{id}\" was not found."),
            Self::InvalidStatusTransition { from, to } => {
                write!(f, "A task cannot move from \"{from}\" to \"{to}\".")
            }
            Self::CompletedTaskCannotBeChanged => {
                f.write_str("A completed task cannot be changed anymore.")
            }
            Self::TaskHasOpenChecklistItems { open } => write!(
                f,
                "A task cannot be completed while {open} checklist item(s) are still open."
            ),
            Self::InvalidChecklistItemId(raw) => {
                write!(f, "\"{raw}\" is not a valid checklist item ID.")
            }
            Self::EmptyChecklistItemTitle => f.write_str("A checklist item title cannot be empty."),
            Self::ChecklistItemTitleTooLong { max, actual } => write!(
                f,
                "A checklist item title can have at most {max} characters, but has {actual}."
            ),
            Self::ChecklistItemNotFound(id) => {
                write!(
                    f,
                    "Checklist item with ID \"{id}\" was not found on this task."
                )
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// Error — what a use case or port can fail with: a broken business rule, or a technical failure.
///
/// Why modelled this way: a repository is a domain port but can fail for technical reasons
///   (database down). Keeping `Infrastructure` separate means technical errors are never shown
///   to users as rule violations, and rule violations are never retried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(DomainError),
    Infrastructure(String),
}

/// Lets `?` turn a `DomainError` into an `Error` automatically.
impl From<DomainError> for Error {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(error) => error.fmt(f),
            Self::Infrastructure(message) => write!(f, "Infrastructure failure: {message}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;
