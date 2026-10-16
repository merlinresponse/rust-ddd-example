use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::application::contracts::ports::UseCase;
use crate::application::tasks::commands::CheckChecklistItemInput;
use crate::infrastructure::web_api::shared::{ApiError, AppState};

/// POST /api/tasks/{id}/checklist/{item_id}/check → 204 No Content
pub async fn check_checklist_item_controller(
    State(state): State<AppState>,
    Path((task_id, item_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    state
        .check_checklist_item
        .execute(CheckChecklistItemInput { task_id, item_id })?;
    Ok(StatusCode::NO_CONTENT)
}
