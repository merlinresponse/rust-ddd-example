//! End-to-end tests: real HTTP requests through the full router, use cases and in-memory store.
#![allow(non_snake_case)]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use common::RecordingLogger;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use task_manager::domain::task_management::tasks::events::{
    TaskCompleted, TaskCreated, TaskDescriptionChanged, TaskStarted,
};
use task_manager::infrastructure::web_api::routes;
use task_manager::main_modules::bootstrap_with;
use tower::ServiceExt;

async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn web_api__health__is_ok() {
    // Arrange
    let app = routes(bootstrap_with(RecordingLogger::new()).state);
    // Act
    let (status, body) = send(&app, "GET", "/api/health", None).await;
    // Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "status": "ok" }));
}

#[tokio::test]
async fn web_api__full_task_lifecycle__works_and_publishes_every_event() {
    // Arrange
    let logger = RecordingLogger::new();
    let container = bootstrap_with(logger.clone());
    let app = routes(container.state);

    // Act + Assert: create
    let (status, body) = send(
        &app,
        "POST",
        "/api/tasks",
        Some(json!({ "description": "Buy milk" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = body["id"].as_str().unwrap().to_string();

    // reword, start, complete
    let (status, _) = send(
        &app,
        "PUT",
        &format!("/api/tasks/{id}/description"),
        Some(json!({ "description": "Buy oat milk" })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        send(&app, "POST", &format!("/api/tasks/{id}/start"), None)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        send(&app, "POST", &format!("/api/tasks/{id}/complete"), None)
            .await
            .0,
        StatusCode::NO_CONTENT
    );

    // read back
    let (status, task) = send(&app, "GET", &format!("/api/tasks/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        task,
        json!({ "id": id, "description": "Buy oat milk", "status": "done", "checklist": [] })
    );
    let (_, list) = send(&app, "GET", "/api/tasks", None).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Policies run in the background: wait for them before checking their log lines.
    container.listener.when_idle();
    for fqdn in [
        TaskCreated::FQDN,
        TaskDescriptionChanged::FQDN,
        TaskStarted::FQDN,
        TaskCompleted::FQDN,
    ] {
        assert!(
            logger.contains(&format!("[event] {fqdn}")),
            "{fqdn} not logged"
        );
    }
    assert!(logger.contains(&format!(
        "[audit] task {id} reworded from \"Buy milk\" to \"Buy oat milk\""
    )));
    assert!(logger.contains(&format!("Well done! Task {id} is completed.")));
}

#[tokio::test]
async fn web_api__completing_twice__returns_409_problem_details() {
    // Arrange
    let app = routes(bootstrap_with(RecordingLogger::new()).state);
    let (_, body) = send(
        &app,
        "POST",
        "/api/tasks",
        Some(json!({ "description": "Buy milk" })),
    )
    .await;
    let id = body["id"].as_str().unwrap();
    send(&app, "POST", &format!("/api/tasks/{id}/complete"), None).await;
    // Act
    let (status, problem) = send(&app, "POST", &format!("/api/tasks/{id}/complete"), None).await;
    // Assert
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(problem["status"], 409);
    assert_eq!(
        problem["detail"],
        "A task cannot move from \"done\" to \"done\"."
    );
}

#[tokio::test]
async fn web_api__unknown_task__returns_404_problem_details() {
    // Arrange
    let app = routes(bootstrap_with(RecordingLogger::new()).state);
    // Act
    let (status, problem) = send(
        &app,
        "GET",
        "/api/tasks/00000000-0000-4000-8000-000000000000",
        None,
    )
    .await;
    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(problem["title"], "Task not found");
}

#[tokio::test]
async fn web_api__empty_description__returns_400_problem_details() {
    // Arrange
    let app = routes(bootstrap_with(RecordingLogger::new()).state);
    // Act
    let (status, problem) = send(
        &app,
        "POST",
        "/api/tasks",
        Some(json!({ "description": "" })),
    )
    .await;
    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(problem["detail"], "A task description cannot be empty.");
}

#[tokio::test]
async fn web_api__checklist__blocks_completion_until_every_item_is_checked() {
    // Arrange
    let app = routes(bootstrap_with(RecordingLogger::new()).state);
    let (_, body) = send(
        &app,
        "POST",
        "/api/tasks",
        Some(json!({ "description": "Bake a cake" })),
    )
    .await;
    let id = body["id"].as_str().unwrap().to_string();
    let (status, item) = send(
        &app,
        "POST",
        &format!("/api/tasks/{id}/checklist"),
        Some(json!({ "title": "Buy eggs" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let item_id = item["id"].as_str().unwrap().to_string();

    // Act + Assert: completing with an open item is refused
    let (status, problem) = send(&app, "POST", &format!("/api/tasks/{id}/complete"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        problem["detail"],
        "A task cannot be completed while 1 checklist item(s) are still open."
    );

    // tick it off, then completing works
    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/tasks/{id}/checklist/{item_id}/check"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        send(&app, "POST", &format!("/api/tasks/{id}/complete"), None)
            .await
            .0,
        StatusCode::NO_CONTENT
    );

    let (_, task) = send(&app, "GET", &format!("/api/tasks/{id}"), None).await;
    assert_eq!(task["status"], "done");
    assert_eq!(
        task["checklist"],
        json!([{ "id": item_id, "title": "Buy eggs", "checked": true }])
    );
}
