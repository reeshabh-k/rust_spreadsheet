use yew::prelude::*;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent, HtmlElement, MouseEvent, WheelEvent, Event};
use std::collections::HashMap;
use helper::*;
use wasm_bindgen::{JsCast, closure::Closure};
use serde::{Serialize, Deserialize};
use std::rc::Rc;
use std::cell::RefCell;



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
    
    // Context menu state
    let context_menu_state = use_state(|| ContextMenuState::new());
    
    // NodeRef for scrolling container
    let container_ref = use_node_ref();
    
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

    // Public function to scroll to a specific cell
    // We'll use this via a callback to scroll to a cell when it's entered in the "Enter cell" input
    let scroll_to_cell = {
        let container_ref = container_ref.clone();
        // Clone selected_cell before moving it into the closure
        let selected_cell = selected_cell.clone();
        let on_cell_select = props.on_cell_select.clone();
        
        Callback::from(move |cell_id: String| {
            // First update the selected cell state and emit cell selection
            selected_cell.set(Some(cell_id.clone()));
            on_cell_select.emit(cell_id.clone());
            
            // Extract column label and row number from cell id (e.g. "A1" -> col="A", row=1)
            let mut digit_start_index = 0;
            for (i, c) in cell_id.chars().enumerate() {
                if c.is_digit(10) {
                    digit_start_index = i;
                    break;
                }
            }
            
            if digit_start_index > 0 {
                if let Some(container) = container_ref.cast::<HtmlElement>() {
                    // Find the cell element we want to scroll to
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            // Try to find the cell by its data-id attribute
                            let selector = format!("td[data-id=\"{}\"]", cell_id);
                            if let Ok(Some(element)) = document.query_selector(&selector) {
                                let cell_element = element.dyn_into::<HtmlElement>().unwrap();
                                
                                // Ensure the spreadsheet container has focus to receive keyboard events
                                container.focus().ok();
                                
                                // Smooth scroll to the element
                                cell_element.scroll_into_view_with_bool(true); // true for smooth scrolling
                                
                                // Debug output to console
                                web_sys::console::log_1(&format!("Scrolling to cell: {}", cell_id).into());
                            } else {
                                // Debug output if cell not found
                                web_sys::console::log_1(&format!("Cell not found: {}", cell_id).into());
                            }
                        }
                    }
                }
            }
        })
    };
    
    // We no longer need to try to share the scroll_to_cell function with the parent component
    // The App component has its own direct implementation for scrolling

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
                        let cell_height = 35;      // Exact height from CSS
                        let row_header_width = 40; // Width of the row headers (1, 2, 3...)
                        let col_header_height = 35; // Height of the column headers (A, B, C...)
                        
                        // Calculate the exact pixel offset
                        // Subtract header sizes to account for the fixed headers
                        // Start exact at top-left corner of the target cell
                        let scroll_top = ((row - 1) * cell_height) ;  // No extra offset needed vertically
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
        
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: HtmlInputElement = e.target_unchecked_into();
                let field_id = input.id();
                let value = input.value();
                
                web_sys::console::log_1(&format!("Enter pressed in field: {}, value: {}", field_id, value).into());
                
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
                                    updated_values.insert(cell_id.clone(), value);
                                    cell_values.set(updated_values);
                                    
                                    // Add success message
                                    updated_messages.insert(field_id.clone(), 
                                        format!("Formula applied: {}", formula));
                                    
                                    // Scroll to the cell with the formula result
                                    web_sys::console::log_1(&format!("Scrolling to cell after formula: {}", cell_id).into());
                                    scroll_to_cell.emit(cell_id);
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
                        },
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
                                                autofocus={field.id() == "cell"} // Auto-focus the cell input
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
                        let scroll_to_cell = scroll_to_cell.clone();
                        
                        Callback::from(move |formula: String| {
                            // Process formula and update state
                            if let Some((cell_id, value)) = process_formula(&formula) {
                                let mut updated = (*cell_values).clone();
                                updated.insert(cell_id.clone(), value);
                                cell_values.set(updated);
                                // Optionally select the cell after formula apply
                                on_cell_select.emit(cell_id.clone());
                                
                                // Scroll to the cell with the formula result
                                scroll_to_cell.emit(cell_id);
                            }
                        })
                    }
                    on_scroll_to_cell_ref={Callback::from(move |_| {
                        // We're not using this anymore, but keep it in props for now
                    })}
                />
                
                <div class="api-info">
                    <p class="note">{"Note: In the future, this spreadsheet will connect to a backend API to process formulas and update cell values."}</p>
                    <p class="instructions">{"To use the spreadsheet: Click on a cell to select it, then enter a formula like \"A1=10+B2\" and press Enter."}</p>
                    <p class="instructions">{"You can also type a cell reference (like A1) in the \"Enter cell\" field and press Enter to quickly navigate to that cell."}</p>
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
    
    // Rest of the code remains the same
    html! {
        <>
            <h1> {"Rust Spreadsheet"} </h1>
            
            <div class="navigation-bar">
                <div class="nav-section">
                    {
                        if let Some(cell_field) = (*fields).get("cell") {
                            html! {
                                <div class="input-field">
                                    <label for={cell_field.id().to_string()}> {format!("Enter {}: ", cell_field.label())} </label>
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
                                    <label for={formula_field.id().to_string()}> {format!("Enter {}: ", formula_field.label())} </label>
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
                    
                    <button class="nav-button graph-button">{"Analyze"}</button>
                </div>
            </div>
            
            {content}
        </>
    }
}

fn main(){
    yew::Renderer::<App>::new().render();   
}