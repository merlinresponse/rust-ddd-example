use axum::routing::{get, post, put};
use axum::Router;

use super::controllers::{
    add_checklist_item_controller, change_task_description_controller,
    check_checklist_item_controller, complete_task_controller, create_task_controller,
    find_task_by_id_controller, list_tasks_controller, start_task_controller,
};
use super::middlewares::health_check;
use super::shared::AppState;

/// All HTTP routes. See `config/openapi.yml` for the contract.
///
/// FLOW 1 — WHAT: axum matches the method + path of a request against this table and calls
/// the controller. `with_state` gives every controller the same `AppState` (the use cases).
/// WHY one table: you can see the whole HTTP surface of the app at a glance.
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route(
            "/api/tasks",
            post(create_task_controller).get(list_tasks_controller),
        )
        .route("/api/tasks/{id}", get(find_task_by_id_controller))
        .route(
            "/api/tasks/{id}/description",
            put(change_task_description_controller),
        )
        .route("/api/tasks/{id}/start", post(start_task_controller))
        .route("/api/tasks/{id}/complete", post(complete_task_controller))
        .route(
            "/api/tasks/{id}/checklist",
            post(add_checklist_item_controller),
        )
        .route(
            "/api/tasks/{id}/checklist/{item_id}/check",
            post(check_checklist_item_controller),
        )
        .with_state(state)
}
