use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::application::contracts::ports::UseCase;
use crate::application::tasks::commands::ChangeTaskDescriptionInput;
use crate::infrastructure::web_api::shared::{ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct ChangeTaskDescriptionRequest {
    pub description: String,
}

/// Hexagonal role: DRIVING adapter (primary adapter): turns an HTTP request into a call on
///   the driving port `UseCase` (here: `ChangeTaskDescription`).
///
/// PUT /api/tasks/{id}/description → 204 No Content
pub async fn change_task_description_controller(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<ChangeTaskDescriptionRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .change_task_description
        .execute(ChangeTaskDescriptionInput {
            id,
            description: request.description,
        })?;
    Ok(StatusCode::NO_CONTENT)
}
