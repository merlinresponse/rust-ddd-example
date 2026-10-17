# Concepts: what each piece is, why it exists, and where to find it

Read this next to the code. Each section answers three questions: **What is it? Why do we do
it? Where is it?** Then it names the pattern, so you can look it up later.

## 1. Layers and the dependency rule

```
┌──────────────────────────────────────────────────────────────┐
│ main_modules        builds and wires everything               │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ infrastructure   HTTP, storage, event output           │   │
│  │  ┌──────────────────────────────────────────────────┐  │   │
│  │  │ application   use cases, ports (UoW, queries)    │  │   │
│  │  │  ┌────────────────────────────────────────────┐  │  │   │
│  │  │  │ domain   Task, rules, events, repo port    │  │  │   │
│  │  │  └────────────────────────────────────────────┘  │  │   │
│  │  └──────────────────────────────────────────────────┘  │   │
│  └────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
          imports only point INWARD  →  domain imports nothing
```

**What:** four layers. A file may only `use` code from its own layer or from layers *inside* it.
You can check this yourself: `grep -rn "infrastructure\|axum" src/domain` finds nothing.
**Why:** the business rules (the part that is hard to get right) never change because you
switched web framework or database. They can be tested without either.
**Pattern:** Layered Architecture (Evans Ch. 4), Clean / Hexagonal Architecture.

## 1b. Ports and adapters: DRIVING vs DRIVEN

The layers above can also be drawn as a hexagon: the **application core** (domain +
application) sits in the middle, and the outside world talks to it only through **ports**,
which are traits. **Adapters** are the technical code that plugs into those ports. The key
question for every port is: *who starts the conversation?*

```
       DRIVING side (left)                              DRIVEN side (right)
  "the world uses the app"                          "the app uses the world"

  ┌───────────────────────┐                        ┌──────────────────────────┐
  │ DRIVING ADAPTERS      │                        │ DRIVEN ADAPTERS          │
  │ HTTP controllers      │                        │ InMemoryTaskRepository   │
  │ tests (E2E, unit)     │                        │ InMemoryUnitOfWork       │
  │ PolicyDomainEvent-    │                        │ InMemory…Query           │
  │   Listener (policies) │                        │ PolicyDomainEvent-       │
  └──────────┬────────────┘                        │   Listener (publishing)  │
             │ calls                               │ ConsoleLogger            │
             ▼                                     └────────────▲─────────────┘
  ╔═══════════════════════╗                                     │ implements
  ║ DRIVING PORTS         ║   ┌─────────────────────┐   ╔═══════╧══════════════╗
  ║ UseCase<I, O>         ║──▶│  APPLICATION CORE   │──▶║ DRIVEN PORTS         ║
  ║ Policy                ║   │  use cases, Task,   │   ║ TaskRepository       ║
  ╚═══════════════════════╝   │  rules, events      │   ║ UnitOfWork           ║
   implemented by the core    └─────────────────────┘   ║ FindTaskByIdQuery    ║
                                                        ║ ListTasksQuery       ║
                                                        ║ DomainEventPublisher ║
                                                        ║ Logger               ║
                                                        ╚══════════════════════╝
                                                         defined by the core,
                                                         implemented outside
```

### Driving ports (primary): the world → the app
**What:** the entry points of the application, i.e. "what can you ask this app to do?" The
**core implements** them, and something outside **calls** them.

| Port | Where | Implemented by (core) | Called by (driving adapter) |
|---|---|---|---|
| `UseCase<I, O>` | `application/contracts/ports/use_case.rs` | every command and query in `application/tasks/` | HTTP controllers; tests |
| `Policy` | `application/contracts/ports/policy.rs` | the policies in `application/tasks/policies/` | `PolicyDomainEventListener` |

### Driven ports (secondary): the app → the world
**What:** what the application **needs** from outside, i.e. "store this", "find that",
"tell others", "write a log line". The **core defines** them and **calls** them, and an
adapter outside **implements** them.

| Port | Where | Implemented by (driven adapter) | Test double |
|---|---|---|---|
| `TaskRepository` | `domain/task_management/tasks/task_repository.rs` | `InMemoryTaskRepository` | `MockTaskRepository` |
| `DomainEventPublisher` | `domain/shared/domain_event_publisher.rs` | `PolicyDomainEventListener` | `RecordingPublisher` |
| `UnitOfWork` | `application/contracts/ports/unit_of_work.rs` | `InMemoryUnitOfWork` | `MockUnitOfWork` |
| `FindTaskByIdQuery` | `application/contracts/ports/queries/` | `InMemoryFindTaskByIdQuery` | — |
| `ListTasksQuery` | `application/contracts/ports/queries/` | `InMemoryListTasksQuery` | — |
| `Logger` | `application/contracts/ports/logger.rs` | `ConsoleLogger` | `RecordingLogger` |

