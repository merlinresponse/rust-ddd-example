use crate::application::contracts::ports::Logger;

/// ConsoleLogger — adapter for the `Logger` port: info to stdout, errors to stderr.
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
