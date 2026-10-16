use super::value_objects::ChecklistItemTitle;
use super::ChecklistItemId;
use crate::domain::shared::Entity;

/// ChecklistItem — one step inside a task ("Buy eggs" inside "Bake a cake").
///
/// DDD role: Entity inside an aggregate (it has its own identity, but only lives inside a
///   `Task`; the `Task` is its aggregate root).
/// Business meaning: a box the user can tick off.
/// Invariants: the title is always valid (guaranteed by `ChecklistItemTitle`); once checked, it
///   stays checked.
///
/// Why an Entity and not a Value Object:
/// - it changes over time (unchecked → checked) and is still *the same item* afterwards;
/// - two items can have the same title and still be different items, so the user must be able
///   to point at one: "tick off item 7f3a…".
///
/// Why not its own Aggregate (with its own repository):
/// - it makes no sense without its task;
/// - the rule "a task cannot be completed while items are open" needs the task and all its items
///   to be consistent at the same moment, so they must be saved together.
///
/// How the aggregate protects it: `new` and `check` are `pub(super)`, meaning only code in this
///   `tasks` module — in practice `Task` — can create or change an item. Outside code gets
///   read-only access through `Task::checklist()`.
/// Pattern: Entity (Eric Evans, Domain-Driven Design, Ch. 5) inside an Aggregate (Ch. 6).
#[derive(Debug)]
pub struct ChecklistItem {
    id: ChecklistItemId,
    title: ChecklistItemTitle,
    checked: bool,
}

impl ChecklistItem {
    /// A new, unchecked item. Only `Task::add_checklist_item` calls this.
    pub(super) fn new(title: ChecklistItemTitle) -> Self {
        Self {
            id: ChecklistItemId::generate(),
            title,
            checked: false,
        }
    }

    /// Rebuilds an existing item from storage (used by mappers, together with `Task::reconstitute`).
    pub fn reconstitute(id: ChecklistItemId, title: ChecklistItemTitle, checked: bool) -> Self {
        Self { id, title, checked }
    }

    /// Ticks the item off. Returns `true` if something changed, `false` if it was already checked.
    /// Only `Task` calls this, so the task can raise the event and enforce its own rules.
    pub(super) fn check(&mut self) -> bool {
        if self.checked {
            return false;
        }
        self.checked = true;
        true
    }

    pub fn title(&self) -> &ChecklistItemTitle {
        &self.title
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }
}

impl Entity for ChecklistItem {
    type Id = ChecklistItemId;

    fn id(&self) -> &ChecklistItemId {
        &self.id
    }
}
