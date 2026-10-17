/// Logger — port for writing log lines.
///
/// DDD role: Port (an interface the application owns; infrastructure decides where lines go).
/// Hexagonal role: DRIVEN port (secondary port): something the application NEEDS from the
///   outside world. Defined on the inside, implemented by a driven adapter outside:
///   `ConsoleLogger` (and a hand-written double in tests/common).
/// Why modelled this way: policies and the event listener log through this trait, so tests can
///   pass a `RecordingLogger` and check exactly what was written, and production can swap the
///   console for a file or a log service without touching the application layer.
/// Pattern: Ports and Adapters (Hexagonal Architecture).
pub trait Logger: Send + Sync {
    fn info(&self, message: &str);
    fn error(&self, message: &str);
}