**Why some driven ports live in `domain/` and others in `application/`:** a port lives with
the code that *needs* it. The domain itself talks about "a repository of tasks" and "publishing
an event", so those two are domain ports. Transactions, logging and read queries are
application concerns.

### Adapters: the technical code at the edges
| Adapter | Side | Plugs into |
|---|---|---|
| `web_api/controllers/*_controller.rs`, plus routes, `ApiError` and DTOs | **driving** | calls `UseCase` |
| `web_api/middlewares/health_check.rs` | driving | calls *no* port on purpose: it only says "this process is up" |
| `persistence/in_memory/repositories/in_memory_task_repository.rs` (+ mapper, store) | **driven** | implements `TaskRepository` |
| `persistence/in_memory/shared/in_memory_unit_of_work.rs` | driven | implements `UnitOfWork` |
| `persistence/in_memory/queries/*` | driven | implements the query ports |
| `shared/console_logger.rs` | driven | implements `Logger` |
| `events/domain_events/policy_domain_event_listener.rs` | **both** | implements `DomainEventPublisher` (driven) **and** calls `Policy` (driving) |

**The listener is the interesting one.** Seen from the Unit of Work, it is a *driven* adapter:
the application calls `publish`. Seen from the policies, it is a *driving* adapter: it calls
`policy.handle`, just like a controller calls a use case. Direction is about **who calls
whom**, not about which folder a file sits in.

**How to tell them apart in one sentence:** if the *outside* calls a trait and the *app*
implements it, it's a **driving** port. If the *app* calls a trait and the *outside*
implements it, it's a **driven** port.

**Why it matters:**
- **Tests:** swap driven adapters for test doubles (`tests/common`) to test use cases without
  storage, and drive the app through its driving ports without HTTP.
- **Technology changes:** a Postgres repository is just another driven adapter for
  `TaskRepository`. A CLI or a message-queue consumer is just another driving adapter
  calling the same `UseCase`s.
- **The core never names an adapter:** only `main_modules` (the composition root) knows which
  adapter goes into which port.

Every port and adapter in the code says which it is in a `Hexagonal role:` doc line
(`grep -rn "Hexagonal role" src`).

**Pattern:** Ports and Adapters / Hexagonal Architecture (Alistair Cockburn, 2005; beyond
Evans).

## 2. Value Object — `TaskDescription`, `TaskStatus`, `TaskId`
**What:** a small immutable type defined by its value. `TaskDescription` has a private field and
a single way in, `create()`, which validates the input.
**Why:** an invalid description *cannot exist*. Every function that receives a
`TaskDescription` can trust it, so validation is not repeated (or forgotten) all over the code.
**Rust trick:** a private tuple field `TaskDescription(String)` plus a constructor that returns
`Result`. This is called the "newtype" pattern.
**Pattern:** Value Object (Evans Ch. 5).

## 3. Entity and Aggregate Root — `Task`
**What:** something with an **identity** that lasts over time (`TaskId`), and the single gate
through which every change goes. It has command methods (`start`, `complete`,
`change_description`) and no setters.
**Why:** the rules ("done is final", "a done task cannot be reworded") live in exactly one
place. You can't write `task.status = Done`, because the field is private.
**Two constructors:**

| | `Task::create` | `Task::reconstitute` |
|---|---|---|
| used for | a brand-new task | loading an existing task |
| new ID? | yes | no, it uses the stored ID |
| raises an event? | yes, `TaskCreated` | no, nothing new happened |

**Pattern:** Entity (Evans Ch. 5), Aggregate (Evans Ch. 6).

## 3b. Entity inside the aggregate — `ChecklistItem`, next to the value object `ChecklistItemTitle`
**What:** a task can have a checklist ("Bake a cake": ☐ Buy eggs, ☐ Buy flour). Each
`ChecklistItem` is an **entity** with its own `ChecklistItemId`. Its text is a
`ChecklistItemTitle`, which is a **value object**. Both live inside the Task aggregate:

```
Task  (aggregate ROOT entity, TaskId)
├── description: TaskDescription          value object
├── status:      TaskStatus               value object
└── checklist:   Vec<ChecklistItem>       inner ENTITIES
                 ├── id:      ChecklistItemId      identity
                 ├── title:   ChecklistItemTitle   value object
                 └── checked: bool
```

**Entity vs value object, side by side:**

| | `ChecklistItem` (entity) | `ChecklistItemTitle` (value object) |
|---|---|---|
| defined by | its identity, `ChecklistItemId` | its value, the text |
| two with the same text are… | **different** (two "Call Bob" items) | **the same** (`==` is true) |
| compared with | `same_identity_as` (the `Entity` trait) | `==` (`#[derive(PartialEq)]`) |
| can change? | yes: unchecked → checked, and it is still the same item | never; to change it, you replace it |
| derives `PartialEq`? | deliberately **not** | yes |

