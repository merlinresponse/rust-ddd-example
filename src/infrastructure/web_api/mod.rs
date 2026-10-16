//! Web API (presentation edge): turns HTTP requests into use-case inputs and results into JSON.
//! Controllers are thin: parse → call ONE use case → map the result. No business rules here.

pub mod controllers;
pub mod middlewares;
pub mod shared;
mod web_api_routes;

pub use web_api_routes::routes;
