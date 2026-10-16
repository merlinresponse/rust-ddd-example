//! Infrastructure layer: technical details that implement the ports of the inner layers.
//! Swap these out (e.g. Postgres instead of in-memory) without touching domain or application.

pub mod events;
pub mod persistence;
pub mod shared;
pub mod web_api;
