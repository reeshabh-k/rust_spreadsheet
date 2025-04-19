use yew::prelude::*;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent, HtmlElement, MouseEvent, WheelEvent, Event, window};
use std::collections::HashMap;
use helper::*;
use wasm_bindgen::{JsCast, closure::Closure};
use serde::{Serialize, Deserialize};
use std::rc::Rc;
use std::cell::RefCell;

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

// Structure to represent the context menu state
#[derive(Clone, PartialEq)]
struct ContextMenuState {
    visible: bool,
    position_x: i32,
    position_y: i32,
    target_cell: String,
}

impl ContextMenuState {
    fn new() -> Self {
        Self {
            visible: false,
            position_x: 0,
            position_y: 0,
            target_cell: String::new(),
        }
    }
}

// Function to convert column number to label (1->A, 2->B, ..., 27->AA, 28->AB, etc.)
fn get_column_label(col: u32) -> String {
    if col == 0 {
        return String::new();
    }
    
    let mut result = String::new();
    let mut n = col;
    
    while n > 0 {
        // Convert to 0-based for calculation
        n -= 1;
        // Get the remainder when divided by 26 (A-Z)
        let remainder = n % 26;
        // Convert to ASCII character (A-Z)
        let ch = (b'A' + remainder as u8) as char;
        // Add to the beginning of the result
        result.insert(0, ch);
        // Integer division to get the next digit
        n /= 26;
    }
    
    result
}

// Function to convert column label to number (A->1, B->2, ..., AA->27, AB->28, etc.)
fn get_column_number(col_label: &str) -> u32 {
    let mut result: u32 = 0;
    
    for &byte in col_label.as_bytes() {
        // Check if character is A-Z
        if byte >= b'A' && byte <= b'Z' {
            // Multiply existing result by 26 (base 26 number system)
            result = result * 26;
            // Add the value of the current character (A=1, B=2, etc.)
            result += (byte - b'A' + 1) as u32;
        }
    }
    
    result
}

#[derive(Properties, PartialEq)]
struct ContextMenuProps {
    position_x: i32,
    position_y: i32,
    target_cell: String,
    onclose: Callback<()>,
    onaddformula: Callback<String>,
}

#[function_component(ContextMenu)]
fn context_menu(props: &ContextMenuProps) -> Html {
    let style = format!(
        "position: absolute; left: {}px; top: {}px; background-color: white; border: 1px solid #ccc; border-radius: 4px; box-shadow: 0 2px 5px rgba(0,0,0,0.2); padding: 10px; z-index: 1000;",
        props.position_x, props.position_y
    );

    // Close the context menu when clicking outside of it
    let document = use_state(|| {
        web_sys::window()
            .and_then(|win| win.document())
    });

    let onclose = props.onclose.clone();
    use_effect_with_deps(
        move |_| {
            let document_clone = document.clone();
            let onclose_clone = onclose.clone();
            
            let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
                onclose_clone.emit(());
            }) as Box<dyn FnMut(MouseEvent)>);
            
            if let Some(doc) = &*document_clone {
                doc.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).ok();
            }

            move || {
                if let Some(doc) = &*document_clone {
                    doc.remove_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).ok();
                }
                drop(closure);
            }
        },
        ()
    );

    // Initialize user input state (not the full formula string)
    let user_input = use_state(String::new);
    
    // Handler for formula input changes
    let on_formula_input = {
        let user_input = user_input.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            user_input.set(input.value());
        })
    };
    
    // Handler for submitting the formula
    let on_submit_formula = {
        let onaddformula = props.onaddformula.clone();
        let user_input = user_input.clone();
        let onclose = props.onclose.clone();
        let target_cell = props.target_cell.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            // Format the complete formula string with cell reference
            let formula = format!("{}={}", target_cell, *user_input);
            onaddformula.emit(formula);
            onclose.emit(());
        })
    };
    
    // Handler for key presses in the formula input
    let on_keydown = {
        let onaddformula = props.onaddformula.clone();
        let user_input = user_input.clone();
        let onclose = props.onclose.clone();
        let target_cell = props.target_cell.clone();
        
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.stop_propagation();
                e.prevent_default();
                // Format the complete formula string with cell reference
                let formula = format!("{}={}", target_cell, *user_input);
                onaddformula.emit(formula);
                onclose.emit(());
            } else if e.key() == "Escape" {
                e.stop_propagation();
                e.prevent_default();
                onclose.emit(());
            }
        })
    };

    html! {
        <div class="context-menu" style={style} onclick={|e: MouseEvent| e.stop_propagation()}>
            <div style="margin-bottom: 8px; font-weight: bold;">
                {format!("Add formula to {}", props.target_cell)}
            </div>
            <div class="formula-input-container">
                <div style="display: flex; align-items: center; margin-bottom: 8px;">
                    <span style="margin-right: 5px; font-weight: bold;">{format!("{}=", props.target_cell)}</span>
                    <input 
                        type="text" 
                        value={(*user_input).clone()}
                        oninput={on_formula_input}
                        onkeydown={on_keydown}
                        style="width: 100%; padding: 5px;"
                        autofocus={true}
                        placeholder="Enter formula..."
                    />
                </div>
                <button 
                    onclick={on_submit_formula}
                    style="padding: 5px 10px; cursor: pointer; background-color: #4CAF50; color: white; border: none; border-radius: 4px;"
                >
                    {"Apply Formula"}
                </button>
            </div>
        </div>
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
    #[prop_or_default]
    pub on_formula: Callback<String>,  // new callback for formula application
}

