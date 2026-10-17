# Task Manager: a small layered DDD example in Rust

A to-do API built to **teach** Domain-Driven Design (DDD) and Rust together. The code is
deliberately small, and every type carries a comment that explains *what it is* and *why it
exists*.

It is based on the article
[Crafting Maintainable Rust Web Apps with Layered DDD](https://leapcell.io/blog/crafting-maintainable-rust-web-apps-with-layered-ddd)
and keeps its domain (tasks) and its layers. It also fixes a few things the article glosses
over (see [Differences from the article](#differences-from-the-article)).

## Quick start

```bash
cargo run                     # serves on http://localhost:3000 (PORT=8080 cargo run to change)
cargo test                    # 72 tests: domain, use cases, policies, storage, events, HTTP end-to-end
```

```bash
curl -s -X POST localhost:3000/api/tasks -H 'content-type: application/json' \
     -d '{"description":"Buy milk"}'                       # → {"id":"…"}
curl -s -X POST localhost:3000/api/tasks/<id>/complete      # → 204; logs [event] …TaskCompleted, then the policy logs "Well done!"
curl -s -X POST localhost:3000/api/tasks/<id>/complete      # → 409 problem+json: done is final

# checklist: a task with an open item cannot be completed
curl -s -X POST localhost:3000/api/tasks/<id2>/checklist -H 'content-type: application/json' \
     -d '{"title":"Buy eggs"}'                             # → {"id":"<item id>"}
curl -s -X POST localhost:3000/api/tasks/<id2>/checklist/<item id>/check   # → 204
```

More requests are in `tasks.http`, and the full API contract is in `config/openapi.yml`.

## How to learn from this repo (suggested order)

1. **`UBIQUITOUS_LANGUAGE.md`**: the business words first. What do *task*, *status* and *done*
   mean?
2. **`docs/CONCEPTS.md`**: what each building block is and *why* it exists, plus the Rust
   features you'll meet.
3. **`src/domain/task_management/tasks/`**: read `value_objects/`, then `checklist_item.rs` (an
   entity *inside* the aggregate), then `task.rs` (the root), then `events/`. This is where the
   rules live.
4. **`docs/CODE_FLOW.md`**: follow one request through every layer, step by step. Then run
   `grep -rn FLOW src` and read the code in the same order.
5. **`src/application/tasks/commands/complete_task.rs`**: the recipe every command follows. Then
   compare it with `queries/find_task_by_id.rs`: a read takes a much shorter path.
6. **`src/application/policies.rs`** and **`src/application/tasks/policies/`**: how the system
   reacts to events after commit (`grep -rn "POLICY FLOW" src`).
7. **`tests/`**: the tests are executable examples of every rule.

## Architecture

```
HTTP ─▶ infrastructure/web_api          DRIVING ADAPTERS (controllers: HTTP ⇄ Rust)
            │ calls
            ▼
        UseCase, Policy                  DRIVING PORTS (implemented by the core)
            │
            ▼
        application/tasks               application core: use cases, policies
        domain/task_management          application core: Task, rules, events
            │ calls
            ▼
        TaskRepository, UnitOfWork,      DRIVEN PORTS (defined by the core)
        queries, DomainEventPublisher,
        Logger
            ▲ implements
            │
        infrastructure/persistence,      DRIVEN ADAPTERS (in-memory store, listener,
        events, shared                   console logger)

        main_modules                     composition root: puts each adapter in its port
```

**Driving vs driven:** the outside *calls* a driving port; the core *calls* a driven port.
Full explanation and tables: `docs/CONCEPTS.md`, section 1b.

**The dependency rule:** imports only point inward. `domain` depends on nothing in the project.
`application` depends on `domain` only.

## Folder map

```
src/
├── domain/                              # business rules, no framework code
│   ├── shared/                          # Entity, AggregateRoot, DomainEvent, DomainError, publisher port
│   └── task_management/                 # bounded context
│       └── tasks/                       # the Task aggregate
│           ├── task.rs                  # aggregate ROOT entity: create / start / complete / change_description / checklist
│           ├── task_id.rs               # typed ID of the root
│           ├── checklist_item.rs        # ENTITY inside the aggregate (no repository of its own)
│           ├── checklist_item_id.rs     # typed ID of the inner entity
│           ├── task_repository.rs       # repository PORT (trait), one per aggregate root
│           ├── value_objects/           # TaskDescription, TaskStatus (transition table), ChecklistItemTitle
│           └── events/                  # TaskCreated, TaskStarted, TaskCompleted, TaskDescriptionChanged,
│                                        #   ChecklistItemAdded, ChecklistItemChecked
├── application/
│   ├── contracts/ports/                 # UseCase, UnitOfWork, Policy, Logger, queries/ (read side)
│   ├── tasks/                           # one use case per file, grouped by side:
│   │   ├── commands/                    #   WRITE: <UseCase>Input + handler, via Unit of Work → aggregate
│   │   ├── queries/                     #   READ: handlers calling the query ports, no aggregate
│   │   └── policies/                    # When<Event><Reaction>: congratulations, audit log
│   └── policies.rs                      # registry: event FQDN → policies
├── infrastructure/
│   ├── persistence/in_memory/           # store + UoW, repository + mapper, query adapters
│   ├── events/domain_events/            # PolicyDomainEventListener (logs events, runs policies)
│   ├── shared/                          # ConsoleLogger, lock helper
│   └── web_api/                         # routes, one controller per use case, ProblemDetails, health
├── main_modules/                        # composition root (bootstrap)
└── main.rs                              # entry point
tests/                                   # unit_domain_*, unit_application_*, integration_*, e2e_*
docs/                                    # CONCEPTS.md, CODE_FLOW.md
```

## Business rules

| Rule | Enforced in | Error → HTTP |
|---|---|---|
| A description is 1–255 characters after trimming | `TaskDescription::create` | `EmptyTaskDescription` / `TaskDescriptionTooLong` → 400 |
| A new task starts as *todo* | `Task::create` | — |
| Status moves only todo → in progress → done (or todo → done) | `TaskStatus::allowed_next` | `InvalidStatusTransition` → 409 |
| A done task cannot be reworded | `Task::change_description` | `CompletedTaskCannotBeChanged` → 409 |
| A checklist item title is 1–100 characters after trimming | `ChecklistItemTitle::create` | `EmptyChecklistItemTitle` / `ChecklistItemTitleTooLong` → 400 |
| A done task gets no new checklist items | `Task::add_checklist_item` | `CompletedTaskCannotBeChanged` → 409 |
| A task cannot be completed while checklist items are open | `Task::complete` | `TaskHasOpenChecklistItems` → 409 |
| A checklist item is found by its ID, only on its own task | `Task::check_checklist_item` | `ChecklistItemNotFound` → 404 |
| Unknown task | use cases | `TaskNotFound` → 404 |

## Domain events

| Event (FQDN) | Raised by | Payload | Reacted to by (policy) |
|---|---|---|---|
| `TaskApp.TaskManagement.TaskCreated` | `Task::create` | `task_id`, `description` | — (only logged) |
| `TaskApp.TaskManagement.TaskStarted` | `Task::start` | `task_id` | — (only logged) |
| `TaskApp.TaskManagement.TaskCompleted` | `Task::complete` | `task_id` | `WhenTaskCompletedLogCongratulations` |
| `TaskApp.TaskManagement.TaskDescriptionChanged` | `Task::change_description` | `task_id`, `old_description`, `new_description` | `WhenTaskDescriptionChangedWriteAuditLog` |
| `TaskApp.TaskManagement.ChecklistItemAdded` | `Task::add_checklist_item` | `task_id`, `item_id`, `title` | — (only logged) |
| `TaskApp.TaskManagement.ChecklistItemChecked` | `Task::check_checklist_item` | `task_id`, `item_id` | — (only logged) |

The Unit of Work publishes them **only after a successful commit** to the
`PolicyDomainEventListener`. The listener logs each one and runs the matching policies on
background threads. A failing policy is logged and never undoes the change.

### Adding a policy (the recipe)

1. Create `src/application/tasks/policies/when_<event>_<reaction>.rs` that implements `Policy`.
   Copy `when_task_completed_log_congratulations.rs` as a starting point.
2. Export it from `policies/mod.rs`.
3. Add one `registry.entry(<Event>::FQDN)…push(…)` line in `src/application/policies.rs`.
4. Write tests: its reaction, a duplicate event, and the wrong event.

## Tests

`cargo test` runs four suites. Their names follow the source tree:

| File | What it covers |
|---|---|
| `tests/unit_domain_task_management_tasks.rs` | value objects, every status transition (allowed and forbidden), the events of each command |
| `tests/unit_application_tasks_commands.rs` | each command use case: happy path, not found, rule violation, storage failure, using the mocks in `tests/common` |
| `tests/unit_application_tasks_policies.rs` | each policy's log line, a duplicate event, the wrong event, the registry wiring |
| `tests/integration_infrastructure_events.rs` | the listener: logs every event, runs policies in the background, isolates failing and panicking policies |
| `tests/integration_infrastructure_persistence_in_memory.rs` | commit then publish, rollback publishes nothing, corrupt rows, queries |
| `tests/e2e_web_api.rs` | real HTTP requests through the router: full lifecycle (including the policy log lines), 400/404/409 Problem Details, health |

Each test is laid out as `// Arrange`, `// Act`, `// Assert`.

## Differences from the article

| Article | Here | Why |
|---|---|---|
| `Task` with public fields | private fields + command methods | rules can't be bypassed |
| one `TaskService` taking command structs | one command + handler per file | small, focused files with explicit dependencies |
| `Box<dyn Error>` | `DomainError` / `Error` enums | HTTP status chosen by variant, checked by the compiler |
| `expect` while mapping DB rows | mapper returns `Result` | one bad row doesn't crash the server |
| no events, no transactions | domain events + Unit of Work + policies | changes are all-or-nothing; reactions happen after commit, in the background |
| Postgres via SQLx, migrations in `main` | in-memory store | runs with no setup; see below for adding a database |

## Adding a real database later

Add `infrastructure/persistence/postgres/` with the same shape as `in_memory/` (a store or
pool, a Unit of Work, the repository, the mapper, the queries). Then change **only**
`main_modules::bootstrap`. The ports would become `async`. Keep these habits:

- **Connect lazily**, on the first request that needs data, not at start-up.
- **Run migrations as a separate, deliberate step**, not on every start (the article runs them
  in `main`).
- **Keep `/api/health` away from the database.**
- **Give the pool a short idle timeout**, so a pay-per-use or auto-pausing database can go back
  to sleep.

## Deliberately left out (to keep it small)

- **Delete:** `TaskRepository` has no `remove`, and there is no delete use case.
- **Policy timeouts and retries:** std threads can't be cancelled. With tokio, wrap each policy
  in `tokio::time::timeout`; a real message bus would add retries.
- **A policy that changes another aggregate:** both policies only log. A good exercise is a
  `Scoreboard` aggregate plus a `WhenTaskCompletedUpdateScoreboard` policy that opens its own
  Unit of Work.
- **Clock port:** events use `SystemTime::now()` directly.
- **Async ports:** they are not needed while storage is in memory.
