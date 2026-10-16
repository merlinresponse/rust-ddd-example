//! Entry point: build the app (composition root), then serve HTTP.

use task_manager::infrastructure::web_api::routes;
use task_manager::main_modules::bootstrap;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let address = format!("0.0.0.0:{port}");

    // FLOW 0 (once, at start-up) — WHAT: build every object and wire them together.
    // WHY in one place: only `main_modules` knows the concrete types; every other file depends
    // on traits, so swapping storage means changing one function. Nothing is loaded yet.
    // Walkthrough of a request: docs/CODE_FLOW.md
    let app = routes(bootstrap().state);

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap_or_else(|e| panic!("cannot listen on {address}: {e}"));
    println!("Task Manager listening on http://{address}");
    axum::serve(listener, app)
        .await
        .expect("server stopped unexpectedly");
}
