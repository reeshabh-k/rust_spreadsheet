//! # Spreadsheet API Server
//!
//! This crate provides a simple HTTP API for evaluating and retrieving values from a spreadsheet.
//! It uses the Axum web framework and supports CORS. The backend maintains shared state
//! including a spreadsheet and a stack of evaluated formulas.

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

/// A thread-safe shared spreadsheet instance.
type SharedSheet = Arc<Mutex<spreadsheet::SpreadSheet>>;

/// A thread-safe stack of evaluated formulas for bookkeeping or history.
type FormulaStack = Arc<Mutex<Vec<String>>>;

/// Shared application state injected into route handlers.
#[derive(Clone)]
pub struct AppState {
    sheet: SharedSheet,
    formula_stack: FormulaStack,
}

/// Request payload for evaluating a cell.
///
/// The client sends a `cell` identifier and a formula or value `val` to be assigned or evaluated.
#[derive(Deserialize)]
struct CellParams {
    cell: String,
    val: String,
}

/// Route handler for evaluating a formula and returning its resolved value.
///
/// This function:
/// - Parses and stores the formula to a stack for record keeping.
/// - Passes the formula to the spreadsheet engine.
/// - Returns the resolved row, column, and value.
///
/// # Arguments
///
/// * `State(state)` - Shared application state.
/// * `Json(params)` - JSON body with cell and formula information.
///
/// # Returns
///
/// JSON object containing:
/// - `"row"`: Row index of the evaluated cell.
/// - `"col"`: Column index of the evaluated cell.
/// - `"val"`: Resolved value of the formula.
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

/// Launches the Axum web server with the `/api/get_value` endpoint.
///
/// The server initializes:
/// - A shared spreadsheet instance with dimensions 101x101.
/// - A stack for storing formula history.
/// - Very permissive CORS settings (suitable for development only).
///
/// The endpoint is available at:
/// `POST http://localhost:8080/api/get_value`
///
/// # Panics
///
/// Panics if the TCP listener fails to bind or the Axum server fails to run.
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
