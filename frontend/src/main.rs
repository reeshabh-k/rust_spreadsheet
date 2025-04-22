use yew::prelude::*;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent, HtmlElement, MouseEvent, WheelEvent, Event};
use std::collections::HashMap;
use helper::*;
use wasm_bindgen::{JsCast, closure::Closure};
use serde::{Serialize, Deserialize};
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;


const API_URL: &str = "http://localhost:8080/api";
// Visualization state structure for bar charts
#[derive(Clone, PartialEq)]
struct VisualizationState {
    visible: bool,
    data: Vec<(String, Vec<(String, f64)>)>,  // Vec of (Category, Vec of (label, value)) pairs to maintain order
    title: String,  // Main title from cell A1
}

impl VisualizationState {
    fn new() -> Self {
        Self {
            visible: false,
            data: Vec::new(),
            title: "Data Analysis".to_string(),
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

// Toast notification component for displaying errors
#[derive(Clone, PartialEq)]
struct ToastNotification {
    visible: bool,
    message: String,
    is_error: bool,
}

impl ToastNotification {
    fn new() -> Self {
        Self {
            visible: false,
            message: String::new(),
            is_error: false,
        }
    }
    
    fn show_error(message: String) -> Self {
        Self {
            visible: true,
            message,
            is_error: true,
        }
    }
}

// Toast component props
#[derive(Properties, PartialEq)]
struct ToastProps {
    visible: bool,
    message: String,
    is_error: bool,
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
        <div class="context-menu" onclick={|e: MouseEvent| e.stop_propagation()}>
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
    #[prop_or_default]
    pub on_scroll_to_cell_ref: Callback<UseStateHandle<Option<Callback<String>>>>, // Get reference to the scroll_to_cell callback
}

#[function_component(Spreadsheet)]
fn spreadsheet(props: &SpreadsheetProps) -> Html {
    // State for the currently selected cell
    let selected_cell = use_state(|| None::<String>);
    
    // New state for the currently editing cell
    let editing_cell = use_state(|| None::<String>);
    
    // New state for the cell being edited's input value
    let edit_input_value = use_state(String::new);
    
    // Context menu state
    let context_menu_state = use_state(|| ContextMenuState::new());
    
    // NodeRef for scrolling container
    let container_ref = use_node_ref();
    
    // NodeRef for the cell edit input field
    let cell_input_ref = use_node_ref();
    
    // Handle cell click to select a cell and enable direct editing
    let on_cell_click = {
        let selected_cell = selected_cell.clone();
        let on_cell_select = props.on_cell_select.clone();
        let editing_cell = editing_cell.clone();
        let edit_input_value = edit_input_value.clone();
        let cell_input_ref = cell_input_ref.clone();
        let cell_values = props.cell_values.clone();
        let props_on_formula = props.on_formula.clone();
        let container_ref = container_ref.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation(); // Stop event propagation
            
            let target: HtmlElement = e.target_unchecked_into();
            if let Some(cell_id) = target.get_attribute("data-id") {
                selected_cell.set(Some(cell_id.clone()));
                on_cell_select.emit(cell_id.clone());
                
                // Focus the container to enable keyboard navigation
                if let Some(container) = container_ref.cast::<HtmlElement>() {
                    container.focus().ok();
                }
                
                // Enable editing mode for the clicked cell
                editing_cell.set(Some(cell_id.clone()));
                
                // Get the current value from cell_values map
                let current_value = cell_values.get(&cell_id).cloned().unwrap_or_default();
                
                // If the cell contains a formula (starts with =), show the formula text instead of the result
                let display_value = if current_value.starts_with(&format!("{}=", cell_id)) || current_value.starts_with('=') {
                    // Extract the formula part after the equals sign
                    let formula_parts: Vec<&str> = current_value.splitn(2, '=').collect();
                    if (formula_parts.len() > 1) {
                        formula_parts[1].to_string()
                    } else {
                        current_value
                    }
                } else {
                    current_value
                };
                
                edit_input_value.set(display_value);
                
                // Focus the input field after a short delay to ensure it's rendered
                let cell_input_ref_clone = cell_input_ref.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    // Small delay to ensure the input field is rendered
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let window = web_sys::window().unwrap();
                        window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve, 10).unwrap();
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    
                    // Focus the input field and select all text
                    if let Some(input) = cell_input_ref_clone.cast::<HtmlInputElement>() {
                        input.focus().ok();
                        input.select();
                    }
                });
            }
        })
    };

