use crate::application::contracts::ports::Logger;

/// ConsoleLogger — adapter for the `Logger` port: info to stdout, errors to stderr.
/// Hexagonal role: DRIVEN adapter (secondary adapter): implements the driven port `Logger`.
///   The application calls it; swap it (e.g. for Postgres) without touching domain or application.
#[derive(Debug, Default)]
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn info(&self, message: &str) {
        println!("[info] {message}");
    }

    fn error(&self, message: &str) {
        eprintln!("[error] {message}");
    }
}
