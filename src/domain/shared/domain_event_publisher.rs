use super::DomainEvent;

/// DomainEventPublisher — port for "tell the rest of the world that this happened".
///
/// DDD role: Port (an interface the domain defines and infrastructure implements).
/// Business meaning: delivers events to whoever listens: a log, a message bus, a policy.
/// Why modelled this way: the Unit of Work calls it only AFTER data was saved, so nobody hears
///   about a change that was rolled back.
/// Pattern: Ports and Adapters (Hexagonal Architecture, Alistair Cockburn).
pub trait DomainEventPublisher: Send + Sync {
    fn publish(&self, event: Box<dyn DomainEvent>);
}
