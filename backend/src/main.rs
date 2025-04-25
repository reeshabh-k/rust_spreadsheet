use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

mod basic;
mod input;
mod spreadsheet;

use std::io::Cursor;

type SharedSheet = Arc<Mutex<spreadsheet::SpreadSheet>>;

type FormulaStack = Arc<Mutex<Vec<String>>>;

#[derive(Clone)]
pub struct AppState {
    sheet: SharedSheet,
    formula_stack: FormulaStack,
}

#[derive(Deserialize)]
struct CellParams {
    cell: String,
    val: String,
}
async fn get_value(
    State(state): State<AppState>,
    Json(params): Json<CellParams>,
) -> Json<serde_json::Value> {
    let mut sheet = state.sheet.lock().unwrap();
    let form = format!("{}={}", params.cell, params.val.clone());
    let mut formula_stack = state.formula_stack.lock().unwrap();
    // let cohere_test = state.cohere.lock().unwrap();
    formula_stack.push(form.clone());
    let mut inp = Cursor::new(form);
    let form = input::get_formula(&mut inp);

    let (x, y, z) = sheet.call_formula_api(form.clone());

    Json(json!({ "row": x, "col": y, "val": z }))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        sheet: Arc::new(Mutex::new(spreadsheet::SpreadSheet::new(101, 101))),
        formula_stack: Arc::new(Mutex::new(vec![])),
    };

    // let spreadsheet: SharedSheet = Arc::new(Mutex::new(spreadsheet::SpreadSheet::new(100, 55)));
    // Create CORS middleware
    let cors = CorsLayer::very_permissive(); // or configure more strictly if needed
                                             // let formula_stack: Formula_stack = Arc::new(Mutex::new(vec![]));
                                             // let cohere = ai::CohereChat::new("tyQev2QzfLWJpuhi041QeENIqhuI1rK1caEELTmi");
                                             // Build app with CORS

    // let cors = CorsLayer::new()
    //     .allow_origin(Any)
    //     .allow_methods(Any)
    //     .allow_headers(Any);

    let app = Router::new()
        .route("/api/get_value", post(get_value))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Backend running at http://localhost:8080");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
