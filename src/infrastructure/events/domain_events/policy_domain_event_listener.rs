use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::application::contracts::ports::Logger;
use crate::application::policies::PolicyRegistry;
use crate::domain::shared::{DomainEvent, DomainEventPublisher};
use crate::infrastructure::shared::lock;

/// PolicyDomainEventListener — receives every committed event and runs the policies for it.
///
/// DDD role: Adapter for the `DomainEventPublisher` port (an in-process event bus).
/// Hexagonal role: BOTH sides at once, a good example of why the direction matters:
///   - DRIVEN adapter for the `DomainEventPublisher` port: the Unit of Work calls it to publish;
///   - DRIVING adapter for the `Policy` port: it calls into the application to run policies.
///
/// What it does for each event:
/// 1. logs the event;
/// 2. looks up its FQDN in the policy registry;
/// 3. runs each matching policy on its OWN background thread ("fire-and-forget").
///
/// Why background threads: the HTTP request that completed the task should not wait for the
///   congratulation (or, later, a slow e-mail), and a failing policy must not turn a successful
///   request into an error.
/// Why `catch_unwind`: even a policy that panics only loses its own reaction; it cannot take
///   the server down or affect other policies.
/// Not implemented (std threads cannot be cancelled): a timeout per policy. With tokio you would
///   wrap the call in `tokio::time::timeout`. A real message bus would also add retries.
pub struct PolicyDomainEventListener {
    registry: PolicyRegistry,
    logger: Arc<dyn Logger>,
    running: Mutex<Vec<JoinHandle<()>>>,
}

impl PolicyDomainEventListener {
    pub fn new(registry: PolicyRegistry, logger: Arc<dyn Logger>) -> Self {
        Self {
            registry,
            logger,
            running: Mutex::new(Vec::new()),
        }
    }

    /// Blocks until every policy started so far has finished. For tests and graceful shutdown;
    /// never call it from inside a policy.
    pub fn when_idle(&self) {
        let handles = std::mem::take(&mut *lock(&self.running));
        for handle in handles {
            let _ = handle.join(); // panics were already caught and logged inside the thread
        }
    }
}

impl DomainEventPublisher for PolicyDomainEventListener {
    fn publish(&self, event: Box<dyn DomainEvent>) {
        // POLICY FLOW P1 — WHAT: every committed event is logged, whether or not anyone reacts.
        self.logger
            .info(&format!("[event] {} {:?}", event.fqdn(), event));

        // POLICY FLOW P2 — WHAT: who reacts to this event? (src/application/policies.rs)
        let Some(policies) = self.registry.get(event.fqdn()) else {
            return; // nobody reacts to this event, and that is fine
        };

        // `Box` → `Arc`: several threads now share the same event, read-only.
        let event: Arc<dyn DomainEvent> = Arc::from(event);
        let mut running = lock(&self.running);
        running.retain(|handle| !handle.is_finished()); // forget threads that are done
        for policy in policies {
            let (policy, event, logger) = (
                Arc::clone(policy),
                Arc::clone(&event),
                Arc::clone(&self.logger),
            );
            // POLICY FLOW P3 — WHAT: run the policy on its own thread and return immediately.
            // WHY: the use case (and the HTTP response) does not wait for reactions.
            running.push(thread::spawn(move || {
                let outcome = panic::catch_unwind(AssertUnwindSafe(|| policy.handle(&*event)));
                // POLICY FLOW P6 — WHAT: a failure is logged, never re-thrown.
                // WHY: the task change was already committed; a reaction failing cannot undo it.
                match outcome {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => logger.error(&format!(
                        "policy {} failed for {}: {error}",
                        policy.name(),
                        event.fqdn()
                    )),
                    Err(_) => logger.error(&format!(
                        "policy {} panicked for {}",
                        policy.name(),
                        event.fqdn()
                    )),
                }
            }));
        }
    }
}