#[function_component(Spreadsheet)]
fn spreadsheet(props: &SpreadsheetProps) -> Html {
    // State for the currently selected cell
    let selected_cell = use_state(|| None::<String>);
    
    // Context menu state
    let context_menu_state = use_state(|| ContextMenuState::new());
    
    // Handle cell click to select a cell
    let on_cell_click = {
        let selected_cell = selected_cell.clone();
        let on_cell_select = props.on_cell_select.clone();
        let context_menu_state = context_menu_state.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation(); // Stop event propagation
            
            let target: HtmlElement = e.target_unchecked_into();
            if let Some(cell_id) = target.get_attribute("data-id") {
                selected_cell.set(Some(cell_id.clone()));
                on_cell_select.emit(cell_id.clone());
                
                // Show the formula dialog when clicking on a cell
                context_menu_state.set(ContextMenuState {
                    visible: true,
                    position_x: e.client_x(),
                    position_y: e.client_y(),
                    target_cell: cell_id,
                });
            }
        })
    };

    // Handle right-click on cells
    let on_cell_context_menu = {
        let context_menu_state = context_menu_state.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.prevent_default(); // Prevent the default browser context menu
            
            let target: HtmlElement = e.target_unchecked_into();
            if let Some(cell_id) = target.get_attribute("data-id") {
                context_menu_state.set(ContextMenuState {
                    visible: true,
                    position_x: e.client_x(),
                    position_y: e.client_y(),
                    target_cell: cell_id,
                });
            }
        })
    };

    // Handle closing the context menu
    let on_close_context_menu = {
        let context_menu_state = context_menu_state.clone();
        
        Callback::from(move |_| {
            context_menu_state.set(ContextMenuState::new());
        })
    };

    // Handle "Add Formula" option from context menu
    let on_add_formula = {
        let on_formula = props.on_formula.clone();
         
        Callback::from(move |formula: String| {
            // Emit formula event to parent to process and update cell values
            on_formula.emit(formula);
        })
    };

    // Handle keyboard navigation
    let onkeydown = {
        let selected_cell = selected_cell.clone();
        let on_cell_select = props.on_cell_select.clone();
        let rows = props.rows;
        let cols = props.cols;
        
        Callback::from(move |e: KeyboardEvent| {
            if let Some(current_cell) = (*selected_cell).clone() {
                // Parse the current cell ID (e.g., "A1", "AA23", etc.)
                // Find the index where the digits start
                let mut digit_start_index = 0;
                for (i, c) in current_cell.chars().enumerate() {
                    if c.is_digit(10) {
                        digit_start_index = i;
                        break;
                    }
                }
                
                if digit_start_index > 0 {
                    let col_label = &current_cell[0..digit_start_index];
                    let row = current_cell[digit_start_index..].parse::<u32>().unwrap_or(1);
                    let col_num = get_column_number(col_label);
                    
                    match e.key().as_str() {
                        "ArrowUp" => {
                            e.prevent_default();
                            if row > 1 {
                                let new_cell = format!("{}{}", col_label, row - 1);
                                selected_cell.set(Some(new_cell.clone()));
                                on_cell_select.emit(new_cell);
                            }
                        },
                        "ArrowDown" => {
                            e.prevent_default();
                            if row < rows {
                                let new_cell = format!("{}{}", col_label, row + 1);
                                selected_cell.set(Some(new_cell.clone()));
                                on_cell_select.emit(new_cell);
                            }
                        },
                        "ArrowLeft" => {
                            e.prevent_default();
                            if col_num > 1 {
                                let new_col = get_column_label(col_num - 1);
                                let new_cell = format!("{}{}", new_col, row);
                                selected_cell.set(Some(new_cell.clone()));
                                on_cell_select.emit(new_cell);
                            }
                        },
                        "ArrowRight" => {
                            e.prevent_default();
                            if col_num < cols {
                                let new_col = get_column_label(col_num + 1);
                                let new_cell = format!("{}{}", new_col, row);
                                selected_cell.set(Some(new_cell.clone()));
                                on_cell_select.emit(new_cell);
                            }
                        },
                        _ => {}
                    }
                }
            }
        })
    };
    
    // NodeRef for scrolling container
    let container_ref = use_node_ref();
    // Handle wheel events to scroll both vertically and horizontally in the spreadsheet
    let on_wheel = {
        let container_ref = container_ref.clone();
        Callback::from(move |e: WheelEvent| {
            e.prevent_default();
            if let Some(div) = container_ref.cast::<HtmlElement>() {
                // Determine if we should do horizontal scrolling based on shift key or delta_x
                let delta_x = if e.shift_key() && e.delta_x() == 0.0 {
                    // If shift key is pressed and there's no horizontal movement,
                    // use the vertical delta for horizontal scrolling
                    e.delta_y() 
                } else {
                    // Otherwise use the actual horizontal delta
                    e.delta_x()
                };
                
                // Apply horizontal scrolling
                let new_left = (div.scroll_left() as f64 + delta_x) as i32;
                div.set_scroll_left(new_left);
                
                // Apply vertical scrolling only if shift key is not pressed or there's actual vertical delta
                if !e.shift_key() || e.delta_x() != 0.0 {
                    let new_top = (div.scroll_top() as f64 + e.delta_y()) as i32;
                    div.set_scroll_top(new_top);
                }
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
                    oncontextmenu={on_cell_context_menu.clone()}
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
        <div ref={container_ref.clone()} class="spreadsheet-container" tabindex="0" onkeydown={onkeydown.clone()} onwheel={on_wheel}>
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
            if context_menu_state.visible {
                <ContextMenu
                    position_x={context_menu_state.position_x}
                    position_y={context_menu_state.position_y}
                    target_cell={context_menu_state.target_cell.clone()}
                    onclose={on_close_context_menu.clone()}
                    onaddformula={on_add_formula.clone()}
                />
            }
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    // Store fields in a map for easy access
    let fields = use_state(|| {
        let mut map: HashMap<String, Box<dyn FormField>> = HashMap::new();
        map.insert("rows".to_string(), Box::new(RowsField { value: "100".to_string() })); // Max rows
        map.insert("cols".to_string(), Box::new(ColsField { value: "55".to_string() })); // Max columns (A-Z)
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
            // Initialize only 28 columns (A through AB) instead of all columns
            for col in 1..=28 {
                let cell_id = format!("{}{}", get_column_label(col), row);
                data.insert(cell_id, "0".to_string());
            }
        }
        data
    });
    
    // Check if dimensions are valid
    let dimensions_valid = rows > 0 && cols > 0 && rows <= 100 && cols <= 55;
    
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
                                
                                // Use the shared process_formula function from helper library
                                if let Some((cell_id, value)) = process_formula(&formula) {
                                    // Update our local state with the processed formula result
                                    let mut updated_values = (*cell_values).clone();
                                    updated_values.insert(cell_id, value);
                                    cell_values.set(updated_values);
                                    
                                    // Add success message
                                    updated_messages.insert(field_id.clone(), 
                                        format!("Formula applied: {}", formula));
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
                    on_cell_select={on_cell_select.clone()}
                    cell_values={(*cell_values).clone()}
                    on_formula={
                        let cell_values = cell_values.clone();
                        let on_cell_select = on_cell_select.clone();
                        Callback::from(move |formula: String| {
                            // Process formula and update state
                            if let Some((cell_id, value)) = process_formula(&formula) {
                                let mut updated = (*cell_values).clone();
                                updated.insert(cell_id.clone(), value);
                                cell_values.set(updated);
                                // Optionally select the cell after formula apply
                                on_cell_select.emit(cell_id);
                            }
                        })
                    }
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
            
            <div class="file-upload-container">
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
            
            <div class="save-button-container">
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
            </div>
        </>
    }
}

fn main(){
    yew::Renderer::<App>::new().render();   
}