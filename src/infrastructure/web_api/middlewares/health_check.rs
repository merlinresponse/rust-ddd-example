use axum::Json;
use serde_json::{json, Value};

/// Hexagonal role: DRIVING adapter that deliberately calls NO port: it only reports that this
///   process is up.
///
/// Liveness check: reports that THIS process is up, nothing more.
///
/// It deliberately never touches storage: with a real (possibly auto-pausing, pay-per-use)
/// database, a health probe that queries it would wake and bill it on every poll.
pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
