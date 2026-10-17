# Test doubles (`tests/common`)

Hand-written mocks, so you can read exactly what they do. No mocking library.

In hexagonal terms these are **driven adapters for tests**: each implements a driven port,
so the use cases can run without real storage or a console. The tests themselves act as
**driving adapters**: they call the use cases through the `UseCase` driving port.

| Double | Replaces | Tracking fields | Use it to… |
|---|---|---|---|
| `MockTaskRepository` | `TaskRepository` | `stored` | hold tasks for a use-case test |
| `MockUnitOfWork` | `UnitOfWork` | `run_called`, `save_called`, `saved_task`, `fail_on_save` | check that a use case loads → calls the domain → saves, or simulate a DB failure (`MockUnitOfWork::failing()`) |
| `RecordingLogger` | `Logger` | `lines` (`contains(text)`) | check what policies and the event listener logged |
| `RecordingPublisher` | `DomainEventPublisher` | `published` | check which events were published, and that none were after a rollback |

`MockUnitOfWork::with_task(description, status)` seeds an existing task and returns its ID.
`with_task_and_checklist(description, status, &[(title, checked)])` also seeds checklist items;
`item_ids(&task_id)` returns their generated IDs in order.
