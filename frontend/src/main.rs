use yew::prelude::*;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent};
use std::collections::HashMap;



// Define a trait for form fields
trait FormField {
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

// Concrete field implementations
#[derive(Clone)]
struct CellField {
    value: String,
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

#[derive(Clone)]
struct RowsField {
    value: String,
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

#[derive(Clone)]
struct ColsField {
    value: String,
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

// Custom Clone implementation for Box<dyn FormField>
impl Clone for Box<dyn FormField> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[function_component(App)]
fn app() -> Html {
    // Store fields in a map for easy access
    let fields = use_state(|| {
        let mut map: HashMap<String, Box<dyn FormField>> = HashMap::new();
        // map.insert("username".to_string(), Box::new(NameField { value: "".to_string() }));
        map.insert("cell".to_string(), Box::new(CellField { value: "".to_string() }));
        map.insert("rows".to_string(), Box::new(RowsField { value: "".to_string() }));
        map.insert("cols".to_string(), Box::new(ColsField { value: "".to_string() }));
        map
    });
    
    // Store validation messages
    let messages = use_state(|| HashMap::<String, String>::new());
    
    // Generic input handler
    let oninput = {
        let fields = fields.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let field_id = input.id();
            let value = input.value();
            
            let mut updated_fields = (*fields).clone();
            if let Some(field) = updated_fields.get_mut(&field_id) {
                field.set_value(value);
                fields.set(updated_fields);
            }
        })
    };
    
    // Generic validation handler
    let onkeydown = {
        let fields = fields.clone();
        let messages = messages.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: HtmlInputElement = e.target_unchecked_into();
                let field_id = input.id();
                
                let mut updated_messages = (*messages).clone();
                if let Some(field) = (*fields).get(&field_id) {
                    match field.validate() {
                        Ok(message) => { updated_messages.insert(field_id.clone(), message); },
                        Err(error) => { updated_messages.insert(field_id.clone(), error); },
                    }
                    messages.set(updated_messages);
                }
            }
        })
    };

    // Generate form fields
    let form_fields = (*fields).keys().map(|field_id| {
        let field = (*fields).get(field_id).unwrap();
        let message = messages.get(field_id).cloned().unwrap_or_default();
        
        html! {
            <>
                <div style="margin-top:2rem;">
                    <label for={field.id().to_string()}> {format!("{}: ", field.label())} </label>
                    <input
                        id={field.id().to_string()}
                        type={field.input_type().to_string()}
                        value={field.value().to_string()}
                        min={field.min().map(|s| s.to_string())}
                        max={field.max().map(|s| s.to_string())}
                        oninput={oninput.clone()}
                        onkeydown={onkeydown.clone()}
                    />
                    <p>{message}</p>
                </div>
            </>
        }
    }).collect::<Vec<_>>();
    
    html! {
        <>
            <h1> {"Rust Spreadsheet"} </h1>
            { for form_fields }
        </>
    }
}

// Keep existing validation functions
fn parse_input_name(name: &String) -> bool{
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();

    let first_char_uppercase = match chars.next(){
        Some(c) => c.is_uppercase(),
        None => false,
    };

    if !first_char_uppercase {return false;}
    for c in chars {
        if !(c.is_lowercase() || c==' ') {
            return false;
        }
    }
    true
}

fn parse_input_cell(cell : &String) -> bool{true}

fn main(){
    yew::Renderer::<App>::new().render();   
}