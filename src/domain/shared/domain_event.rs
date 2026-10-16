use std::any::Any;
use std::fmt::Debug;
use std::time::SystemTime;

/// DomainEvent — an immutable, past-tense record of something that happened in the business.
///
/// DDD role: Domain Event. A *fact* ("TaskCompleted"), never a request ("CompleteTask").
/// Business meaning: other parts of the system can react to it (send an e-mail, update a report)
///   without the aggregate knowing about them.
/// Invariants: payloads hold plain values (ids, strings), never a live aggregate.
/// Why modelled this way: aggregates keep their events in one list of different event types,
///   so we need a trait object (`Box<dyn DomainEvent>`). `Send + Sync` lets events cross threads
///   (policies run on background threads). `as_any` lets a policy get the concrete event back
///   with `event.downcast_ref::<TaskCompleted>()`.
/// Pattern: Domain Events (a later addition to DDD, popularised after Evans' book).
pub trait DomainEvent: Any + Debug + Send + Sync {
    /// Fully qualified name, `Namespace.Context.EventName`, e.g. `TaskApp.TaskManagement.TaskCreated`.
    fn fqdn(&self) -> &'static str;
    fn occurred_on(&self) -> SystemTime;
    /// Always implemented as `{ self }`; needed because Rust cannot downcast a trait object directly.
    fn as_any(&self) -> &dyn Any;
}

impl dyn DomainEvent {
    /// The concrete event, if it is of type `E`; `None` otherwise.
    pub fn downcast_ref<E: DomainEvent>(&self) -> Option<&E> {
        self.as_any().downcast_ref::<E>()
    }
}
