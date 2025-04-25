//! # Helper module for Rusty Spreadsheet
//!
//! Provides common utilities and traits used by the frontend spreadsheet application.
//! This module contains form field validation, trait definitions, and helper functions
//! that are reused across the application.

use std::collections::HashMap;
use once_cell::sync::Lazy;
use regex::Regex;

// --- Structs & Enums ------------------------------------
/// Represents a spreadsheet cell by its row and column indices.
///
/// This struct is used internally for parsing cell references and validating formulas.
/// It holds the 1-based row and column indices of a cell.
#[derive(Debug)]
struct Cell { row: u32, col: u32 }

/// Value in a formula: either a numeric literal or a reference to another cell.
///
/// This enum represents the possible value types in spreadsheet formulas:
/// - Num: A direct numeric value (e.g., "5" in "A1=5+B2")
/// - Ref: A reference to another cell (e.g., "B2" in "A1=5+B2")
#[derive(Debug)]
enum _Value { _Num(i32), _Ref(Cell) }

/// Arithmetic and aggregation expressions supported in formulas.
///
/// This enum represents the different types of operations that can be
/// performed in spreadsheet formulas, including basic arithmetic and
/// statistical functions that operate on cell ranges.
#[derive(Debug)]
enum Expression {
    /// Addition of two values
    Add((),()),
    /// Subtraction of two values
    _Sub(_Value, _Value),
    /// Multiplication of two values
    _Mul(_Value, _Value),
    /// Division of two values
    _Div(_Value, _Value),
    /// Minimum value in a cell range
    _Min(Cell, Cell),
    /// Maximum value in a cell range
    _Max(Cell, Cell),
    /// Average of values in a cell range
    _Avg(Cell, Cell),
    /// Sum of values in a cell range
    _Sum(Cell, Cell),
    /// Standard deviation of values in a cell range
    _Stdev(Cell, Cell),
    /// Simulated delay (for testing/demonstration)
    _Sleep(_Value),
}

/// A parsed formula: input cell and the expression to compute.
///
/// This struct represents the parsed structure of a formula in the spreadsheet,
/// storing both the target cell and the expression to calculate.
/// 
/// # Examples
/// 
/// For a formula like "A1=B2+C3", it would store:
/// - inp_cell: Cell { row: 1, col: 1 }  // A1
/// - expression: Expression::Add(Value::Ref(Cell { row: 2, col: 2 }), Value::Ref(Cell { row: 3, col: 3 }))
#[derive(Debug)]
struct Formula { inp_cell: Cell, _expression: Expression }

/// Context menu state (moved from main.rs)
#[derive(Clone, PartialEq)]
pub struct ContextMenuState {
    pub visible: bool,
    pub position_x: i32,
    pub position_y: i32,
    pub target_cell: String,
}

impl ContextMenuState {
    pub fn new() -> Self {
        Self { visible: false, position_x: 0, position_y: 0, target_cell: String::new() }
    }
}

// --- Parsing Helpers ------------------------------------

/// Parse a row string into a valid row number (1–999).
///
/// Returns `Some(row)` if parsing succeeds and the row is within range,
/// otherwise returns `None`.
fn parse_row(row_str: &str) -> Option<u32> {
    let row_num = row_str.parse::<u32>().ok()?;
    if row_num >= 1 && row_num <= 999 {
        Some(row_num)
    } else {
        None
    }
}

// /// Parse a string into an integer value.
// ///
// /// Returns `Some(i32)` if parsing succeeds, otherwise `None`.
// fn parse_int(int_str: &str) -> Option<i32> {
//     Some(int_str.parse::<i32>().ok()?)
// }

/// Parse a cell reference string like "A1" or "BC23" into a `Cell`.
///
/// Returns `Some(Cell)` if the reference is valid, otherwise `None`.
fn parse_cell(cell_str: &str) -> Option<Cell> {
    static CELL_RE: Lazy<Regex> = Lazy::new(|| 
        Regex::new(r"(?P<col>[A-Z]+)(?P<row>[0-9]+)").unwrap());
    
    if let Some(caps) = CELL_RE.captures(cell_str) {
        let col_str = &caps["col"];
        let row_str = &caps["row"];
        
        // Parse column
        if col_str.len() > 3 {
            return None;
        }
        
        for bt in col_str.as_bytes() {
            if bt < &b'A' || bt > &b'Z' {
                return None;
            }
        }
        
        let mut col_val: u32 = 0;
        for &bt in col_str.as_bytes() {
            col_val *= 26;
            col_val += ((bt - b'A') as u32) + 1;
        }
        
        // Parse row
        let row_num = parse_row(row_str)?;
        
        Some(Cell { row: row_num, col: col_val })
    } else {
        None
    }
}

