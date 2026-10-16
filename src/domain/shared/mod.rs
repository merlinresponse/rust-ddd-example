//! Shared kernel: small building blocks every aggregate in this project reuses.

mod aggregate_root;
mod domain_error;
mod domain_event;
mod domain_event_publisher;
mod entity;

pub use aggregate_root::{AggregateRoot, DomainEvents};
pub use domain_error::{DomainError, Error, Result};
pub use domain_event::DomainEvent;
pub use domain_event_publisher::DomainEventPublisher;
pub use entity::Entity;
