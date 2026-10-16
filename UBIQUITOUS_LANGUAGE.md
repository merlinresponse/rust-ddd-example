# Ubiquitous language: Task Management context

These are the words business people and code share. A term means exactly this, *inside this
bounded context*.

| Term | Meaning | In code |
|---|---|---|
| Task | One thing a user has to do. | `Task` (aggregate root) |
| Description | What the task says, 1–255 characters, trimmed. | `TaskDescription` |
| Status | Where the task is in its life: todo, in progress, done. | `TaskStatus` |
| Todo | Not started yet. Every new task starts here. | `TaskStatus::Todo` |
| In progress | Someone is working on it. | `TaskStatus::InProgress` |
| Done | Finished. Final: it cannot be reopened or reworded. | `TaskStatus::Done` |
| Start | Move a todo task to in progress. | `Task::start`, `StartTask` |
| Complete | Move a todo or in-progress task to done. | `Task::complete`, `CompleteTask` |
| Reword | Change the description of a task that is not done. | `Task::change_description` |
| Checklist | The smaller steps of one task. Belongs to exactly one task. | `Task::checklist()` |
| Checklist item | One step on a checklist, with a title and a checkbox. Two items may share a title and are still different items. | `ChecklistItem` (entity), `ChecklistItemId` |
| Check (an item) | Tick a checklist item off. Once checked, it stays checked. | `Task::check_checklist_item` |
| Open item | A checklist item that is not checked yet. A task with open items cannot be completed. | `TaskHasOpenChecklistItems` |
| Task created / started / completed / description changed | Facts announced to the rest of the system after they are saved. | `events/` |
| Congratulations | A friendly message logged when a task is completed. | `WhenTaskCompletedLogCongratulations` |
| Audit log | A record of every rewording, with the old and new text. | `WhenTaskDescriptionChangedWriteAuditLog` |
