use crate::domain::shared::DomainError;

/// ChecklistItemTitle — the text of one step on a task's checklist ("Buy eggs").
///
/// DDD role: Value Object (defined only by its value, immutable, validated on creation).
/// Business meaning: what the user reads next to the checkbox.
/// Invariants: never empty after trimming; at most 100 characters.
/// Why a Value Object and not a plain `String`: the rule "1–100 characters" lives here once.
///   Two titles with the same text are fully interchangeable, which is exactly what makes it
///   a value and not an entity (compare `ChecklistItem`, which has an identity).
/// Pattern: Value Object (Eric Evans, Domain-Driven Design, Ch. 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItemTitle(String);

impl ChecklistItemTitle {
    pub const MAX_LENGTH: usize = 100;

    pub fn create(raw: &str) -> Result<Self, DomainError> {
        let text = raw.trim();
        let length = text.chars().count();
        if length == 0 {
            return Err(DomainError::EmptyChecklistItemTitle);
        }
        if length > Self::MAX_LENGTH {
            return Err(DomainError::ChecklistItemTitleTooLong {
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
