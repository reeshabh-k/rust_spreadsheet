use yew::prelude::*;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent};
use std::collections::HashMap;
use helper::*;



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



fn main(){
    yew::Renderer::<App>::new().render();   
}