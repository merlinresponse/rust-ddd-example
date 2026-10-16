use crate::domain::shared::Result;

/// UseCase — one thing a user can do with the system ("create a task", "complete a task").
///
/// Two kinds:
/// - command handlers: `UseCase<CompleteTaskInput>` — change state, go through the Unit of Work;
/// - query handlers: `UseCase<FindTaskByIdInput, TaskView>` — only read, through a query port.
///
/// DDD role: Application Service / Use Case (a thin coordinator: load the aggregate, call ONE
///   domain method, save it). It holds no business rules itself.
/// Why modelled this way: one small struct per operation instead of one big `TaskService`
///   (as in the article) keeps each file short and makes dependencies per operation explicit.
/// Pattern: Application Layer (Eric Evans, Domain-Driven Design, Ch. 4); Command pattern.
pub trait UseCase<Input, Output = ()> {
    fn execute(&self, input: Input) -> Result<Output>;
}
