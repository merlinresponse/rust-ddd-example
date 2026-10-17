use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::application::contracts::ports::UseCase;
use crate::application::tasks::commands::CompleteTaskInput;
use crate::infrastructure::web_api::shared::{ApiError, AppState};

/// Hexagonal role: DRIVING adapter (primary adapter): turns an HTTP request into a call on
///   the driving port `UseCase` (here: `CompleteTask`).
///
/// POST /api/tasks/{id}/complete → 204 No Content
pub async fn complete_task_controller(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // FLOW 2 — WHAT: translate HTTP → use-case input. `Path(id)` pulled the id out of the URL.
    // WHY so thin: no rules here, so the same use case could also be called from a CLI or a test.
    // The `?` converts any error into `ApiError`, rendered in FLOW 9b.
    state.complete_task.execute(CompleteTaskInput { id })?;
    // FLOW 9a: success → translate back to HTTP.
    Ok(StatusCode::NO_CONTENT)
}