**Why an entity:** the user says "tick off *this* item". Titles may repeat, so only an
identity can point at one item. The item also changes over time and must stay "the same item"
after it is stored and loaded again.

**Why not its own aggregate:** an item makes no sense without its task. The new rule *"a task
cannot be completed while checklist items are still open"* needs the task and **all** its items
to be consistent at the same moment. So they are loaded and saved together, as one aggregate.
That is also why there is **no `ChecklistItemRepository`**: the use cases load the task and ask
it to change its item (`task.check_checklist_item(&id)`).

**How Rust enforces "only through the root":** `ChecklistItem::new` and `ChecklistItem::check`
are `pub(super)`, so only code in the `tasks` module (that is, `Task`) can call them.
`Task::checklist()` gives everyone else a read-only `&[ChecklistItem]`.

**Pattern:** Entity (Evans Ch. 5), Aggregate boundaries (Evans Ch. 6).

## 4. State machine — `TaskStatus::allowed_next`
**What:** one `match` lists every legal move: `Todo → InProgress | Done`, `InProgress → Done`,
and `Done` is final.
**Why:** the whole rulebook is visible in a few lines. If you add a status, the compiler makes
you decide its transitions.

## 5. Domain Events — `events/` folder
**What:** past-tense facts (`TaskCompleted`). The aggregate *records* them in its private
`DomainEvents` list, and the Unit of Work *publishes* them after commit.
**Why:** other features ("e-mail the owner when a task is done") can react without touching
`Task`. And because events are published after commit, nobody reacts to a change that was
rolled back.
**Pattern:** Domain Events (added to DDD after Evans' book).

## 6. Repository (port) — `TaskRepository`
**What:** a trait in the **domain** with `by_id` and `save`. It behaves like a collection of
tasks.
**Why:** the domain says *what* it needs. *How* it is stored (HashMap, Postgres) is an
infrastructure detail, in `InMemoryTaskRepository`.
**Rule:** a repository never commits and never publishes events.
**Pattern:** Repository (Evans Ch. 6), Ports and Adapters.

## 7. Mapper — `task_record_mapper.rs`
**What:** `to_record` turns a `Task` into a row; `reconstitute` turns a row back into a `Task`.
**Why:** the stored shape (strings) and the domain shape (types with rules) evolve separately.
Bad stored data becomes an error, not a panic.

## 8. Use Case — `application/tasks/commands/*.rs` and `application/tasks/queries/*.rs`
**What:** one small struct per user intention (`CompleteTask`), with one method, `execute`.
**Why:** each file reads like a recipe: *parse → load → one domain call → save*. Its
dependencies are explicit in `new(...)`. The article this project is based on uses one
`TaskService` with every method instead. That works, but it grows into a grab bag.
**Rule:** a use case holds **no business rules**. It only coordinates.
**Pattern:** Application Service (Evans Ch. 4), Command pattern. See section 10 for commands vs
queries.

## 9. Unit of Work — `UnitOfWork` port, `InMemoryUnitOfWork` adapter
**What:** wraps a use case in one all-or-nothing transaction: BEGIN → your closure → COMMIT or
ROLLBACK → publish events.
**Why:** transactions and event publishing are easy to get subtly wrong. Doing them once,
centrally, means no use case can forget them.
**Rule:** one aggregate changed per Unit of Work.
**The guarantee, in two sentences:** all changes of a command are saved together or not at
all; and an event is published only *after* its change was committed, never for a change that
was rolled back. The step-by-step walkthrough (BEGIN → rules → save → COMMIT/ROLLBACK →
PUBLISH/DROP), the three outcomes side by side, and the one remaining gap (a crash right after
commit, solved by a transactional outbox) are in `docs/CODE_FLOW.md`, section "All or
nothing".
**Rust note:** `run(&self, action: &mut dyn FnMut(&dyn UnitOfWork) -> Result<()>)` uses trait
objects (`dyn`) instead of generics, so it can be shared as `Arc<dyn UnitOfWork>`.
**Pattern:** Unit of Work (Fowler, PoEAA).

## 9b. Policy — `application/tasks/policies/`, registry in `application/policies.rs`
**What:** a reaction to an event, "WHEN `TaskCompleted` THEN log congratulations". It is a
struct implementing `Policy { name(), handle(&dyn DomainEvent) }`. The registry maps each event
name to its policies. The `PolicyDomainEventListener` (infrastructure) runs them on background
threads after commit.
**Why:** the aggregate does its own job, completing the task, and knows nothing about who
reacts. New reactions are added without touching `Task` or any use case. A broken reaction
cannot break or undo the change that triggered it.
**Rules:** named `When<Event><Reaction>`; safe to run twice; it changes at most one other
aggregate, in its own Unit of Work; failures are logged, never re-thrown.
**Rust notes:**
- `event.downcast_ref::<TaskCompleted>()` (through `as_any`) gets the concrete event back from a
  `dyn DomainEvent`.
- `Arc<dyn DomainEvent>` shares one event across policy threads.
- `catch_unwind` stops a panic from crossing the thread.

**Pattern:** Policy / event handler (Domain Events; vocabulary from Event Storming).
The walkthrough is in `docs/CODE_FLOW.md#after-commit-policies`.

## 10. Commands and queries (CQRS-lite)
**What:** every use case is one of two kinds:

| | **Command** side (write) | **Query** side (read) |
|---|---|---|
| asks the system to… | *change* something | *tell* something |
| folder | `application/tasks/commands/` | `application/tasks/queries/` (handlers) + `application/contracts/ports/queries/` (ports) |
| input type | `CompleteTaskInput`, `AddChecklistItemInput`, … | `FindTaskByIdInput`, `()` |
| handler | `CompleteTask`, `AddChecklistItem`, … | `FindTaskById`, `ListTasks` |
| goes through | Unit of Work → repository → **aggregate** | query port → plain rows |
| returns | nothing, or the new ID | data (`TaskView`) |
| can be refused by a business rule? | **yes** (`TaskHasOpenChecklistItems`, …) | no, only "not found" |
| raises events? | yes, published after commit | never |
| HTTP | `POST` / `PUT` | `GET` |

**A command vs an event:** both are messages, but a **command** is an imperative request that
may still be refused (`CompleteTaskInput`: "please complete it"). An **event** is a
past-tense fact that has already happened (`TaskCompleted`: "it is completed").

```
CompleteTaskInput ──▶ CompleteTask ──▶ Task::complete() ──▶ TaskCompleted ──▶ policies
   (request)            (handler)        (rule: may refuse)    (fact)          (reactions)
```

**Where you can see it:** the **folder** tells you the side. Everything in
`application/tasks/commands/` changes state; everything in `application/tasks/queries/` only
reads. The data types keep the course's `<UseCase>Input` naming (`CompleteTaskInput`). The
article calls the same thing `CompleteTaskCommand`; it is the same idea under a different name.
The imports show the side too: `use crate::application::tasks::commands::CompleteTaskInput;`.

**Why separate them:** the write side needs strict rules, transactions and events. The read
side needs none of that, so it reads rows directly and stays simple and fast. Splitting them
keeps each side clean. "CQRS-lite" means we split the *code* paths but keep **one** store; full
CQRS would also keep a separate read database.

## 11. Typed errors — `DomainError`, `Error`
**What:** an `enum` with one variant per broken rule, plus `Error::Infrastructure` for technical
failures. `?` converts between them through `impl From<DomainError> for Error`.
**Why:** the HTTP layer `match`es on the variant to choose 400, 404 or 409. The article uses
`Box<dyn Error>`, which loses that information.

## 12. Composition root — `main_modules::bootstrap`
**What:** the one function that creates concrete objects and passes them in as `Arc<dyn Trait>`.
**Why:** everything else depends on traits, so tests can pass mocks (`tests/common`) and
production can pass real adapters.
**Pattern:** Dependency Injection / Composition Root.

## 13. Health check without storage — `middlewares/health_check.rs`
**What:** `GET /api/health` returns `{"status":"ok"}` and touches nothing else.
**Why:** health probes run every few seconds. If they queried a pay-per-use database that
pauses when idle, every probe would wake it up and bill you. "The process is up" is all a
liveness check should say.

## Rust features you will meet

| Feature | Where | Why it's used |
|---|---|---|
| `Result<T, E>` and `?` | everywhere | errors are values; `?` returns early on error |
| `Option<T>` | `by_id` | "maybe there is no such task", with no null |
| `let … else` | use cases | load the task or return `TaskNotFound` early |
| `trait` + `dyn Trait` | ports | an interface whose implementation is picked at runtime |
| `Arc<T>` | composition root | shared ownership across threads (axum handles requests on many threads) |
| `Mutex<T>` | in-memory store | only one thread changes the HashMap at a time |
| `impl From<A> for B` | `Error`, `ApiError` | lets `?` convert error types automatically |
| private fields + `pub fn` | value objects, `Task` | the compiler enforces "no setters" |
| `pub(super)` | `ChecklistItem::new`, `check` | only the parent module (the aggregate root) may create or change an inner entity |
| `std::mem::take` / `replace` | UoW, `Task` | move a value out and leave a valid one behind |
| `Any` + `downcast_ref` | policies | get the concrete event type back from `dyn DomainEvent` |
| `thread::spawn` + `JoinHandle` | policy listener | run reactions in the background; `when_idle` joins them in tests |
| `panic::catch_unwind` | policy listener | a panicking policy is logged instead of crashing the server |
