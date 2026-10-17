use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::application::contracts::ports::UseCase;
use crate::application::tasks::commands::StartTaskInput;
use crate::infrastructure::web_api::shared::{ApiError, AppState};

/// Hexagonal role: DRIVING adapter (primary adapter): turns an HTTP request into a call on
///   the driving port `UseCase` (here: `StartTask`).
///
/// POST /api/tasks/{id}/start → 204 No Content
pub async fn start_task_controller(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.start_task.execute(StartTaskInput { id })?;
    Ok(StatusCode::NO_CONTENT)
}
