# Rust Spreadsheet with Extra Functionality
1) Reeshabh Rajesh Kotecha
2) Vijay Balaji Narasimma Bharathi
3) Vivswan Savyasachi

## Installation Requirements

### Prerequisites
1. Install Rust and Cargo: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2. Install wasm-pack: `cargo install wasm-pack`
3. Install Trunk (for building and serving): `cargo install trunk`

### Frontend Dependencies
The frontend requires the following crates (these are automatically installed by Cargo):
- `yew`: Frontend framework for creating web applications in Rust
- `web-sys`: Bindings for web APIs
- `wasm-bindgen`: For JavaScript interoperability
- `wasm-bindgen-futures`: For async JavaScript interop
- `js-sys`: JavaScript standard library bindings
- `serde` and `serde_json`: For serialization/deserialization
- `reqwest`: HTTP client for making API requests
- `regex`: For pattern matching in formulas
- `once_cell`: For lazy initialization
- `ordered-float`: For floating point comparison


## Features
- Cell editing and formula support
- Navigation via keyboard shortcuts
- Data visualization with multiple chart types including bar graphs, line charts, pie chart(for frequency) and heat map
- Statistical analysis for a budget sheet with first row as income and second row as expense with the columns being various sources of income and expense.
- Import/export functionality for CSV and JSON formats
- AI-powered formula suggestions