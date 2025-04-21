use axum::{
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use std::sync::{Arc, Mutex};
use axum::extract::State;

mod spreadsheet;
mod basic; 
mod input;

type SharedSheet = Arc<Mutex<spreadsheet::SpreadSheet>>;

async fn hello(State(sheet): State<SharedSheet>) -> Json<serde_json::Value> {
    let mut sheet = sheet.lock().unwrap();
    sheet.val[0] = 10;
    Json(json!({ "message": format!("{}", sheet.val[0]) }))
}

async fn get_value(State(sheet): State<SharedSheet>) -> Json<serde_json::Value> {
    let mut sheet = sheet.lock().unwrap();
    sheet.val[0] = 9090909;
    Json(json!({ "value": format!("{}", sheet.val[0]) }))
}

#[tokio::main]
async fn main() {
    let spreadsheet: SharedSheet = Arc::new(Mutex::new(spreadsheet::SpreadSheet::new(100, 100)));
    // Create CORS middleware
    let cors = CorsLayer::very_permissive(); // or configure more strictly if needed

    // Build app with CORS
    let app = Router::new()
        .route("/api/hello", get(hello))
        .route("/api/get_value", get(get_value))
        .with_state(spreadsheet.clone())
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Backend running at http://localhost:8080");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
