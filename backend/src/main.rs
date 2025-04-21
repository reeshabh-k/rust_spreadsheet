use axum::{
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

async fn hello() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello vivswan from backend!" }))
}

#[tokio::main]
async fn main() {
    // Create CORS middleware
    let cors = CorsLayer::very_permissive(); // or configure more strictly if needed

    // Build app with CORS
    let app = Router::new()
        .route("/api/hello", get(hello))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Backend running at http://localhost:8080");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