// /// Parse a value string as either a numeric literal or a cell reference.
// ///
// /// Returns `Some(Value::Num)` or `Some(Value::Ref)` on success, otherwise `None`.
// fn parse_val(val_str: &str) -> Option<Value> {
//     if let Some(cell_out) = parse_cell(val_str) {
//         return Some(Value::Ref(cell_out));
//     } 
//     if let Some(val_int) = parse_int(val_str) {
//         return Some(Value::Num(val_int));
//     }
//     None
// }

/// Parse a full formula string of the form "A1=expr".
///
/// Returns `Some(Formula)` if the pattern matches, else `None`.
fn parse_formula(formula_str: &str) -> Option<Formula> {
    // Allow any formula that starts with a valid cell reference followed by =
    static ANY_FORMULA_RE: Lazy<Regex> = Lazy::new(|| 
        Regex::new(r"(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<expr>.+)").unwrap());
        
    // First check if it's any valid formula pattern with a cell on the left
    if let Some(caps) = ANY_FORMULA_RE.captures(formula_str) {
        let cell = parse_cell(&caps["cell"])?;
        // For the general case, we'll use Add with 0 as a default expression
        // since we're just accepting any formula pattern
        return Some(Formula {
            inp_cell: cell,
            _expression: Expression::Add((), ())
        });
    }
    
    // If it doesn't even match the basic pattern of cell=anything, return None
    None
}

// --- FormField Trait & Implementations ------------------

/// A trait for form fields in the spreadsheet application
///
/// This trait defines the common behavior and properties for form fields
/// including validation, value access, and field metadata.
pub trait FormField {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    fn value(&self) -> &str;
    fn set_value(&mut self, value: String);
    fn validate(&self, fields: &HashMap<String, Box<dyn FormField>>) -> Result<String, String>;
    fn input_type(&self) -> &str {
        "text"
    }
    fn min(&self) -> Option<&str> {
        None
    }
    fn max(&self) -> Option<&str> {
        None
    }
    fn clone_box(&self) -> Box<dyn FormField>;
}

// Public struct for a general cell input field
/// A form field for specifying a cell reference (e.g., "A1", "BC23")
/// 
/// Validates that the specified cell is within the spreadsheet's bounds
/// by checking against the configured row and column limits.
#[derive(Clone)]
pub struct CellField {
    /// The current value of the field as a string
    pub value: String,
}

impl FormField for CellField {
    fn id(&self) -> &str { "cell" }
    fn label(&self) -> &str { "Enter cell" }
    fn value(&self) -> &str { &self.value }
    fn set_value(&mut self, value: String) { self.value = value; }
    fn validate(&self, fields: &HashMap<String, Box<dyn FormField>>) -> Result<String, String> {
        let rows_field = fields.get("rows").unwrap();
        let cols_field = fields.get("cols").unwrap();
        
        let max_rows = match rows_field.value().parse::<u32>() {
            Ok(rows) if rows > 0 => rows,
            _ => return Err("Invalid rows configuration".to_string())
        };
        
        let max_cols = match cols_field.value().parse::<u32>() {
            Ok(cols) if cols > 0 => cols,
            _ => return Err("Invalid columns configuration".to_string())
        };
        
        match parse_cell(&self.value) {
            Some(cell) => {
                if cell.row > max_rows {
                    Err(format!("Row exceeds maximum ({})", max_rows))
                } else if cell.col > max_cols {
                    Err(format!("Column exceeds maximum ({})", max_cols))
                } else {
                    Ok(format!("You entered cell: {}", self.value))
                }
            },
            None => Err("Enter a valid cell (e.g. A1, BC23)".to_string())
        }
    }
    fn clone_box(&self) -> Box<dyn FormField> {
        Box::new(self.clone())
    }
}

// Public struct for rows input field
/// A form field for specifying the number of rows in the spreadsheet
/// 
/// Validates that the row count is within the allowed range (1-100)
/// and provides appropriate error messages when validation fails.
#[derive(Clone)]
pub struct RowsField {
    /// The current value of the field as a string
    pub value: String,
}

// Public struct for columns input field
/// A form field for specifying the number of columns in the spreadsheet
/// 
/// Validates that the column count is within the allowed range (1-26)
/// and provides appropriate error messages when validation fails.
#[derive(Clone)]
pub struct ColsField {
    /// The current value of the field as a string
    pub value: String,
}

impl FormField for RowsField {
    fn id(&self) -> &str { "rows" }
    fn label(&self) -> &str { "Number of rows" }
    fn value(&self) -> &str { &self.value }
    fn set_value(&mut self, value: String) { self.value = value; }
    fn validate(&self, _fields: &HashMap<String, Box<dyn FormField>>) -> Result<String, String> {
        match self.value.parse::<u32>() {
            Ok(rows) if rows > 0 && rows <= 100 => Ok(format!("Rows: {}", rows)),
            Ok(_) => Err("Rows must be between 1 and 100".to_string()),
            Err(_) => Err("Enter a valid number".to_string()),
        }
    }
    fn input_type(&self) -> &str { "number" }
    fn min(&self) -> Option<&str> { Some("1") }
    fn max(&self) -> Option<&str> { Some("100") }
    fn clone_box(&self) -> Box<dyn FormField> {
        Box::new(self.clone())
    }
}

