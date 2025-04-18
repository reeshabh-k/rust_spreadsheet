use yew::prelude::*;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use regex::Regex;

// Define struct for cell representation
#[derive(Debug)]
struct Cell {
    row: u32,
    col: u32,
}

// Define enum for values in formulas (either numbers or cell references)
#[derive(Debug)]
enum Value {
    Num(i32),
    Ref(Cell),
}

// Expression types supported in the spreadsheet
#[derive(Debug)]
enum Expression {
    Add(Value, Value),
    Sub(Value, Value),
    Mul(Value, Value),
    Div(Value, Value),
    Min(Cell, Cell),
    Max(Cell, Cell),
    Avg(Cell, Cell),
    Sum(Cell, Cell),
    Stdev(Cell, Cell),
    Sleep(Value),
}

// Formula representation
#[derive(Debug)]
struct Formula {
    inp_cell: Cell,
    expression: Expression,
}

// Parsing helper functions
fn parse_row(row_str: &str) -> Option<u32> {
    let row_num = row_str.parse::<u32>().ok()?;
    if row_num >= 1 && row_num <= 999 {
        Some(row_num)
    } else {
        None
    }
}

fn parse_int(int_str: &str) -> Option<i32> {
    Some(int_str.parse::<i32>().ok()?)
}

// Parse a cell reference like "A1", "B23", etc.
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

// Parse a value (either a number or a cell reference)
fn parse_val(val_str: &str) -> Option<Value> {
    if let Some(cell_out) = parse_cell(val_str) {
        return Some(Value::Ref(cell_out));
    } 
    if let Some(val_int) = parse_int(val_str) {
        return Some(Value::Num(val_int));
    }
    None
}

// Parse a formula string
fn parse_formula(formula_str: &str) -> Option<Formula> {
    // Regex patterns for different formula types
    static BINARY_OP_RE: Lazy<Regex> = Lazy::new(|| 
        Regex::new(r"(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<val1>-?\d+|[A-Z]+[0-9]+)\s*(?P<op>['*'|'/'|'-'|'+'])\s*(?P<val2>-?\d+|[A-Z]+[0-9]+)").unwrap());
    
    static RANGE_OP_RE: Lazy<Regex> = Lazy::new(|| 
        Regex::new(r"(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<op>MAX|MIN|STDEV|AVG|SUM)\s*['(']\s*(?P<cell1>[A-Z]+[0-9]+)\s*:\s*(?P<cell2>[A-Z]+[0-9]+)\s*[')']").unwrap());
    
    static SLEEP_OP_RE: Lazy<Regex> = Lazy::new(|| 
        Regex::new(r"(?P<cell>[A-Z]+[0-9]+)\s*=\s*SLEEP\s*['(']\s*(?P<val>-?\d+|[A-Z]+[0-9]+)\s*[')']").unwrap());
    
    // Try matching binary operation formula (e.g. A1=B2+C3)
    if let Some(caps) = BINARY_OP_RE.captures(formula_str) {
        let cell = parse_cell(&caps["cell"])?;
        let val1 = parse_val(&caps["val1"])?;
        let val2 = parse_val(&caps["val2"])?;
        
        let op = &caps["op"];
        
        let form = match op {
            "+" => Formula {
                inp_cell: cell,
                expression: Expression::Add(val1, val2)
            },
            "-" => Formula {
                inp_cell: cell,
                expression: Expression::Sub(val1, val2)
            },
            "/" => Formula {
                inp_cell: cell,
                expression: Expression::Div(val1, val2)
            },
            "*" => Formula {
                inp_cell: cell,
                expression: Expression::Mul(val1, val2)
            },
            _ => return None
        };
        
        return Some(form);
    }
    
    // Try matching range function formula (e.g. A1=SUM(B1:B10))
    if let Some(caps) = RANGE_OP_RE.captures(formula_str) {
        let cell = parse_cell(&caps["cell"])?;
        let cell1 = parse_cell(&caps["cell1"])?;
        let cell2 = parse_cell(&caps["cell2"])?;
        
        let op = &caps["op"];
        
        let form = match op {
            "MAX" => Formula {
                inp_cell: cell,
                expression: Expression::Max(cell1, cell2)
            },
            "MIN" => Formula {
                inp_cell: cell,
                expression: Expression::Min(cell1, cell2)
            },
            "AVG" => Formula {
                inp_cell: cell,
                expression: Expression::Avg(cell1, cell2)
            },
            "STDEV" => Formula {
                inp_cell: cell,
                expression: Expression::Stdev(cell1, cell2)
            },
            "SUM" => Formula {
                inp_cell: cell,
                expression: Expression::Sum(cell1, cell2)
            },
            _ => return None
        };
        
        return Some(form);
    }
    
    // Try matching sleep function formula (e.g. A1=SLEEP(500))
    if let Some(caps) = SLEEP_OP_RE.captures(formula_str) {
        let cell = parse_cell(&caps["cell"])?;
        let val = parse_val(&caps["val"])?;
        
        let form = Formula {
            inp_cell: cell,
            expression: Expression::Sleep(val)
        };
        
        return Some(form);
    }
    
    None
}

// Define a public trait for form fields
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
#[derive(Clone)]
pub struct CellField {
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
#[derive(Clone)]
pub struct RowsField {
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

// Public struct for columns input field
#[derive(Clone)]
pub struct ColsField {
    pub value: String,
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
#[derive(Clone)]
pub struct FormulaField {
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

// Helper functions for external use with simplified interface
pub fn parse_input_cell(input: &str) -> bool {
    parse_cell(input).is_some()
}

pub fn parse_input_formula(input: &str) -> bool {
    parse_formula(input).is_some()
}
