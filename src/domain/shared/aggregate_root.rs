use super::{DomainEvent, Entity};

/// DomainEvents — the events an aggregate recorded that nobody has published yet.
///
/// DDD role: building block, embedded as a private field in every aggregate root.
/// Why modelled this way: Rust has no inheritance, so instead of an `AggregateRoot` base class
///   each aggregate *contains* this struct. Because the field is private to the aggregate, only
///   the aggregate itself can `raise` an event.
#[derive(Debug, Default)]
pub struct DomainEvents {
    recorded: Vec<Box<dyn DomainEvent>>,
}

impl DomainEvents {
    /// Record that something happened inside the aggregate.
    pub fn raise(&mut self, event: impl DomainEvent + 'static) {
        self.recorded.push(Box::new(event));
    }

    /// Hand over all recorded events and empty the list.
    pub fn pull(&mut self) -> Vec<Box<dyn DomainEvent>> {
        std::mem::take(&mut self.recorded)
    }

    pub fn as_slice(&self) -> &[Box<dyn DomainEvent>] {
        &self.recorded
    }
}

/// AggregateRoot — the single entry point to a cluster of objects that must stay consistent.
///
/// DDD role: Aggregate Root. An *aggregate* is a group of objects (the root entity, inner
///   entities such as `ChecklistItem`, and value objects) treated as one unit for changes; the
///   *root* is the only object outsiders may hold or call.
/// Business meaning: it guards the business rules (invariants). There are no public setters,
///   only intention-revealing commands such as `complete()`.
/// Why modelled this way: events are NOT published by the aggregate. The Unit of Work calls
///   `pull_domain_events()` after a successful save and publishes them then.
/// Pattern: Aggregate (Eric Evans, Domain-Driven Design, Ch. 6).
pub trait AggregateRoot: Entity {
    /// Read-only view of the recorded events (handy in tests).
    fn domain_events(&self) -> &[Box<dyn DomainEvent>];

    fn pull_domain_events(&mut self) -> Vec<Box<dyn DomainEvent>>;
}
