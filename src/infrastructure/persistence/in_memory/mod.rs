//! In-memory adapters: a HashMap pretending to be a database. Same shape a Postgres adapter
//! folder would have (shared/, repositories/, queries/).

pub mod queries;
pub mod repositories;
pub mod shared;
