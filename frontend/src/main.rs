use yew::prelude::*;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent};

#[function_component(App)]
fn app() -> Html {
    let input_value = use_state(|| "".to_string());
    let displayed_name = use_state(|| "".to_string());
    let oninput = {
        let input_value = input_value.clone();
        Callback::from(move |e: InputEvent|{
            let input: HtmlInputElement = e.target_unchecked_into();
            input_value.set(input.value());
        })
    };
    let onkeydown = {
        let input_value = input_value.clone();
        let displayed_name = displayed_name.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                if parse_input_name(&input_value) {
                    displayed_name.set((*input_value).clone());
                } else {
                    displayed_name.set(String::from("Enter a valid name"));
                }
            }
        })
    };
    let input_cell = use_state(|| "".to_string());
    let displayed_cell = use_state(|| "".to_string());
    let oninput_cell = {
        let input_cell = input_cell.clone();
        Callback::from(move |e: InputEvent|{
            let input: HtmlInputElement = e.target_unchecked_into();
            input_cell.set(input.value());
        })
    };
    let onkeydown_cell = {
        let input_cell = input_cell.clone();
        let displayed_cell = displayed_cell.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                if parse_input_cell(&input_cell) {
                    displayed_cell.set((*input_cell).clone());
                } else {
                    displayed_cell.set(String::from("Enter a valid cell"));
                }
            }
        })
    };
    html!{
        <>
            <h1> {"RustConf Explorer"} </h1>
            <div style="margin-top:2rem;">
                <label for="username"> {"Enter your name: "}</label>
                <input
                    id="username"
                    type="text"
                    value={ (*input_value).clone() }
                    oninput={ oninput }
                    onkeydown={onkeydown}
                />
                <p>{ format!("Hello, {}", *displayed_name) }</p>
            </div>
            <div style="margin-top:2rem;">
                <label for="cell"> {"Enter you cell: "}</label>
                <input
                    id="cell"
                    type="text"
                    value={ (*input_cell).clone() }
                    oninput={ oninput_cell }
                    onkeydown={onkeydown_cell}
                />
                <p>{ format!("You entered cell: {}", *displayed_cell) }</p>
            </div>
        </>
    }
}

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