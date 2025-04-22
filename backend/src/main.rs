use axum::{
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use std::sync::{Arc, Mutex};
use axum::extract::State;
use axum::extract::Query;
use axum::routing::post;
use serde::Deserialize;

mod spreadsheet;
mod basic; 
mod input;
use std::io::Cursor;


use crate::basic::{Formula, Expression, Cell};

type SharedSheet = Arc<Mutex<spreadsheet::SpreadSheet>>;

#[derive(Deserialize)]
struct CellParams {
    cell: String,
    val: String,
}

async fn hello(State(sheet): State<SharedSheet>) -> Json<serde_json::Value> {
    let mut sheet = sheet.lock().unwrap();
    sheet.val[0] = 10;
    Json(json!({ "message": format!("{}", sheet.val[0]) }))
}

async fn get_value(
        State(sheet): State<SharedSheet>,
        Json(params): Json<CellParams>,
    ) -> Json<serde_json::Value> {
        let mut sheet = sheet.lock().unwrap();


        let form = format!("{}={}", params.cell, params.val.clone());
        let mut inp = Cursor::new(form);
        let form = input::get_formula(&mut inp); 

        let (x, y, z) = sheet.call_formula_api(form.clone());

        
        Json(json!({ "row": x, "col": y, "val": z }))
}

#[tokio::main]
async fn main() {
    let spreadsheet: SharedSheet = Arc::new(Mutex::new(spreadsheet::SpreadSheet::new(100, 55)));
    // Create CORS middleware
    let cors = CorsLayer::very_permissive(); // or configure more strictly if needed

    // Build app with CORS
    let app = Router::new()
        .route("/api/hello", get(hello))
        .route("/api/get_value", post(get_value))
        .with_state(spreadsheet)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Backend running at http://localhost:8080");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
