use axum::extract::State;
use axum::Json;

use crate::application::contracts::ports::UseCase;
use crate::infrastructure::web_api::shared::{ApiError, AppState, TaskResponse};

/// GET /api/tasks → 200 list of tasks
pub async fn list_tasks_controller(
    State(state): State<AppState>,
) -> Result<Json<Vec<TaskResponse>>, ApiError> {
    let views = state.list_tasks.execute(())?;
    Ok(Json(views.into_iter().map(TaskResponse::from).collect()))
}
