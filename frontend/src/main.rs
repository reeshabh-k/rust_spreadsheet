use yew::prelude::*;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent, HtmlElement, MouseEvent, Event};
use std::collections::HashMap;
use helper::*;
use wasm_bindgen::{JsCast, closure::Closure};
use serde::{Serialize, Deserialize};

// Cell data structure to store cell values and formulas
#[derive(Clone, PartialEq)]
struct CellData {
    value: String,
    formula: Option<String>,
}

impl CellData {
    fn new() -> Self {
        Self {
            value: "0".to_string(),
            formula: None,
        }
    }
}

// Spreadsheet component to display a grid of cells
#[derive(Properties, PartialEq)]
pub struct SpreadsheetProps {
    pub rows: u32,
    pub cols: u32,
    #[prop_or_default]
    pub on_cell_select: Callback<String>,
    #[prop_or_default]
    pub cell_values: HashMap<String, String>,
}

#[function_component(Spreadsheet)]
fn spreadsheet(props: &SpreadsheetProps) -> Html {
    // State for the currently selected cell
    let selected_cell = use_state(|| None::<String>);
    
    // Function to convert column number to label (1->A, 2->B, etc.)
    fn get_column_label(col: u32) -> String {
        match col {
            1 => "A".to_string(),
            2 => "B".to_string(),
            3 => "C".to_string(),
            4 => "D".to_string(),
            5 => "E".to_string(),
            6 => "F".to_string(),
            7 => "G".to_string(),
            8 => "H".to_string(),
            9 => "I".to_string(),
            10 => "J".to_string(),
            11 => "K".to_string(),
            12 => "L".to_string(),
            13 => "M".to_string(),
            14 => "N".to_string(),
            15 => "O".to_string(),
            16 => "P".to_string(),
            17 => "Q".to_string(),
            18 => "R".to_string(),
            19 => "S".to_string(),
            20 => "T".to_string(),
            21 => "U".to_string(),
            22 => "V".to_string(),
            23 => "W".to_string(),
            24 => "X".to_string(),
            25 => "Y".to_string(),
            26 => "Z".to_string(),
            _ => format!("Col{}", col),
        }
    }
    
    // Handle cell click to select a cell
    let on_cell_click = {
        let selected_cell = selected_cell.clone();
        let on_cell_select = props.on_cell_select.clone();
        
        Callback::from(move |e: MouseEvent| {
            let target: HtmlElement = e.target_unchecked_into();
            if let Some(cell_id) = target.get_attribute("data-id") {
                selected_cell.set(Some(cell_id.clone()));
                on_cell_select.emit(cell_id);
            }
        })
    };
    
    // Creates column headers (A, B, C, etc.)
    let column_headers = (1..=props.cols).map(|col| {
        let col_label = get_column_label(col);
        
        html! {
            <th class="column-header">{ col_label }</th>
        }
    }).collect::<Html>();
    
    // Creates all rows with cells
    let rows = (1..=props.rows).map(|row| {
        let cells = (1..=props.cols).map(|col| {
            let cell_id = format!("{}{}", get_column_label(col), row);
            
            // Get cell value from props (from parent component)
            let cell_value = props.cell_values.get(&cell_id)
                .cloned()
                .unwrap_or_else(|| "0".to_string());
            
            // Check if this cell is currently selected
            let is_selected = selected_cell.as_ref()
                .map_or(false, |id| *id == cell_id);
            
            let cell_class = if is_selected {
                "cell selected"
            } else {
                "cell"
            };
            
            html! {
                <td
                    class={cell_class}
                    data-id={cell_id.clone()}
                    onclick={on_cell_click.clone()}
                >
                    { cell_value }
                </td>
            }
        }).collect::<Html>();
        
        html! {
            <tr>
                <th class="row-header">{ row }</th>
                { cells }
            </tr>
        }
    }).collect::<Html>();
    
    html! {
        <div class="spreadsheet-container">
            <table class="spreadsheet">
                <thead>
                    <tr>
                        <th class="corner-header"></th>
                        { column_headers }
                    </tr>
                </thead>
                <tbody>
                    { rows }
                </tbody>
            </table>
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    // Store fields in a map for easy access
    let fields = use_state(|| {
        let mut map: HashMap<String, Box<dyn FormField>> = HashMap::new();
        map.insert("rows".to_string(), Box::new(RowsField { value: "100".to_string() })); // Max rows
        map.insert("cols".to_string(), Box::new(ColsField { value: "26".to_string() })); // Max columns (A-Z)
        map.insert("cell".to_string(), Box::new(CellField { value: "".to_string() }));
        map.insert("formula".to_string(), Box::new(FormulaField { value: "".to_string() }));
        map
    });
    
    // Store validation messages
    let messages = use_state(|| HashMap::<String, String>::new());
    
    // Get the current number of rows and columns
    let rows = if let Some(rows_field) = (*fields).get("rows") {
        rows_field.value().parse::<u32>().unwrap_or(5) // Default to 5 rows
    } else {
        5
    };
    
    let cols = if let Some(cols_field) = (*fields).get("cols") {
        cols_field.value().parse::<u32>().unwrap_or(5) // Default to 5 columns
    } else {
        5
    };
    
    // Store cell data for fake API state
    let cell_values = use_state(|| {
        let mut data = HashMap::new();
        for row in 1..=rows {
            for col in 1..=cols {
                let cell_id = match col {
                    1 => format!("A{}", row),
                    2 => format!("B{}", row),
                    3 => format!("C{}", row),
                    4 => format!("D{}", row),
                    5 => format!("E{}", row),
                    6 => format!("F{}", row),
                    7 => format!("G{}", row),
                    8 => format!("H{}", row),
                    9 => format!("I{}", row),
                    10 => format!("J{}", row),
                    // Add more as needed...
                    _ => format!("Col{}{}", col, row),
                };
                data.insert(cell_id, "0".to_string());
            }
        }
        data
    });
    
    // Check if dimensions are valid
    let dimensions_valid = rows > 0 && cols > 0 && rows <= 100 && cols <= 26;
    
    // Generic input handler
    let oninput = {
        let fields = fields.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let field_id = input.id();
            let value = input.value();
            
            // Only update fields other than rows and cols immediately
            // For rows and cols, we'll update them on Enter keypress
            let mut updated_fields = (*fields).clone();
            if let Some(field) = updated_fields.get_mut(&field_id) {
                if field_id != "rows" && field_id != "cols" {
                    field.set_value(value);
                    fields.set(updated_fields);
                }
                // For rows and cols, the temporary value is stored in the HTML input,
                // but we'll only update the field state when Enter is pressed
            }
        })
    };
    
    // Handle cell selection in the spreadsheet
    let on_cell_select = {
        let fields = fields.clone();
        
        Callback::from(move |cell_id: String| {
            let mut updated_fields = (*fields).clone();
            if let Some(field) = updated_fields.get_mut("cell") {
                field.set_value(cell_id);
                fields.set(updated_fields);
            }
        })
    };
    
    // Generic validation handler with formula processing
    let onkeydown = {
        let fields = fields.clone();
        let messages = messages.clone();
        let cell_values = cell_values.clone();
        
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: HtmlInputElement = e.target_unchecked_into();
                let field_id = input.id();
                let value = input.value();
                
                // For rows and cols fields, update their values on Enter key press
                let mut updated_fields = (*fields).clone();
                if field_id == "rows" || field_id == "cols" {
                    if let Some(field) = updated_fields.get_mut(&field_id) {
                        field.set_value(value);
                        fields.set(updated_fields.clone());
                    }
                }
                
                let mut updated_messages = (*messages).clone();
                if let Some(field) = updated_fields.get(&field_id) {
                    match field.validate(&updated_fields) {
                        Ok(message) => { 
                            updated_messages.insert(field_id.clone(), message); 
                            
                            // Process formula if valid
                            if field_id == "formula" {
                                let formula = field.value();
                                // In the future, this would call the API
                                // For now, we'll do a simple processing for demo purposes
                                
                                // Simple regex to extract cell reference from formula
                                if let Some(cell_ref) = formula.split('=').next() {
                                    let cell_ref = cell_ref.trim();
                                    if parse_input_cell(cell_ref) {
                                        // Here we would normally call the backend API
                                        // For now, just update our local state
                                        let mut updated_values = (*cell_values).clone();
                                        
                                        // For demo, set cell value to "42" if formula is valid
                                        updated_values.insert(cell_ref.to_string(), "42".to_string());
                                        cell_values.set(updated_values);
                                        
                                        // Add success message
                                        updated_messages.insert(field_id.clone(), 
                                            format!("Formula applied: {}", formula));
                                    }
                                }
                            }
                        },
                        Err(error) => { updated_messages.insert(field_id.clone(), error); },
                    }
                    messages.set(updated_messages);
                }
            }
        })
    };

    // Render content based on dimensions validity
    let content = if dimensions_valid {
        html! {
            <div class="content-container">
                <div class="cell-formula-inputs">
                    {
                        vec!["cell", "formula"].iter()
                            .filter_map(|field_id| {
                                if let Some(field) = (*fields).get(*field_id) {
                                    let message = messages.get(*field_id).cloned().unwrap_or_default();
                                    
                                    Some(html! {
                                        <div class="input-field">
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
                            .collect::<Html>()
                    }
                </div>
                <Spreadsheet 
                    rows={rows} 
                    cols={cols}
                    on_cell_select={on_cell_select}
                    cell_values={(*cell_values).clone()}
                />
                
                <div class="api-info">
                    <p class="note">{"Note: In the future, this spreadsheet will connect to a backend API to process formulas and update cell values."}</p>
                    <p class="instructions">{"To use the spreadsheet: Click on a cell to select it, then enter a formula like \"A1=10+B2\" and press Enter."}</p>
                </div>
            </div>
        }
    } else {
        // If dimensions aren't valid, show a message
        html! {
            <div style="margin-top:2rem; color: #777;">
                {"Please enter valid dimensions to continue."}
            </div>
        }
    };
    
    html! {
        <>
            <h1> {"Rust Spreadsheet"} </h1>
            <div class="action-buttons">
                <button 
                    onclick={
                        let cell_values = cell_values.clone();
                        Callback::from(move |_| {
                            // Convert cell_values to JSON and save as file
                            let json_data = serde_json::to_string(&*cell_values).unwrap_or_default();
                            
                            // Use web_sys to create and download a file
                            let window = web_sys::window().unwrap();
                            let document = window.document().unwrap();
                            let element = document.create_element("a").unwrap();
                            let element = element.dyn_into::<web_sys::HtmlElement>().unwrap();
                            
                            // Create a blob URL for the JSON data
                            let blob_props = web_sys::BlobPropertyBag::new();
                            let blob = web_sys::Blob::new_with_str_sequence_and_options(
                                &js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(&json_data)),
                                &blob_props
                            ).unwrap();
                            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                            
                            // Set up the download link
                            element.set_attribute("href", &url).unwrap();
                            element.set_attribute("download", "spreadsheet.json").unwrap();
                            element.style().set_css_text("display: none");
                            
                            // Append to document, click, and clean up
                            let body = document.body().unwrap();
                            body.append_child(&element).unwrap();
                            element.click();
                            body.remove_child(&element).unwrap();
                            web_sys::Url::revoke_object_url(&url).unwrap();
                        })
                    }
                    class="action-button"
                >
                    {"Save Spreadsheet"}
                </button>
                
                <div class="file-upload">
                    <label for="spreadsheet-upload">{"Upload Spreadsheet: "}</label>
                    <input
                        id="spreadsheet-upload"
                        type="file"
                        accept=".json"
                        onchange={
                            let cell_values = cell_values.clone();
                            Callback::from(move |e: Event| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                if let Some(file_list) = input.files() {
                                    if let Some(file) = file_list.get(0) {
                                        let file_reader = web_sys::FileReader::new().unwrap();
                                        let cell_values_clone = cell_values.clone();
                                        
                                        // Set up onload handler
                                        let onload_callback = Closure::wrap(Box::new(move |e: Event| {
                                            let target: web_sys::FileReader = e.target().unwrap().dyn_into().unwrap();
                                            let result = target.result().unwrap();
                                            let json_str = result.as_string().unwrap();
                                            
                                            // Parse JSON and update cell_values
                                            match serde_json::from_str::<HashMap<String, String>>(&json_str) {
                                                Ok(data) => {
                                                    cell_values_clone.set(data);
                                                },
                                                Err(_) => {
                                                    web_sys::window()
                                                        .unwrap()
                                                        .alert_with_message("Invalid spreadsheet file format")
                                                        .unwrap();
                                                }
                                            }
                                        }) as Box<dyn FnMut(Event)>);
                                        
                                        file_reader.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
                                        file_reader.read_as_text(&file).unwrap();
                                        onload_callback.forget(); // Prevent callback from being dropped
                                    }
                                }
                            })
                        }
                    />
                </div>
            </div>
            { content }
        </>
    }
}

fn main(){
    yew::Renderer::<App>::new().render();   
}