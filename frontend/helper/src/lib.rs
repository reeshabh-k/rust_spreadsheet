use yew::prelude::*;

// Define a public trait for form fields
pub trait FormField {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    fn value(&self) -> &str;
    fn set_value(&mut self, value: String);
    fn validate(&self) -> Result<String, String>;
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
    fn validate(&self) -> Result<String, String> {
        if parse_input_cell(&self.value) {
            Ok(format!("You entered cell: {}", self.value))
        } else {
            Err("Enter a valid cell".to_string())
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
    fn validate(&self) -> Result<String, String> {
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
    fn validate(&self) -> Result<String, String> {
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
    fn validate(&self) -> Result<String, String> {
        if parse_input_formula(&self.value) {
            Ok(format!("Formula: {}", self.value))
        } else {
            Err("Enter a valid formula".to_string())
        }
    }
    fn clone_box(&self) -> Box<dyn FormField> {
        Box::new(self.clone())
    }
}

// Dummy validation functions (you can replace with actual logic)
pub fn parse_input_cell(input: &str) -> bool {
    !input.trim().is_empty()
}

pub fn parse_input_formula(input: &str) -> bool {
    !input.trim().is_empty()
}
