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

#[derive(Clone)]
struct FormulaField {
    value: String,
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

#[function_component(App)]
fn app() -> Html {
    // Store fields in a map for easy access
    let fields = use_state(|| {
        let mut map: HashMap<String, Box<dyn FormField>> = HashMap::new();
        // map.insert("username".to_string(), Box::new(NameField { value: "".to_string() }));
        map.insert("rows".to_string(), Box::new(RowsField { value: "".to_string() }));
        map.insert("cols".to_string(), Box::new(ColsField { value: "".to_string() }));
        map.insert("cell".to_string(), Box::new(CellField { value: "".to_string() }));
        map.insert("formula".to_string(), Box::new(FormulaField { value: "".to_string() }));
        map
    });
    let dimensions_valid = {
        let fields = &*fields;
        
        // Check if rows are valid
        let rows_valid = if let Some(rows_field) = fields.get("rows") {
            if let Ok(rows) = rows_field.value().parse::<u32>() {
                rows > 0 && rows <= 100
            } else {
                false
            }
        } else {
            false
        };
        
        // Check if columns are valid
        let cols_valid = if let Some(cols_field) = fields.get("cols") {
            if let Ok(cols) = cols_field.value().parse::<u32>() {
                cols > 0 && cols <= 26
            } else {
                false
            }
        } else {
            false
        };
        
        // Both must be valid
        rows_valid && cols_valid
    };
    
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
    // let field_order = vec!["rows", "cols", "cell", "formula"]; // Define your preferred order here
    // let form_fields = field_order.iter()
    //     .filter_map(|field_id| {
    //         if let Some(field) = (*fields).get(*field_id) {
    //             let message = messages.get(*field_id).cloned().unwrap_or_default();
                
    //             Some(html! {
    //                 <>
    //                     <div style="margin-top:2rem;">
    //                         <label for={field.id().to_string()}> {format!("{}: ", field.label())} </label>
    //                         <input
    //                             id={field.id().to_string()}
    //                             type={field.input_type().to_string()}
    //                             value={field.value().to_string()}
    //                             min={field.min().map(|s| s.to_string())}
    //                             max={field.max().map(|s| s.to_string())}
    //                             oninput={oninput.clone()}
    //                             onkeydown={onkeydown.clone()}
    //                         />
    //                         <p>{message}</p>
    //                     </div>
    //                 </>
    //             })
    //         } else {
    //             None
    //         }
    //     })
    //     .collect::<Vec<_>>();

    let dimension_fields = vec!["rows", "cols"].iter()
        .filter_map(|field_id| {
            if let Some(field) = (*fields).get(*field_id) {
                let message = messages.get(*field_id).cloned().unwrap_or_default();
                
                Some(html! {
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
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    
    // Generate cell and formula fields only if dimensions are valid
    let content_fields = if dimensions_valid {
        vec!["cell", "formula"].iter()
            .filter_map(|field_id| {
                if let Some(field) = (*fields).get(*field_id) {
                    let message = messages.get(*field_id).cloned().unwrap_or_default();
                    
                    Some(html! {
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
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        // If dimensions aren't valid, show a message
        vec![html! {
            <div style="margin-top:2rem; color: #777;">
                {"Please enter valid dimensions to continue."}
            </div>
        }]
    };
    
    html! {
        <>
            <h1> {"Rust Spreadsheet"} </h1>
            // { for form_fields }
            { for dimension_fields }
            { for content_fields }
        </>
    }
}

fn parse_input_formula(_formula: &String) -> bool {
    // For now, accept any input as valid
    // You can add actual formula validation later
    true
}

fn parse_input_cell(_cell : &String) -> bool{true}

fn main(){
    yew::Renderer::<App>::new().render();   
}