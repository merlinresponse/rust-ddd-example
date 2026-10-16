//! Task Manager — a small layered Domain-Driven Design example.
//!
//! Read the layers from the inside out:
//!
//! 1. [`domain`]          — business rules only. Knows nothing about HTTP, JSON or storage.
//! 2. [`application`]     — use cases: "what can a user do?". Orchestrates the domain.
//! 3. [`infrastructure`]  — technical details: in-memory storage, HTTP controllers, event logging.
//! 4. [`main_modules`]    — the composition root: wires concrete pieces together.
//!
//! The dependency rule: every arrow points inward. `domain` imports nothing from the other
//! layers, `application` only imports `domain`, and so on.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod main_modules;
