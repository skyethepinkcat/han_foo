use std::{cell::RefCell, rc::Rc};

mod han_foo;
mod web;
use wasm_bindgen::prelude::*;
use web::*;

use web_sys::{HtmlDocument, InputEvent, KeyboardEvent, MouseEvent, Node, console};

use wasm_cookies;

pub fn make_card_click_handler(state: Rc<RefCell<State>>) -> Closure<dyn FnMut(MouseEvent)> {
    Closure::new(move |_event| {
        #[cfg(debug_assertions)]
        {
            console::log_1(&"Card Clicked!".to_string().into());
        }
        flip_card(&state);
    })
}

pub fn make_space_handler(state: Rc<RefCell<State>>) -> Closure<dyn FnMut(KeyboardEvent)> {
    Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
        #[cfg(debug_assertions)]
        {
            console::log_1(&"Spacebar Hit!".to_string().into());
        }
        if event.key() == " " {
            flip_card(&state);
        }
    })
}

pub fn make_options_handler(state: Rc<RefCell<State>>) -> Closure<dyn FnMut(InputEvent)> {
    Closure::<dyn FnMut(_)>::new(move |_event: InputEvent| {
        #[cfg(debug_assertions)]
        {
            console::log_1(&"Input Event!".to_string().into());
        }
        let mut state = state.borrow_mut();
        state.save_options();
    })
}

pub fn make_options_click_handler(state: Rc<RefCell<State>>) -> Closure<dyn FnMut(MouseEvent)> {
    Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
        #[cfg(debug_assertions)]
        {
            console::log_1(&"Click away logic!".to_string().into());
        }
        let mut state = state.borrow_mut();
        let target_node: Option<Node> = event.target().and_then(|t| t.dyn_into::<Node>().ok());
        if state.menu().button().contains(target_node.as_ref()) {
            state.menu_mut().toggle();
        } else if !state.menu().root().contains(target_node.as_ref()) && state.menu().open() {
            state.menu_mut().toggle();
        }
    })
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window = web_sys::window().expect("no global `window` exists");
    let document = window
        .document()
        .expect("should have a document on window")
        .dyn_into::<HtmlDocument>()
        .unwrap();

    let state = Rc::new(RefCell::new(State::new(&document)));

    #[cfg(target_arch = "wasm32")]
    {
        let mut state = state.borrow_mut();
        match wasm_cookies::get("options") {
            Some(t) => {
                match serde_json::from_str::<Options>(&t.unwrap()) {
                    Ok(o) => {
                        state.set_options(o);
                    }
                    Err(_) => {}
                };
            }
            None => {}
        };
    }

    // state.borrow_mut().generate();

    let card_click_closure = make_card_click_handler(Rc::clone(&state));
    state
        .borrow_mut()
        .card()
        .root()
        .add_event_listener_with_callback("click", card_click_closure.as_ref().unchecked_ref())?;

    let key_closure = make_space_handler(Rc::clone(&state));
    window.add_event_listener_with_callback("keydown", key_closure.as_ref().unchecked_ref())?;

    let options_clickaway_closure = make_options_click_handler(Rc::clone(&state));
    document.add_event_listener_with_callback(
        "click",
        options_clickaway_closure.as_ref().unchecked_ref(),
    )?;

    let options_closure = make_options_handler(Rc::clone(&state));
    document.add_event_listener_with_callback("input", options_closure.as_ref().unchecked_ref())?;

    options_closure.forget();
    card_click_closure.forget();
    key_closure.forget();
    options_clickaway_closure.forget();

    #[cfg(debug_assertions)]
    {
        console::log_1(&"Starting with Debug Mode".to_string().into());
    }
    Ok(())
}