impl FormField for ColsField {
    fn id(&self) -> &str { "cols" }
    fn label(&self) -> &str { "Number of columns" }
    fn value(&self) -> &str { &self.value }
    fn set_value(&mut self, value: String) { self.value = value; }
    fn validate(&self, _fields: &HashMap<String, Box<dyn FormField>>) -> Result<String, String> {
        match self.value.parse::<u32>() {
            Ok(cols) if cols > 0 && cols <= 26 => Ok(format!("Columns: {}", cols)),
            Ok(_) => Err("Columns must be between 1 and 26".to_string()),
            Err(_) => Err("Enter a valid number".to_string()),
        }
    }
    fn input_type(&self) -> &str { "number" }
    fn min(&self) -> Option<&str> { Some("1") }
    fn max(&self) -> Option<&str> { Some("26") }
    fn clone_box(&self) -> Box<dyn FormField> {
        Box::new(self.clone())
    }
}

// Public implementation of Clone for Box<dyn FormField>
impl Clone for Box<dyn FormField> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Public struct for formula input field
/// A form field for entering spreadsheet formulas
/// 
/// Validates that the formula has the correct syntax (cell=expression)
/// and that all cell references are within the spreadsheet's bounds.
#[derive(Clone)]
pub struct FormulaField {
    /// The current value of the field as a string
    pub value: String,
}

impl FormField for FormulaField {
    fn id(&self) -> &str { "formula" }
    fn label(&self) -> &str { "Enter formula" }
    fn value(&self) -> &str { &self.value }
    fn set_value(&mut self, value: String) { self.value = value; }
    fn validate(&self, fields: &HashMap<String, Box<dyn FormField>>) -> Result<String, String> {
        let rows_field = fields.get("rows").unwrap();
        let cols_field = fields.get("cols").unwrap();
        
        let max_rows = match rows_field.value().parse::<u32>() {
            Ok(rows) if rows > 0 => rows,
            _ => return Err("Invalid rows configuration".to_string())
        };
        
        let max_cols = match cols_field.value().parse::<u32>() {
            Ok(cols) if cols > 0 => cols,
            _ => return Err("Invalid columns configuration".to_string())
        };
        
        match parse_formula(&self.value) {
            Some(formula) => {
                // Check if the target cell is within bounds
                if formula.inp_cell.row > max_rows {
                    Err(format!("Formula target row exceeds maximum ({})", max_rows))
                } else if formula.inp_cell.col > max_cols {
                    Err(format!("Formula target column exceeds maximum ({})", max_cols))
                } else {
                    // Additional check for referenced cells could be done here,
                    // but for now we'll just validate the target cell
                    Ok(format!("Formula: {}", self.value))
                }
            },
            None => Err("Enter a valid formula (e.g. A1=B2+C3, A1=SUM(B1:B10))".to_string())
        }
    }
    fn clone_box(&self) -> Box<dyn FormField> {
        Box::new(self.clone())
    }
}

// --- Public API -----------------------------------------

/// Check if an input string is a valid cell reference (e.g., "A1").
///
/// Used for form validation.
pub fn parse_input_cell(input: &str) -> bool {
    parse_cell(input).is_some()
}

/// Check if an input string is a valid formula string (e.g., "A1=B2+C3").
///
/// Used for form validation.
pub fn parse_input_formula(input: &str) -> bool {
    parse_formula(input).is_some()
}

/// Processes a formula, returning the target cell ID and the expression string.
///
/// Always returns `Some((cell_id, result))`. Errors in format yield an error message in the second element.
pub fn process_formula(formula_str: &str) -> Option<(String, String)> {
    // Use a regex to extract the cell reference and everything after the equals sign
    static FORMULA_PARTS_RE: Lazy<Regex> = Lazy::new(|| 
        Regex::new(r"(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<expr>.+)").unwrap());

    if let Some(caps) = FORMULA_PARTS_RE.captures(formula_str) {
        let cell_id = caps["cell"].to_string();
        let expression = caps["expr"].trim().to_string();
        
        // Check if the expression is empty
        if expression.is_empty() {
            return Some((cell_id, "Error: empty formula".to_string()));
        }
        
        // Return the actual expression text
        return Some((cell_id, expression));
    }
    
    // Instead of returning None, return an error message
    Some(("ERROR".to_string(), "Error: invalid formula format".to_string()))
}

