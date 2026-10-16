use axum::extract::{Path, State};
use axum::Json;

use crate::application::contracts::ports::UseCase;
use crate::application::tasks::queries::FindTaskByIdInput;
use crate::infrastructure::web_api::shared::{ApiError, AppState, TaskResponse};

/// GET /api/tasks/{id} → 200 task
pub async fn find_task_by_id_controller(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TaskResponse>, ApiError> {
    let view = state.find_task_by_id.execute(FindTaskByIdInput { id })?;
    Ok(Json(view.into()))
}
