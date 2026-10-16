use crate::domain::shared::DomainError;

/// TaskDescription — what has to be done, in the user's words.
///
/// DDD role: Value Object (an object defined only by its value; two descriptions with the same
///   text are interchangeable, and it never changes after creation).
/// Business meaning: the text a user reads in their to-do list.
/// Invariants: never empty after trimming; at most 255 characters.
/// Why modelled this way: the field is private and the only way in is `create()`, so an invalid
///   description simply cannot exist. Every function receiving a `TaskDescription` can trust it.
/// Pattern: Value Object (Eric Evans, Domain-Driven Design, Ch. 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDescription(String);

impl TaskDescription {
    pub const MAX_LENGTH: usize = 255;

    /// Validates and builds a description. Leading and trailing whitespace is removed.
    pub fn create(raw: &str) -> Result<Self, DomainError> {
        let text = raw.trim();
        // Count characters, not bytes: "café" is 4 characters but 5 bytes.
        let length = text.chars().count();
        if length == 0 {
            return Err(DomainError::EmptyTaskDescription);
        }
        if length > Self::MAX_LENGTH {
            return Err(DomainError::TaskDescriptionTooLong {
                max: Self::MAX_LENGTH,
                actual: length,
            });
        }
        Ok(Self(text.to_string()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
