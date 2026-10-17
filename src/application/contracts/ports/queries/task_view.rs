/// TaskView — read-only data describing a task, shaped for display.
///
/// DDD role: Read model (plain data for queries; it has no behaviour and enforces no rules).
/// Hexagonal role: not a port: the data that comes back through the driven query ports.
/// Why modelled this way: reading does not need the aggregate's rules, so a simple struct is
///   cheaper and cannot accidentally be used to change a task.
/// Pattern: CQRS-lite (Command–Query Responsibility Segregation; beyond Evans).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskView {
    pub id: String,
    pub description: String,
    pub status: String,
    pub checklist: Vec<ChecklistItemView>,
}

/// ChecklistItemView — read-only data for one checklist item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItemView {
    pub id: String,
    pub title: String,
    pub checked: bool,
}
