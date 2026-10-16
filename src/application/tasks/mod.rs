//! Task use cases, split by what they do:
//!
//! - [`commands`] — requests that CHANGE a task. Each one goes through the Unit of Work, asks
//!   the Task aggregate to do something (which may refuse), and raises events.
//! - [`queries`]  — requests that only READ. They skip the aggregate and the Unit of Work and
//!   read plain data through a query port. They never change anything.
//! - [`policies`] — reactions that run AFTER a command's events were committed.
//!
//! One use case per file, named verb + noun. The folder tells you which side it is on.

pub mod commands;
pub mod policies;
pub mod queries;
