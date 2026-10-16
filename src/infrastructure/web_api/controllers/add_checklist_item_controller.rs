use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::application::contracts::ports::UseCase;
use crate::application::tasks::commands::AddChecklistItemInput;
use crate::infrastructure::web_api::shared::{ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct AddChecklistItemRequest {
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct AddChecklistItemResponse {
    pub id: String,
}

/// POST /api/tasks/{id}/checklist → 201 `{ "id": "<checklist item id>" }`
pub async fn add_checklist_item_controller(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(request): Json<AddChecklistItemRequest>,
) -> Result<(StatusCode, Json<AddChecklistItemResponse>), ApiError> {
    let id = state.add_checklist_item.execute(AddChecklistItemInput {
        task_id,
        title: request.title,
    })?;
    Ok((
        StatusCode::CREATED,
        Json(AddChecklistItemResponse { id: id.to_string() }),
    ))
}