// Handle cell input changes in the editing cell
    let on_cell_input_change = {
        let edit_input_value = edit_input_value.clone();
        let props_on_formula = props.on_formula.clone();
        let editing_cell = editing_cell.clone();
        let on_cell_select = props.on_cell_select.clone();
        
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            edit_input_value.set(value.clone());
            
            // Check if the input starts with "=" to handle as a formula
            if value.starts_with('=') {
                // When typing a formula directly in a cell, update the formula field in the parent component
                if let Some(cell_id) = (*editing_cell).clone() {
                    on_cell_select.emit(cell_id.clone()); // Update the cell reference field
                    
                    // Only update the formula field in parent, but DON'T apply the formula yet
                    // Use the special prefix "__preview__" to indicate this is just for display
                    props_on_formula.emit(format!("__preview__{}={}", cell_id, &value[1..]));
                }
            }
        })
    };
    
    // Handle keyboard events in the editing cell
    let on_cell_input_keydown = {
        let edit_input_value = edit_input_value.clone();
        let editing_cell = editing_cell.clone();
        let props_on_formula = props.on_formula.clone();
        let container_ref = container_ref.clone();
        
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.stop_propagation();
                e.prevent_default();
                
                if let Some(cell_id) = (*editing_cell).clone() {
                    let value = (*edit_input_value).clone();
                    
                    // If input starts with =, treat as formula but remove the =
                    let formula = if value.starts_with('=') {
                        format!("{}={}", cell_id, &value[1..]) // Remove the = from the start
                    } else {
                        format!("{}={}", cell_id, value)
                    };
                    
                    props_on_formula.emit(formula);
                    
                    // Exit editing mode
                    editing_cell.set(None);
                    
                    // Focus the container to enable keyboard navigation
                    if let Some(container) = container_ref.cast::<HtmlElement>() {
                        container.focus().ok();
                    }
                }
            } else if e.key() == "Escape" {
                e.stop_propagation();
                e.prevent_default();
                
                // Cancel editing
                editing_cell.set(None);
                
                // Focus the container to enable keyboard navigation after pressing Escape
                if let Some(container) = container_ref.cast::<HtmlElement>() {
                    container.focus().ok();
                }
            }
        })
    };
    
    // Handle clicking outside of the editing cell to exit edit mode
    let on_click_outside = {
        let editing_cell = editing_cell.clone();
        
        Callback::from(move |_: MouseEvent| {
            editing_cell.set(None);
        })
    };
    
    // Effect to add global click handler to detect clicks outside the editing cell
    {
        let editing_cell = editing_cell.clone();
        
        use_effect_with_deps(
            move |_| {
                let document = web_sys::window().unwrap().document().unwrap();
                let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                    let target = event.target().unwrap();
                    let target_element = target.dyn_ref::<web_sys::Element>();
                    
                    if let Some(target_el) = target_element {
                        // Check if click is inside an input field or on a cell that's being edited
                        if target_el.tag_name() != "INPUT" && !target_el.has_attribute("data-editing") {
                            editing_cell.set(None);
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                
                document.add_event_listener_with_callback(
                    "click",
                    closure.as_ref().unchecked_ref(),
                ).unwrap();
                
                // Return a cleanup function
                move || {
                    document.remove_event_listener_with_callback(
                        "click",
                        closure.as_ref().unchecked_ref(),
                    ).unwrap();
                    closure.forget(); // Prevent memory leak
                }
            },
            (), // Dependencies
        );
    }

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
        let editing_cell = editing_cell.clone();
        
        Callback::from(move |e: KeyboardEvent| {
            // Skip keyboard navigation if we're currently editing a cell
            if (*editing_cell).is_some() {
                return;
            }
            
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
                        // Start editing on Enter or F2 key
                        "Enter" | "F2" => {
                            e.prevent_default();
                            editing_cell.set(Some(current_cell.clone()));
                        },
                        _ => {}
                    }
                }
            }
        })
    };
    
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
                if (!e.shift_key() || e.delta_x() != 0.0) {
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
            
            // Check if this cell is currently being edited
            let is_editing = editing_cell.as_ref()
                .map_or(false, |id| *id == cell_id);
            
            // Apply appropriate CSS classes
            let cell_class = if is_editing {
                "cell editing"
            } else if is_selected {
                "cell selected highlighted"
            } else {
                "cell"
            };
            
            // Render either the cell value or an input field for editing
            if is_editing {
                html! {
                    <td
                        class={cell_class}
                        data-id={cell_id.clone()}
                        data-editing="true"
                        onclick={|e: MouseEvent| e.stop_propagation()}
                    >
                        <input
                            ref={cell_input_ref.clone()}
                            type="text"
                            value={(*edit_input_value).clone()}
                            oninput={on_cell_input_change.clone()}
                            onkeydown={on_cell_input_keydown.clone()}
                            class="cell-edit-input"
                            style="width: 100%; height: 100%; box-sizing: border-box; border: none; outline: none; font-family: inherit; font-size: inherit;"
                        />
                    </td>
                }
            } else {
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

async fn fetch_message() -> Result<String, String> {
    // Assuming your frontend and backend are served from the same origin
    let resp = reqwest::get("http://localhost:8080/api/hello").await
        .map_err(|e| format!("Failed to send request: {:?}", e))?;
    
    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;
    
    Ok(json["message"].as_str().unwrap_or("No message received").to_string())
}

async fn update_cell_logic(cell_id: &str, val: &str) -> Result<(String, String), String> {
    let client = reqwest::Client::new();
    let params = serde_json::json!({
        "cell": cell_id,
        "val": val
    });
    
    let resp = client.post("http://localhost:8080/api/get_value")
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {:?}", e))?;

    let json: serde_json::Value = resp.json()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))?;

    let x = (json["row"].as_str().unwrap_or("")).to_string();
    let y = (json["val"].as_str().unwrap_or("")).to_string();
    Ok((x, y))
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
    
    // Toast notification state
    let toast_state = use_state(|| ToastProps {
        visible: false,
        message: String::new(),
        is_error: false,
    });

    // Add a counter to force the toast to reset between identical errors
    let toast_counter = use_state(|| 0);

    // Add visualization state
    let visualization_state = use_state(|| VisualizationState {
        visible: false,
        data: Vec::new(),
        title: "Data Analysis".to_string(),
    });
    let statistics_state = use_state(|| false);
    
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

// Handle for analyzing data and showing visualization
    let on_analyze = {
        let cell_values = cell_values.clone();
        let visualization_state = visualization_state.clone();
        
        Callback::from(move |_| {
            let mut visualization_data = Vec::new();
            let cells = (*cell_values).clone();
            // web_sys::console::log_1(&format!("cell_values: {:?}", cells).into());
            
            // Get main chart title from A1
            let main_title = cells.get("A1").cloned().unwrap_or_else(|| "Data Analysis".to_string());
            web_sys::console::log_1(&format!("Main title: {}", main_title).into()); // Log main_title
            
            // Find how many columns have data in the header row (row 1)
            let mut header_columns = Vec::new();
            // Start from column B (index 2)
            for col in 2..=cols {
                let col_label = get_column_label(col);
                let header_cell_id = format!("{}{}", col_label, 1);
                // If the header cell has a value, add it to our list of columns
                if let Some(header_value) = cells.get(&header_cell_id) {
                    if !header_value.trim().is_empty() && header_value != "0" {
                        header_columns.push((col, header_value.clone()));
                    }
                }
            }
            web_sys::console::log_1(&format!("Header columns: {:?}", header_columns).into());
            
            // Find data rows starting from row 2 (index 2) and continue until we find a row where A{i} is empty or "0"
            for row in 2..=rows {
                let row_label_cell = format!("A{}", row);
                if let Some(row_label) = cells.get(&row_label_cell) {
                    // If cell A{i} is empty or "0", we've reached the end of our data series
                    if row_label.trim().is_empty() || row_label == "0" {
                        break;
                    }
                    
                    // This row has a label in column A, process it as a data series
                    let mut row_data = Vec::new();
                    for (col, header) in &header_columns {
                        let cell_id = format!("{}{}", get_column_label(*col), row);
                        if let Some(value_str) = cells.get(&cell_id) {
                            if let Ok(value) = value_str.parse::<f64>() {
                                row_data.push((header.clone(), value));
                            }
                        }
                    }
                    
                    // If we have data for this row, add it to our visualization data
                    if !row_data.is_empty() {
                        visualization_data.push((row_label.clone(), row_data));
                    }
                } else {
                    // No label in column A, we've reached the end of our data series
                    break;
                }
            }
            web_sys::console::log_1(&format!("Visualization data: {:?}", visualization_data).into());
            
            // Only show visualization if we have data
            if !visualization_data.is_empty() {
                visualization_state.set(VisualizationState {
                    visible: true,
                    data: visualization_data,
                    title: main_title,
                });
            }
        })
    };
    let on_toggle_statistics = {
        let statistics_state = statistics_state.clone();
        let cell_values = cell_values.clone();
        
        Callback::from(move |_| {
            // Toggle statistics visibility
            statistics_state.set(!*statistics_state);
        })
    };
    
    // Function to scroll to a specific cell
    let scroll_to_cell = Callback::from(move |cell_id: String| {
        web_sys::console::log_1(&format!("App: Attempting to scroll to cell: {}", cell_id).into());
        
        // Find and scroll to the cell directly using DOM API
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                // Try to find the cell by its data-id attribute
                let selector = format!("td[data-id=\"{}\"]", cell_id);
                if let Ok(Some(element)) = document.query_selector(&selector) {
                    let cell_element = element.dyn_into::<HtmlElement>().unwrap();
                    
                    // Get the container element
                    if let Ok(Some(container)) = document.query_selector(".spreadsheet-container") {
                        let container_element = container.dyn_into::<HtmlElement>().unwrap();
                        
                        // Focus the container to enable keyboard navigation
                        container_element.focus().ok();
                        
                        // Parse the cell ID to get column and row
                        let col_chars: String = cell_id.chars()
                            .take_while(|c| c.is_ascii_alphabetic())
                            .collect();
                        
                        let row_str: String = cell_id.chars()
                            .skip_while(|c| c.is_ascii_alphabetic())
                            .collect();
                        
                        let row = row_str.parse::<i32>().unwrap_or(1);
                        let col_num = get_column_number(&col_chars) as i32;
                        
                        // Use exact dimensions from the CSS file:
                        // From CSS: .cell { min-width: 80px; height: 35px; }
                        // From CSS: .row-header, .column-header are in similar dimensions
                        let cell_width = 80;       // Exact width from CSS
                        let cell_height = 35.11;      // Exact height from CSS
                        let row_header_width = 40; // Width of the row headers (1, 2, 3...)
                        let col_header_height = 35; // Height of the column headers (A, B, C...)
                        
                        // Calculate the exact pixel offset
                        // Subtract header sizes to account for the fixed headers
                        // Start exact at top-left corner of the target cell
                        let scroll_top = ((row - 1) as f64 * cell_height) as i32 ;  // No extra offset needed vertically
                        let scroll_left = ((col_num - 1) * cell_width) + 0; // No extra offset needed horizontally
                        
                        // Set scroll position
                        container_element.set_scroll_top(scroll_top);
                        container_element.set_scroll_left(scroll_left);
                        
                        web_sys::console::log_1(&format!(
                            "Scrolled to cell: {}. Position: left={}, top={}, col={}, row={}", 
                            cell_id, scroll_left, scroll_top, col_num, row
                        ).into());
                    }
                } else {
                    web_sys::console::log_1(&format!("Cell not found in DOM: {}", cell_id).into());
                }
            }
        }
    });
    
    // Generic validation handler with formula processing
    let onkeydown = {
        let fields = fields.clone();
        let messages = messages.clone();
        let cell_values = cell_values.clone();
        let scroll_to_cell = scroll_to_cell.clone();
        let toast_state = toast_state.clone();
        let toast_counter = toast_counter.clone();
        let on_cell_select = on_cell_select.clone();
        
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: HtmlInputElement = e.target_unchecked_into();
                let field_id = input.id();
                let value = input.value();
                
                
                web_sys::console::log_1(&format!("Enter pressed in field: {}, value: {}", field_id, value).into());

                let fields = fields.clone();
                let messages = messages.clone();
                let cell_values = cell_values.clone();
                let scroll_to_cell = scroll_to_cell.clone();
                let toast_state = toast_state.clone();
                let on_cell_select = on_cell_select.clone();
                let toast_counter = toast_counter.clone();
                wasm_bindgen_futures::spawn_local(async move {
                
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
                                if let Some((cell_id, _value)) = process_formula(&formula) {
                                    // Update our local state with the processed formula result
                                    match update_cell_logic(cell_id.as_str(), _value.as_str()).await {
                                        Ok((row_str, val_str)) => {
                                            web_sys::console::log_1(&format!("Fetched value: {:?} {:?}", row_str, val_str).into());
                                            let mut updated = (*cell_values).clone();

                                            let row_parts: Vec<&str> = row_str.split_whitespace().collect();
                                            let val_parts: Vec<&str> = val_str.split_whitespace().collect();

                                            if row_str == "IV" && &_value[0..1] == "\"" && &_value[_value.len()-1..] == "\""  {
                                                updated.insert(cell_id.clone(), _value[1.._value.len()-1].to_string());
                                                web_sys::console::log_1(&format!("Went into IF statement!").into());
                                                cell_values.set(updated);
                                            } else if row_str == "IV" {
                                                toast_state.set(ToastProps {
                                                    visible: false,
                                                    message: String::new(),
                                                    is_error: false,
                                                });
                                                
                                                // Use setTimeout to force the browser to process the state change
                                                let toast_state_clone = toast_state.clone();
                                                let error_message_clone = "Invalid Formula".to_string();
                                                
                                                
                                                let window = web_sys::window().unwrap();
                                                let closure = Closure::once_into_js(move || {
                                                    toast_state_clone.set(ToastProps {
                                                        visible: true,
                                                        message: error_message_clone,
                                                        is_error: true,
                                                    });
                                                });
                                                
                                                window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                                    &closure.into(),
                                                    10, // Very short timeout to ensure it happens in the next event loop
                                                ).unwrap();
                                            } else if row_str == "CY" {
                                                toast_state.set(ToastProps {
                                                    visible: false,
                                                    message: String::new(),
                                                    is_error: false,
                                                });
                                                
                                                // Use setTimeout to force the browser to process the state change
                                                let toast_state_clone = toast_state.clone();
                                                let error_message_clone = "Cycle Detected, Formula Rejected".to_string();
                                                
                                                
                                                let window = web_sys::window().unwrap();
                                                let closure = Closure::once_into_js(move || {
                                                    toast_state_clone.set(ToastProps {
                                                        visible: true,
                                                        message: error_message_clone,
                                                        is_error: true,
                                                    });
                                                });
                                                
                                                window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                                    &closure.into(),
                                                    10, // Very short timeout to ensure it happens in the next event loop
                                                ).unwrap();
                                            }
                                            else {
                                                for i in 0..row_parts.len() {
                                                    updated.insert((row_parts[i].to_string()), val_parts[i].to_string());
                                                }
                                                cell_values.set(updated);
                                                updated_messages.insert(field_id.clone(), 
                                                format!("Formula applied: {}", formula));
                                            }
                                            

                                            
                
                                            on_cell_select.emit(cell_id.clone());
                                            
                                        }
                                        Err(err) => {
                                            ();
                                        }
                                    }
                                }
                            }
                            
                            // If it's a cell input and valid, scroll to that cell
                            if field_id == "cell" {
                                let cell_id = field.value();
                                // Check if the cell is valid (already validated by the field's validate method)
                                web_sys::console::log_1(&format!("Cell input validated, scrolling to: {}", cell_id).into());
                                scroll_to_cell.emit(cell_id.to_string());
                            }
                        },
                        Err(error) => { 
                            // Clone the error before moving it to the updated_messages
                            let error_message = error.clone();
                            updated_messages.insert(field_id.clone(), error); 
                            web_sys::console::log_1(&format!("Validation error: {}", error_message).into());
                            
                            // First, reset the toast to ensure it's hidden
                            toast_state.set(ToastProps {
                                visible: false,
                                message: String::new(),
                                is_error: false,
                            });
                            
                            // Use setTimeout to force the browser to process the state change
                            let toast_state_clone = toast_state.clone();
                            let error_message_clone = error_message.clone();
                            let counter = *toast_counter + 1;
                            toast_counter.set(counter);
                            
                            let window = web_sys::window().unwrap();
                            let closure = Closure::once_into_js(move || {
                                toast_state_clone.set(ToastProps {
                                    visible: true,
                                    message: error_message_clone,
                                    is_error: true,
                                });
                            });
                            
                            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                &closure.into(),
                                10, // Very short timeout to ensure it happens in the next event loop
                            ).unwrap();
                        },
                    }
                    messages.set(updated_messages);
                
                }
              })
            }
        })
    };
    
    // Handle closing visualization
    let on_close_visualization = {
        let visualization_state = visualization_state.clone();
        
        Callback::from(move |_| {
            visualization_state.set(VisualizationState {
                visible: false,
                data: Vec::new(),
                title: "Data Analysis".to_string(),
            });
        })
    };

    let on_close_statistics = {
        let statistics_state = statistics_state.clone();
        
        Callback::from(move |_: ()| {
            statistics_state.set(false);
        })
    };
    let toast_state_clone = toast_state.clone();

    // Render content based on dimensions validity
    let content = if dimensions_valid {
        html! {
            <div class="content-container">
                <Spreadsheet 
                    rows={rows} 
                    cols={cols}
                    on_cell_select={on_cell_select.clone()}
                    cell_values={(*cell_values).clone()}
                    on_formula={
                        let cell_values = cell_values.clone();
                        let on_cell_select = on_cell_select.clone();
                        let scroll_to_cell = scroll_to_cell.clone();
                        let fields = fields.clone();
                        let toast_state = toast_state_clone.clone();
                        
                        Callback::from(move |formula: String| {

                            let cell_values = cell_values.clone();
                            let on_cell_select = on_cell_select.clone();
                            let scroll_to_cell = scroll_to_cell.clone();
                            let fields = fields.clone();
                            let toast_state = toast_state.clone();
                            
                            wasm_bindgen_futures::spawn_local(async move {

                            // Check if this is just a preview update from typing in a cell
                            if formula.starts_with("__preview__") {
                                // This is just to update the formula field in the UI
                                // Extract the actual formula without the preview prefix
                                let actual_formula = formula.trim_start_matches("__preview__");
                                
                                // Update the formula field but don't process it yet
                                let mut updated_fields = (*fields).clone();
                                if let Some(field) = updated_fields.get_mut("formula") {
                                    field.set_value(actual_formula.to_string());
                                    fields.set(updated_fields);
                                }
                            } else {
                                // This is an actual formula submission (Enter was pressed)
                                // Process formula and update state
                                if let Some((cell_id, _value)) = process_formula(&formula) {
                                    match update_cell_logic(cell_id.as_str(), _value.as_str()).await {
                                        Ok((row_str, val_str)) => {
                                            web_sys::console::log_1(&format!("Fetched value: {:?} {:?}", row_str, val_str).into());
                                            let mut updated = (*cell_values).clone();

                                            let row_parts: Vec<&str> = row_str.split_whitespace().collect();
                                            let val_parts: Vec<&str> = val_str.split_whitespace().collect();

                                            if row_str == "IV" && &_value[0..1] == "\"" && &_value[_value.len()-1..] == "\""  {
                                                updated.insert(cell_id.clone(), _value[1.._value.len()-1].to_string());
                                                web_sys::console::log_1(&format!("Went into IF statement!").into());
                                                cell_values.set(updated);
                                            } else if row_str == "IV" {
                                                toast_state.set(ToastProps {
                                                    visible: false,
                                                    message: String::new(),
                                                    is_error: false,
                                                });
                                                
                                                // Use setTimeout to force the browser to process the state change
                                                let toast_state_clone = toast_state.clone();
                                                let error_message_clone = "Invalid Formula".to_string();
                                                
                                                
                                                let window = web_sys::window().unwrap();
                                                let closure = Closure::once_into_js(move || {
                                                    toast_state_clone.set(ToastProps {
                                                        visible: true,
                                                        message: error_message_clone,
                                                        is_error: true,
                                                    });
                                                });
                                                
                                                window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                                    &closure.into(),
                                                    10, // Very short timeout to ensure it happens in the next event loop
                                                ).unwrap();
                                            } else if row_str == "CY" {
                                                toast_state.set(ToastProps {
                                                    visible: false,
                                                    message: String::new(),
                                                    is_error: false,
                                                });
                                                
                                                // Use setTimeout to force the browser to process the state change
                                                let toast_state_clone = toast_state.clone();
                                                let error_message_clone = "Cycle Detected, Formula Rejected".to_string();
                                                
                                                
                                                let window = web_sys::window().unwrap();
                                                let closure = Closure::once_into_js(move || {
                                                    toast_state_clone.set(ToastProps {
                                                        visible: true,
                                                        message: error_message_clone,
                                                        is_error: true,
                                                    });
                                                });
                                                
                                                window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                                    &closure.into(),
                                                    10, // Very short timeout to ensure it happens in the next event loop
                                                ).unwrap();
                                            }
                                            else {
                                                for i in 0..row_parts.len() {
                                                    updated.insert((row_parts[i].to_string()), val_parts[i].to_string());
                                                }
                                                cell_values.set(updated);
                                            }

                                            
                
                                            on_cell_select.emit(cell_id.clone());
                                            
                                        }
                                        Err(err) => {
                                            ();
                                        }
                                    }
                                }
                            }
                        })
                        })
                    }
                    on_scroll_to_cell_ref={Callback::from(move |_| {
                        // We're not using this anymore, but keep it in props for now
                    })}
                />
                
                // <div class="api-info">
                //     <p class="note">{"Note: In the future, this spreadsheet will connect to a backend API to process formulas and update cell values."}</p>
                //     <p class="instructions">{"To use the spreadsheet: Click on a cell to select it, then enter a formula like \"A1=10+B2\" and press Enter."}</p>
                //     <p class="instructions">{"You can also type a cell reference (like A1) in the \"Enter cell\" field and press Enter to quickly navigate to that cell."}</p>
                // </div>
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

    fn assign_colors(data_len: usize, palette: Vec<String>) -> Vec<String> {
        (0..data_len).map(|i| palette[i % palette.len()].clone()).collect()
    }

    

        
    
    // Rest of the code remains the same
    html! {
        <>
            <h1> {"Rusty Spreadsheet"} </h1>
            
            <div class="navigation-bar">
                <div class="nav-section">
                    {
                        if let Some(cell_field) = (*fields).get("cell") {
                            html! {
                                <div class="input-field">
                                    <label for={cell_field.id().to_string()}> {format!("{}: ", cell_field.label())} </label>
                                    <input
                                        id={cell_field.id().to_string()}
                                        type={cell_field.input_type().to_string()}
                                        value={cell_field.value().to_string()}
                                        min={cell_field.min().map(|s| s.to_string())}
                                        max={cell_field.max().map(|s| s.to_string())}
                                        oninput={oninput.clone()}
                                        onkeydown={onkeydown.clone()}
                                        autofocus={true} 
                                    />
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }

                    {
                        if let Some(formula_field) = (*fields).get("formula") {
                            html! {
                                <div class="input-field">
                                    <label for={formula_field.id().to_string()}> {format!("{}: ", formula_field.label())} </label>
                                    <input
                                        id={formula_field.id().to_string()}
                                        type={formula_field.input_type().to_string()}
                                        value={formula_field.value().to_string()}
                                        min={formula_field.min().map(|s| s.to_string())}
                                        max={formula_field.max().map(|s| s.to_string())}
                                        oninput={oninput.clone()}
                                        onkeydown={onkeydown.clone()}
                                    />
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                    
                    // Spacer to push buttons to the right
                    <div style="flex-grow: 1;"></div>
                    
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
                                let element = element.dyn_into::<HtmlElement>().unwrap();
                                
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
                        class="nav-button save-button"
                    >
                        {"Save"}
                    </button>
                    // ...inside your html! for the navigation bar...
                    <button 
                    class="nav-button save-csv-button"
                    onclick={
                        let cell_values = cell_values.clone();
                        Callback::from(move |_| {
                            // Convert cell_values to CSV and save as file
                            let mut csv_data = String::new();

                            // Collect all cell keys and sort them
                            let mut keys: Vec<_> = cell_values.keys().collect();
                            keys.sort();

                            // Find max row and col
                            let mut max_row = 1;
                            let mut max_col = 1;
                            for key in &keys {
                                let col: String = key.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
                                let row: String = key.chars().skip_while(|c| c.is_ascii_alphabetic()).collect();
                                let col_num = get_column_number(&col);
                                let row_num = row.parse::<u32>().unwrap_or(1);
                                if col_num > max_col { max_col = col_num; }
                                if row_num > max_row { max_row = row_num; }
                            }

                            // Write CSV rows
                            for row in 1..=max_row {
                                let mut row_cells = Vec::new();
                                for col in 1..=max_col {
                                    let cell_id = format!("{}{}", get_column_label(col), row);
                                    let val = cell_values.get(&cell_id).cloned().unwrap_or_default();
                                    // Escape double quotes
                                    let val = val.replace('"', "\"\"");
                                    // Wrap in quotes if needed
                                    let val = if val.contains(',') || val.contains('"') { format!("\"{}\"", val) } else { val };
                                    row_cells.push(val);
                                }
                                csv_data.push_str(&row_cells.join(","));
                                csv_data.push('\n');
                            }

                            // Use web_sys to create and download a file
                            let window = web_sys::window().unwrap();
                            let document = window.document().unwrap();
                            let element = document.create_element("a").unwrap();
                            let element = element.dyn_into::<web_sys::HtmlElement>().unwrap();

                            // Create a blob URL for the CSV data
                            let mut binding = web_sys::BlobPropertyBag::new();
                            let blob_props = binding.type_("text/csv");
                            let blob = web_sys::Blob::new_with_str_sequence_and_options(
                                &js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(&csv_data)),
                                &blob_props
                            ).unwrap();
                            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

                            // Set up the download link
                            element.set_attribute("href", &url).unwrap();
                            element.set_attribute("download", "spreadsheet.csv").unwrap();
                            element.style().set_css_text("display: none");

                            // Append to document, click, and clean up
                            let body = document.body().unwrap();
                            body.append_child(&element).unwrap();
                            element.click();
                            body.remove_child(&element).unwrap();
                            web_sys::Url::revoke_object_url(&url).unwrap();
                        })
                    }
                    >
                    {"Download CSV"}
                    </button>
                    // Completely hide the default file input and use only the custom button
                    <label for="spreadsheet-upload" class="upload-button">{"Upload"}</label>
                    <input
                        id="spreadsheet-upload"
                        type="file"
                        accept=".json"
                        // native file input is hidden via CSS
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
                                            
                                            // Parse JSON to HashMap
                                            match serde_json::from_str::<HashMap<String, String>>(&json_str) {
                                                Ok(loaded_data) => {
                                                    // Update cell values state with loaded data
                                                    cell_values_clone.set(loaded_data);
                                                },
                                                Err(err) => {
                                                    web_sys::console::error_1(&format!("Error parsing JSON: {}", err).into());
                                                    // Optionally display error to user
                                                }
                                            }
                                        }) as Box<dyn FnMut(Event)>);
                                        
                                        // Attach event listener and trigger file read
                                        file_reader.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
                                        file_reader.read_as_text(&file).unwrap();
                                        
                                        // Keep callback alive
                                        onload_callback.forget();
                                    }
                                }
                            })
                        }
                    />
                    
                    <button 
                        class="nav-button graph-button"
                        onclick={on_analyze}
                    >
                        {"Analyze"}
                    </button>
                    <button
                        class="nav-button stats-button"
                        onclick={on_toggle_statistics}
                    >
                        {"Statistics"}
                    </button>
                </div>
            </div>
            
            {content}
            
            // Render the Toast component
            <Toast
                visible={toast_state.visible}
                message={toast_state.message.clone()}
                is_error={toast_state.is_error}
                key={*toast_counter} // Add key prop with the counter to force re-render
            />

            // Render visualization if visible
            { if visualization_state.visible {
                let data = &visualization_state.data;
                let colors1 = assign_colors(data.len(), vec![
                                                                "#3498db".to_string(), 
                                                                "#2ecc71".to_string(), 
                                                                "#e74c3c".to_string(),
                                                                "#f1c40f".to_string(), 
                                                                "#9b59b6".to_string()
                                                            ]);
                html! {
                    <div class="visualization-container">
                        <div class="visualization-panel">
                            <div class="visualization-header">
                                <h2>{&visualization_state.title}</h2>
                                <button class="visualization-close" onclick={on_close_visualization}>{"×"}</button>
                            </div>
                            <div class="charts-container">
                                {
                                    // Dynamically generate chart sections for all series in the order they appear in the Vec
                                    visualization_state.data.iter().enumerate().map(|(index, (title, data))| {
                                        // Alternate colors for different charts
                                        let chart_color = match index % 3 {
                                            0 => "#3498db", // Blue
                                            1 => "#2ecc71", // Green
                                            _ => "#e74c3c", // Red
                                        };
                                        
                                        html! {
                                            <div class="chart-section">
                                                <h3>{title}</h3>
                                                <div class="chart-row">
                                                    <div class="chart-column">
                                                        <h4>{"Bar Chart"}</h4>
                                                        <div class="chart-container">
                                                            <div class="bar-chart">
                                                                { 
                                                                    {
                                                                        // Calculate max value outside html! macro
                                                                        let max_value = data.iter()
                                                                            .map(|(_, value)| *value)
                                                                            .fold(0.0_f64, |a, b| a.max(b));
                                                                        
                                                                        // Generate bars inside a block expression
                                                                        data.iter().map(move |(label, value)| {
                                                                            let height_percent = if max_value > 0.0 { (value / max_value) * 100.0 } else { 0.0 };
                                                                            
                                                                            html! {
                                                                                <div class="bar" style={format!("height: {}%; background-color: {};", height_percent, chart_color)}>
                                                                                    <div class="bar-value">{format!("{:.1}", value)}</div>
                                                                                    <div class="bar-label">{label.clone()}</div>
                                                                                </div>
                                                                            }
                                                                        }).collect::<Html>()
                                                                    }
                                                                }
                                                            </div>
                                                        </div>
                                                    </div>
                                                    
                                                    <div class="chart-column">
                                                        <h4>{"Line Chart"}</h4>
                                                        <div class="chart-container">
                                                            <LineChart 
                                                                label={title.clone()} 
                                                                data={data.clone()} 
                                                                color={chart_color.to_string()} 
                                                            />
                                                        </div>
                                                    </div>

                                                    <div class="chart-column">
                                                        <h4>{"Pie Chart"}</h4>
                                                        <div class="chart-container">
                                                            <PieChart 
                                                                title={title.clone()}
                                                                data={data.clone()}
                                                                colors={colors1.clone()}
                                                                radius={100.0}
                                                            />
                                                        </div>
                                                    </div>
                                                    
                                                    <div class="chart-column">
                                                        <h4>{"Heat Map"}</h4>
                                                        <div class="chart-container">
                                                            <HeatMap 
                                                                title={title.clone()}
                                                                data={data.clone()}
                                                                color_scale={vec![
                                                                    "#ffffcc".to_string(),
                                                                    "#c7e9b4".to_string(),
                                                                    "#7fcdbb".to_string(),
                                                                    "#41b6c4".to_string(),
                                                                    "#1d91c0".to_string(),
                                                                    "#225ea8".to_string(),
                                                                    "#0c2c84".to_string(),
                                                                ]}
                                                                width={220}
                                                                height={220}
                                                            />
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
            { if *statistics_state {
                html! {
                    <StatisticsView
                        data={(*cell_values).clone()}
                        onclose={on_close_statistics}
                    />
                }
            } else {
                html! {}
            }}
        </>
    }
}

#[function_component(Toast)]
fn toast(props: &ToastProps) -> Html {
    // Create internal state to track visibility
    let visible = use_state(|| props.visible);
    let message = use_state(|| props.message.clone());
    let is_error = use_state(|| props.is_error);
    
    // Add a timestamp state to track when the message was last updated
    let timestamp = use_state(|| js_sys::Date::now());
    
    // Update the internal state whenever props change
    {
        let visible = visible.clone();
        let message = message.clone();
        let is_error = is_error.clone();
        let timestamp = timestamp.clone();
        
        use_effect_with_deps(
            move |(new_visible, new_message, new_is_error)| {
                // Don't create a default cleanup closure, instead initialize with None
                let mut cleanup_fn = None;
                
                // Always show a new toast when props.visible is true, regardless if the message is the same
                if *new_visible {
                    visible.set(true);
                    message.set(new_message.clone());
                    is_error.set(*new_is_error);
                    // Always update timestamp to force re-render even for identical messages
                    timestamp.set(js_sys::Date::now());
                    
                    // Auto-hide toast after 3 seconds
                    let visible_clone = visible.clone();
                    let window = web_sys::window().unwrap();
                    let timeout_id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        &Closure::once_into_js(move || {
                            visible_clone.set(false);
                        }).into(),
                        3000,
                    ).unwrap();
                    
                    // Create the timeout cleanup closure of the expected type
                    cleanup_fn = Some(Box::new(move || {
                        window.clear_timeout_with_handle(timeout_id);
                    }) as Box<dyn FnOnce()>);
                } else if !*new_visible && *visible {
                    // Handle when toast should be hidden
                    visible.set(false);
                }
                
                // Return the cleanup function or a no-op function of the right type
                match cleanup_fn {
                    Some(cleanup) => cleanup,
                    None => Box::new(|| {}) as Box<dyn FnOnce()>
                }
            },
            (props.visible, props.message.clone(), props.is_error),
        );
    }
    
    let toast_class = if *visible {
        if *is_error {
            "toast toast-visible toast-error"
        } else {
            "toast toast-visible toast-success"
        }
    } else {
        "toast toast-hidden"
    };
    
    html! {
        <div class={toast_class} key={(*timestamp).to_string()}>
            {(*message).clone()}
        </div>
    }
}

// Bar Chart Component Props
#[derive(Properties, PartialEq, Clone)]
struct BarChartProps {
    label: String,
    data: Vec<(String, f64)>,
    color: String,
}

// Bar Chart Component
#[function_component(BarChart)]
fn bar_chart(props: &BarChartProps) -> Html {
    let max_value = props.data.iter()
        .map(|(_, value)| *value)
        .fold(0.0_f64, |a, b| a.max(b));
    
    let chart_height = 300.0;
    let bar_width = 80;
    let gap = 20;
    let chart_width = (bar_width + gap) * props.data.len();
    
    html! {
        <div class="chart-container">
            <h3>{ &props.label }</h3>
            <div class="chart" style={format!("height: {}px; width: {}px;", chart_height, chart_width)}>
                {
                    props.data.iter().enumerate().map(|(index, (label, value))| {
                        let height_percent = if max_value > 0.0 { (value / max_value) * 100.0 } else { 0.0 };
                        let bar_height = (height_percent / 100.0) * chart_height;
                        let position = index * (bar_width + gap);
                        
                        html! {
                            <div class="chart-bar-container" style={format!("left: {}px; width: {}px;", position, bar_width)}>
                                <div class="chart-bar" 
                                     style={format!("height: {}px; background-color: {};", 
                                                  bar_height, props.color)}>
                                </div>
                                <div class="chart-value">{format!("{:.1}", value)}</div>
                                <div class="chart-label">{label}</div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </div>
    }
}

// Line Chart Component Props - Similar to BarChartProps
#[derive(Properties, PartialEq, Clone)]
struct LineChartProps {
    label: String,
    data: Vec<(String, f64)>,
    color: String,
}

// Line Chart Component
#[function_component(LineChart)]
fn line_chart(props: &LineChartProps) -> Html {
    // Find the maximum value for scaling
    let max_value = props.data.iter()
        .map(|(_, value)| *value)
        .fold(0.0_f64, |a, b| a.max(b));
    
    // Chart dimensions
    let chart_height = 300.0;
    let chart_width = 500;
    let padding_top = 40.0;
    let padding_bottom = 40.0;
    let padding_left = 40;
    let padding_right = 20;
    
    // If no data, return empty container
    if props.data.is_empty() {
        return html! {
            <div class="chart-container">
                <h3>{ &props.label }</h3>
                <div class="line-chart-empty">{"No data available"}</div>
            </div>
        };
    }
    
    // Calculate the usable area
    let usable_width = chart_width - padding_left - padding_right;
    let usable_height = chart_height - padding_top - padding_bottom;
    
    // Calculate horizontal spacing between points
    let point_spacing = if props.data.len() > 1 {
        usable_width as f64 / (props.data.len() - 1) as f64
    } else {
        usable_width as f64 / 2.0 // Center single point
    };
    
    // Generate SVG path for the line
    let path_points = props.data.iter().enumerate().map(|(i, (_, value))| {
        let x = padding_left as f64 + (i as f64 * point_spacing);
        let y_ratio = if max_value > 0.0 { value / max_value } else { 0.0 };
        let y = (chart_height - padding_bottom) - (y_ratio * usable_height as f64);
        format!("{},{}", x, y)
    }).collect::<Vec<String>>().join(" L ");
    
    let path_d = if !path_points.is_empty() {
        format!("M {}", path_points)
    } else {
        String::new()
    };
    
    // Calculate grid lines
    let grid_lines = (0..5).map(|i| {
        let y_pos = chart_height - padding_bottom - (i as f64 / 4.0) * usable_height as f64;
        let value = (i as f64 / 4.0) * max_value;
        (y_pos, value)
    }).collect::<Vec<(f64, f64)>>();
    
    html! {
        <div class="chart-container">
            <h3>{ &props.label }</h3>
            <svg class="line-chart" viewBox={format!("0 0 {} {}", chart_width, chart_height)}>
                // Grid lines
                {
                    grid_lines.iter().map(|(y_pos, value)| {
                        html! {
                            <>
                                <line 
                                    x1={padding_left.to_string()} y1={y_pos.to_string()} 
                                    x2={(chart_width - padding_right).to_string()} y2={y_pos.to_string()} 
                                    stroke="#eee" stroke-width="1" stroke-dasharray="4,4" 
                                />
                                <text 
                                    x={(padding_left - 5).to_string()} y={y_pos.to_string()} 
                                    text-anchor="end" 
                                    dominant-baseline="middle"
                                    font-size="10" 
                                    fill="#777"
                                >
                                    {format!("{:.1}", value)}
                                </text>
                            </>
                        }
                    }).collect::<Html>()
                }
                
                // X and Y axes
                <line 
                    x1={padding_left.to_string()} y1={(chart_height - padding_bottom).to_string()} 
                    x2={(chart_width - padding_right).to_string()} y2={(chart_height - padding_bottom).to_string()} 
                    stroke="#aaa" stroke-width="1.5" 
                />
                <line 
                    x1={padding_left.to_string()} y1={padding_top.to_string()} 
                    x2={padding_left.to_string()} y2={(chart_height - padding_bottom).to_string()} 
                    stroke="#aaa" stroke-width="1.5" 
                />
                
                // The line path with animated dash stroke
                <path 
                    d={path_d.clone()} 
                    fill="none" 
                    stroke={props.color.clone()} 
                    stroke-width="2.5" 
                    stroke-linejoin="round"
                    stroke-linecap="round"
                    style="animation: dash 1.5s ease-in-out forwards;"
                />
                
                // Area fill under the line (with transparency)
                <path 
                    d={format!("{} L {},{} L {},{} Z", 
                        path_d,
                        padding_left as f64 + ((props.data.len() - 1) as f64 * point_spacing), 
                        chart_height - padding_bottom,
                        padding_left,
                        chart_height - padding_bottom
                    )} 
                    fill={format!("{}40", props.color)} // 40 is hex for 25% opacity
                />
                
                // Data points and labels
                {props.data.iter().enumerate().map(|(i, (label, value))| {
                    let x = padding_left as f64 + (i as f64 * point_spacing);
                    let y_ratio = if max_value > 0.0 { value / max_value } else { 0.0 };
                    let y = (chart_height - padding_bottom) - (y_ratio * usable_height as f64);
                    
                    html! {
                        <>
                            // Point circle
                            <circle 
                                cx={x.to_string()} cy={y.to_string()} 
                                r="4" fill="white" stroke={props.color.clone()} stroke-width="2" 
                            />
                            
                            // Value label above point
                            <text 
                                x={x.to_string()} y={(y - 10.0).to_string()} 
                                text-anchor="middle" 
                                font-size="12" 
                                fill="#333"
                            >
                                {format!("{:.1}", value)}
                            </text>
                            
                            // X-axis label
                            <text 
                                x={x.to_string()} y={(chart_height - padding_bottom + 15.0).to_string()} 
                                text-anchor="middle" 
                                font-size="11" 
                                fill="#555"
                                transform={format!("rotate(45, {}, {})", x, chart_height - padding_bottom + 5.0)}
                            >
                                {label.clone()}
                            </text>
                        </>
                    }
                }).collect::<Html>()}
            </svg>
        </div>
    }
}

#[function_component(PieChart)]
fn pie_chart(props: &PieChartProps) -> Html {
    use ordered_float::OrderedFloat;

    // Build the counts map
    let mut counts = std::collections::HashMap::new();
    for (_, value) in props.data.iter() {
        let key = OrderedFloat(*value); // Wrap f64 in OrderedFloat
        *counts.entry(key).or_insert(0) += 1;
    }

    // Check if there's only one distinct value
    if counts.len() == 1 {
        // Special case: single value (100% fill)
        let (single_value, single_count) = counts.iter().next().unwrap();
        html! {
            <div class="chart-container">
                <h3>{ &props.title }</h3>
                <svg class="pie-chart" viewBox="0 0 300 300">
                    <path
                        d="M 150,150 L 150,0 A 150,150 0 1 1 149.99,0 Z"
                        fill={props.colors[0].clone()}
                    />
                    <text
                        x="150"
                        y="150"
                        text-anchor="middle"
                        font-size="12"
                        fill="#333"
                    >
                        {format!("{} ({:.1}%)", single_value, 100.0)}
                    </text>
                </svg>
            </div>
        }
    } else {
        // Calculate the total count
        let total_count: usize = counts.values().sum();

        // Calculate the pie chart angles
        let mut angles = Vec::new();
        for (value, count) in counts.iter() {
            let angle = (*count as f64 / total_count as f64) * 360.0;
            angles.push((value, angle));
        }

        html! {
            <div class="chart-container">
                <h3>{ &props.title }</h3>
                <svg class="pie-chart" viewBox="0 0 300 300">
                    // Generate pie chart slices
                    {
                        angles.iter().enumerate().map(|(i, (value, angle))| {
                            // Start angle is cumulative of all previous angles
                            let start_angle = if i == 0 { 0.0 } else { angles[..i].iter().map(|(_, a)| *a).sum::<f64>() };
                            let end_angle = start_angle + angle;
                    
                            // Convert angles to radians for trigonometric functions
                            let start_radians = start_angle.to_radians();
                            let end_radians = end_angle.to_radians();
                    
                            // Compute SVG coordinates for the start and end points of the arc
                            let x1 = 150.0 + 150.0 * start_radians.cos();
                            let y1 = 150.0 + 150.0 * start_radians.sin();
                            let x2 = 150.0 + 150.0 * end_radians.cos();
                            let y2 = 150.0 + 150.0 * end_radians.sin();
                    
                            // Determine if this arc is greater than 180 degrees
                            let large_arc = if *angle > 180.0 { 1 } else { 0 };
                    
                            html! {
                                <path
                                    d={format!("M 150,150 L {},{} A 150,150 0 {} 1 {},{} Z", x1, y1, large_arc, x2, y2)}
                                    fill={props.colors[i % props.colors.len()].clone()}
                                />
                            }
                        }).collect::<Html>()
                    }
                    
                    // Add labels to the pie slices
                    {
                        angles.iter().enumerate().map(|(i, (value, angle))| {
                            let mid_angle = angles[..i].iter().map(|(_, a)| *a).sum::<f64>() + (angle / 2.0);
                            let x = 150.0 + (120.0 * (mid_angle / 360.0 * 2.0 * std::f64::consts::PI).cos());
                            let y = 150.0 + (120.0 * (mid_angle / 360.0 * 2.0 * std::f64::consts::PI).sin());

                            html! {
                                <text
                                    x={x.to_string()}
                                    y={y.to_string()}
                                    text-anchor="middle"
                                    font-size="12"
                                    fill="#333"
                                >
                                    {format!("{} ({:.1}%)", value, angle / 360.0 * 100.0)}
                                </text>
                            }
                        }).collect::<Html>()
                    }
                </svg>
            </div>
        }
    }
}



// Pie Chart Props
#[derive(Properties, PartialEq)]
struct PieChartProps {
    title: String,
    // row_label: String,
    data: Vec<(String, f64)>,
    colors: Vec<String>,
    #[prop_or(150.0)]
    radius: f64,
}

// Visualization Component Props
#[derive(Properties, PartialEq, Clone)]
struct VisualizationProps {
    data: HashMap<String, Vec<(String, f64)>>,
    #[prop_or_default]
    onclose: Callback<()>,
}

// Visualization Component that shows all charts
#[function_component(Visualization)]
fn visualization(props: &VisualizationProps) -> Html {
    let house_data = props.data.get("house").cloned().unwrap_or_default();
    let eat_data = props.data.get("eat").cloned().unwrap_or_default();
    
    html! {
        <div class="visualization-overlay">
            <div class="visualization-container">
                <div class="visualization-header">
                    <h2>{"Data Visualization"}</h2>
                    <button class="close-button" onclick={props.onclose.reform(|_| ())}>{"×"}</button>
                </div>
                <div class="charts-container">
                    <BarChart label="House Data" data={house_data} color="#3498db" />
                    <BarChart label="Eat Data" data={eat_data} color="#2ecc71" />
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct HeatMapProps {
    title: String,
    data: Vec<(String, f64)>,
    color_scale: Vec<String>,
    #[prop_or(300)]
    width: usize,
    #[prop_or(300)]
    height: usize,
}

// HeatMap Component
#[function_component(HeatMap)]
fn heat_map(props: &HeatMapProps) -> Html {
    // Find min and max values for color scaling
    let min_value = props.data.iter()
        .map(|(_, value)| *value)
        .fold(f64::INFINITY, |a, b| a.min(b));
        
    let max_value = props.data.iter()
        .map(|(_, value)| *value)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    
    // Determine grid dimensions - try to make it roughly square
    let total_cells = props.data.len();
    let grid_width = (total_cells as f64).sqrt().ceil() as usize;
    let grid_height = (total_cells as f64 / grid_width as f64).ceil() as usize;
    
    // Cell size calculation
    let cell_width = props.width / grid_width;
    let cell_height = props.height / grid_height;
    let font_size = (cell_width.min(cell_height) / 4).max(8);
    
    // If no data, return empty container
    if props.data.is_empty() {
        return html! {
            <div class="chart-container">
                <h3>{ &props.title }</h3>
                <div class="heatmap-empty">{"No data available"}</div>
            </div>
        };
    }
    
    html! {
        <div class="chart-container">
            <h3>{ &props.title }</h3>
            <div class="heat-map" style={format!("width: {}px; height: {}px;", props.width, props.height)}>
                {
                    props.data.iter().enumerate().map(|(index, (label, value))| {
                        let row = index / grid_width;
                        let col = index % grid_width;
                        
                        // Normalize value to 0-1 range for color interpolation
                        let normalized_value = if max_value > min_value {
                            (value - min_value) / (max_value - min_value)
                        } else {
                            0.5 // If all values are the same
                        };
                        
                        // Color selection based on normalized value
                        let color_index = (normalized_value * (props.color_scale.len() - 1) as f64).round() as usize;
                        let color = &props.color_scale[color_index];
                        
                        // Text color - use white for dark backgrounds, black for light ones
                        let is_dark = color_index > props.color_scale.len() / 2;
                        let text_color = if is_dark { "#ffffff" } else { "#333333" };
                        
                        html! {
                            <div class="heat-map-cell" 
                                 style={format!("width: {}px; height: {}px; top: {}px; left: {}px; background-color: {}; color: {};",
                                              cell_width - 2, cell_height - 2, 
                                              row * cell_height, col * cell_width,
                                              color, text_color)}>
                                <div class="heat-map-value" style={format!("font-size: {}px;", font_size)}>
                                    {format!("{:.1}", value)}
                                </div>
                                <div class="heat-map-label" style={format!("font-size: {}px;", (font_size as f64 * 0.8) as usize)}>
                                    {label.clone()}
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
            <div class="heat-map-legend">
                <div class="heat-map-legend-min">{format!("{:.1}", min_value)}</div>
                <div class="heat-map-legend-gradient">
                    {
                        props.color_scale.iter().map(|color| {
                            html! {
                                <div class="heat-map-legend-color" style={format!("background-color: {};", color)}></div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="heat-map-legend-max">{format!("{:.1}", max_value)}</div>
            </div>
        </div>
    }
}


// StatisticsView Component Props
#[derive(Properties, PartialEq)]
struct StatisticsViewProps {
    data: HashMap<String, String>,  // Changed from HashMap<String, Vec<(String, f64)>>
    #[prop_or_default]
    onclose: Callback<()>,
}

#[function_component(StatisticsView)]
fn statistics_view(props: &StatisticsViewProps) -> Html {
    // Parse data from cell_values
    let all_data = &props.data;
    
    // Extract numeric values and labels for financial data
    let mut financial_data: HashMap<String, f64> = HashMap::new();
    let mut income_sources: Vec<(String, f64)> = Vec::new();
    let mut expenses: Vec<(String, f64)> = Vec::new();
    
    // Extract main title from A1
    let title = all_data.get("A1").cloned().unwrap_or_else(|| "Personal Financial Analysis".to_string());
    
    // Process data rows (assuming similar structure to visualization data)
    for row in 2..=3 {  // Check first 20 rows
        let row_label_cell = format!("A{}", row);
        if let Some(row_label) = all_data.get(&row_label_cell) {
            if row_label.trim().is_empty() || row_label == "0" {
                continue;
            }
            
            // Process data in this row (columns B-I)
            for col in 2..=10 {
                let column_label_cell = format!("{}1", get_column_label(col));
                if let Some(col_label) = all_data.get(&column_label_cell) {
                    
                    let cell_id = format!("{}{}", get_column_label(col), row);
                    if let Some(value_str) = all_data.get(&cell_id) {
                        if let Ok(value) = value_str.parse::<f64>() {
                            if value != 0.0 {
                                if row == 2 {
                                    // First few rows might be income
                                    income_sources.push((col_label.clone(), value));
                                } else {
                                    // Later rows likely expenses
                                    expenses.push((col_label.clone(), value));
                                }
                                
                                financial_data.insert(col_label.clone(), value);
                            }
                        }
                    }
                }
            }
        }
    }

    // Build savings using header values as keys
    let mut savings: Vec<(String, f64)> = Vec::new();
    for col in 2..=10 {
        let col_label = get_column_label(col);
        let header = all_data.get(&format!("{}1", col_label)).cloned().unwrap_or_default();
        if header.is_empty() { continue; }
        let income = income_sources.iter().find(|(label, _)| label == &header).map(|(_, v)| *v).unwrap_or(0.0);
        let expense = expenses.iter().find(|(label, _)| label == &header).map(|(_, v)| *v).unwrap_or(0.0);
        let saving = income - expense;
        if income != 0.0 || expense != 0.0 {
            savings.push((header, saving));
        }
    }
    
    // Sort expenses by value for top expenses
    expenses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Calculate financial summary
    let total_income: f64 = income_sources.iter().map(|(_, value)| *value).sum();
    let total_expenses: f64 = expenses.iter().map(|(_, value)| *value).sum();
    let net_amount = total_income - total_expenses;
    
    html! {
        <div class="statistics-overlay">
            <div class="statistics-container">
                <header>
                    <div class="logo">
                        <i class="fas fa-chart-line logo-icon"></i>
                        <span>{"Finance"}</span>
                    </div>
                    <h1 class="statistics-title">{title}</h1>
                    <div class="date-container">
                        <button class="close-button" onclick={props.onclose.reform(|_| ())}>{"×"}</button>
                    </div>
                </header>
                
                <div class="dashboard">
                    <div class="card expenses-by-category">
                        <h2 class="card-title">{"Expenses"}</h2>
                        <div class="chart-container">
                            <div id="expenses-chart" class="chart-placeholder">
                                {
                                    // let max_value = expenses.iter().map(|(_, v)| *v).fold(0.0, f64::max);
                                    expenses.iter().take(6).map(|(category, value)| {
                                        let max_value = expenses.iter().map(|(_, v)| *v).fold(0.0, f64::max);
                                        let width_percent = if max_value > 0.0 { (value / max_value) * 100.0 } else { 0.0 };
                                        
                                        html! {
                                            <div class="chart-bar-row">
                                                <span class="chart-bar-label">{category}</span>
                                                <div class="chart-bar-outer">
                                                    <div class="chart-bar-value" style={format!("width: {}%;", width_percent)}></div>
                                                </div>
                                                <span class="chart-bar-amount">{format!("${:.2}", value)}</span>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        </div>
                    </div>

                    <div class="card top-expenses">
                        <h2 class="card-title">{"Top 5 Expenses"}</h2>
                        <div>
                            {
                                expenses.iter().take(5).map(|(category, value)| {
                                    let max_value = expenses.iter().take(5).map(|(_, v)| *v).fold(0.0, f64::max);
                                    let width_percent = if max_value > 0.0 { (value / max_value) * 100.0 } else { 0.0 };
                                    
                                    html! {
                                        <div class="expense-bar">
                                            <div class="expense-bar-label">{category}</div>
                                            <div class="expense-bar-value" style={format!("width: {}%;", width_percent)}></div>
                                            <div class="expense-bar-amount">{format!("${:.2}", value)}</div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    </div>

                    <div class="card net-amount">
                        <h2 class="card-title">{"Financial Summary"}</h2>
                        <div class="finance-summary">
                            <div class="summary-column" style="background-color: var(--primary);">
                                <div class="summary-amount">{format!("${:.2}", net_amount)}</div>
                                <div class="summary-label">{"Net Amount"}</div>
                            </div>
                            <div class="summary-column" style="background-color: #7f8c8d;">
                                <div class="summary-amount">{format!("${:.2}", total_income)}</div>
                                <div class="summary-label">{"Credit"}</div>
                            </div>
                            <div class="summary-column" style="background-color: #e74c3c;">
                                <div class="summary-amount">{format!("${:.2}", total_expenses)}</div>
                                <div class="summary-label">{"Debit"}</div>
                            </div>
                        </div>
                    </div>

                    <div class="card cashflow">
                        <h2 class="card-title">{"CashFlow (Income)"}</h2>
                        <div class="donut-chart-container">
                            <div class="donut-chart" style={format!("--percentage: {};", if total_income + total_expenses > 0.0 { (total_income / (total_income + total_expenses)) * 100.0 } else { 50.0 })}></div>
                            <div class="donut-label">{format!("{:.0}%", if total_income + total_expenses > 0.0 { (total_income / (total_income + total_expenses)) * 100.0 } else { 0.0 })}</div>
                        </div>
                        {
                            income_sources.iter().take(3).map(|(source, value)| {
                                html! {
                                    <div class="income-source">
                                        <div>{source}</div>
                                        <div>{format!("${:.2}", value)}</div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    
                    // Expense cards - use expense data from spreadsheet
                    // {
                    //     expenses.iter().take(6).enumerate().map(|(i, (category, value))| {
                    //         // Assign different colors and icons based on category types
                    //         let (bg_color, icon) = match i % 6 {
                    //             0 => ("#ff7675", "fas fa-home"),
                    //             1 => ("#34495e", "fas fa-utensils"),
                    //             2 => ("#34495e", "fas fa-hand-holding-heart"),
                    //             3 => ("#34495e", "fas fa-briefcase-medical"),
                    //             4 => ("#34495e", "fas fa-shopping-bag"),
                    //             _ => ("#6c5ce7", "fas fa-car"),
                    //         };
                            
                    //         html! {
                    //             <div class="expense-card" style={format!("background-color: {}", bg_color)}>
                    //                 <i class={format!("{} expense-card-icon", icon)}></i>
                    //                 <div class="expense-card-title">{category}</div>
                    //                 <div class="expense-card-amount">{format!("${:.2}", value)}</div>
                    //             </div>
                    //         }
                    //     }).collect::<Html>()
                    // }
                    <div class="card weekly-savings">
                        <h2 class="card-title">{"Weekly Savings"}</h2>
                        <div style="display: flex; flex-direction: column; gap: 10px; align-items: flex-start;">
                            {
                                savings.iter().map(|(label, value)| {
                                    html! {
                                        <div style="background: #27ae60; color: #fff; padding: 8px 18px; border-radius: 6px; font-weight: bold;">
                                            {format!("{}: ${:.2}", label, value)}
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}


fn main(){
    yew::Renderer::<App>::new().render();   
}