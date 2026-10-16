use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::application::contracts::ports::UseCase;
use crate::application::tasks::commands::CreateTaskInput;
use crate::infrastructure::web_api::shared::{ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTaskResponse {
    pub id: String,
}

/// POST /api/tasks → 201 `{ "id": "…" }`
///
/// The use cases are synchronous (the in-memory store answers instantly). With a real database
/// the ports would become `async` and this would `.await` them.
pub async fn create_task_controller(
    State(state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<CreateTaskResponse>), ApiError> {
    let id = state.create_task.execute(CreateTaskInput {
        description: request.description,
    })?;
    Ok((
        StatusCode::CREATED,
        Json(CreateTaskResponse { id: id.to_string() }),
    ))
}
