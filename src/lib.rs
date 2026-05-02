use std::{cell::RefCell, rc::Rc};

use rand::prelude::*;
use wasm_bindgen::prelude::*;

mod han_foo;
mod web;

use han_foo as hf;
use web_sys::{HtmlDocument, HtmlElement, KeyboardEvent, MouseEvent};

static DEFAULT_PARAM: f32 = 0.5;

/// Gets an HTML element by its ID from an HTML document.
///
/// # Errors
///
/// This function will return an error if it can't find the element.
fn get_html_element(document: &HtmlDocument, id: &str) -> Result<HtmlElement, JsValue> {
    Ok(document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("No element with id {}", id)))?
        .dyn_into::<HtmlElement>()
        .unwrap())
}

struct Card {
    root: HtmlElement,
    flipped: bool,
    back: Back,
    front: Front,
}

impl Card {
    pub fn new(document: &web_sys::HtmlDocument) -> Self {
        Card {
            root: get_html_element(document, "card").unwrap(),
            back: Back {
                root: get_html_element(document, "back").unwrap(),
                points: get_html_element(document, "points").unwrap(),
            },
            flipped: false,
            front: Front {
                dealer: get_html_element(document, "dealer").unwrap(),
                win_type: get_html_element(document, "win_type").unwrap(),
                root: get_html_element(document, "front").unwrap(),
                han_section: get_html_element(document, "han_section").unwrap(),
                han_num: get_html_element(document, "han_count").unwrap(),
                fu_section: get_html_element(document, "fu_section").unwrap(),
                fu_num: get_html_element(document, "fu_count").unwrap(),
            },
        }
    }
    pub fn flip(&mut self) {
        if self.flipped {
            self.root.class_list().remove_1("flip").unwrap();
            self.flipped = false;
        } else {
            self.root.class_list().add_1("flip").unwrap();
            self.flipped = true;
        }
    }

    fn update(&mut self, score: hf::Score, dealer: bool, tsumo: bool) {
        self.front.update(score.han, score.fu, dealer, tsumo);
        self.back
            .update(&score.points(tsumo, dealer, true).unwrap().to_string())
    }
}

struct Front {
    root: HtmlElement,
    dealer: HtmlElement,
    win_type: HtmlElement,
    han_section: HtmlElement,
    han_num: HtmlElement,
    fu_section: HtmlElement,
    fu_num: HtmlElement,
}

impl Front {
    const DEALER_TEXT: &str = "DEALER";
    const NON_DEALER_TEXT: &str = "NON‑DEALER";
    const RON_TEXT: &str = "RON";
    const TSUMO_TEXT: &str = "TSUMO";

    fn update(&self, han_num: u32, fu_num: u32, dealer: bool, tsumo: bool) {
        if han_num >= 5 {
            self.fu_section.set_hidden(true);
        } else {
            self.fu_section.set_hidden(false);
        }

        self.fu_num.set_text_content(Some(&fu_num.to_string()));
        self.han_num.set_text_content(Some(&han_num.to_string()));
        self.dealer.set_text_content(Some(match dealer {
            true => Self::DEALER_TEXT,
            false => Self::NON_DEALER_TEXT,
        }));

        self.win_type.set_text_content(Some(match tsumo {
            true => Self::TSUMO_TEXT,
            false => Self::RON_TEXT,
        }));
    }
}

struct Back {
    root: HtmlElement,
    points: HtmlElement,
}
impl Back {
    fn update(&self, string: &str) {
        self.points.set_text_content(Some(string));
    }
}
pub trait CardSide {
    fn root(&mut self) -> &mut HtmlElement;
    fn hide(&mut self) {
        self.root().class_list().add_1("hidden").unwrap();
    }
    fn show(&mut self) {
        self.root().class_list().remove_1("hidden").unwrap();
    }
}

impl CardSide for Front {
    fn root(&mut self) -> &mut HtmlElement {
        &mut self.root
    }
}
impl CardSide for Back {
    fn root(&mut self) -> &mut HtmlElement {
        &mut self.root
    }
}

struct State {
    card: Card,
    score: hf::Score,
    tsumo: bool,
    dealer: bool,
    random_param: f32,
    rng: ThreadRng,
}

impl State {
    pub fn new(document: &web_sys::HtmlDocument, random_param: f32) -> Self {
        let mut rng = rand::rng();
        let mut s = Self {
            card: Card::new(document),
            dealer: rng.random_bool(0.5),
            tsumo: rng.random_bool(0.5),
            score: hf::random_score(&mut rng, random_param),
            random_param: random_param,
            rng: rng,
        };

        s.card.update(s.score, s.dealer, s.tsumo);

        s
    }
    pub fn generate(&mut self) {
        let last = self.score;
        self.dealer = self.rng.random_bool(0.5);
        self.tsumo = self.rng.random_bool(0.5);
        self.score = hf::random_score(&mut self.rng, self.random_param);
        // Re-roll score if we got it last... Its no fun getting duplicates!
        if self.score == last {
            self.generate();
        }
        // Re-roll scores that aren't possible.
        if self.score.fu == 20 && !self.tsumo {
            self.generate();
        } else if self.score.fu == 25 && self.score.han == 2 && self.tsumo {
            self.generate();
        }
    }
}

fn flip_card(state: &Rc<RefCell<State>>) {
    let mut state = state.borrow_mut();
    if state.card.flipped {
        state.generate();
        state
            .card
            .front
            .update(state.score.han, state.score.fu, state.dealer, state.tsumo);
        state.card.flip();
    } else {
        state.card.back.update(
            &state
                .score
                .points(state.tsumo, state.dealer, true)
                .unwrap()
                .to_string(),
        );
        state.card.flip();
    }
}
fn make_click_handler(state: Rc<RefCell<State>>) -> Closure<dyn FnMut(MouseEvent)> {
    Closure::new(move |_event| {
        flip_card(&state);
    })
}
fn make_keydown_handler(state: Rc<RefCell<State>>) -> Closure<dyn FnMut(KeyboardEvent)> {
    Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
        if event.key() == " " {
            flip_card(&state);
        }
    })
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window
        .document()
        .expect("should have a document on window")
        .dyn_into::<HtmlDocument>()
        .unwrap();

    let state = Rc::new(RefCell::new(State::new(&document, DEFAULT_PARAM)));

    // state.borrow_mut().generate();

    let click_closure = make_click_handler(Rc::clone(&state));
    state
        .borrow_mut()
        .card
        .root
        .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;

    let key_closure = make_keydown_handler(Rc::clone(&state));
    document.add_event_listener_with_callback("keydown", key_closure.as_ref().unchecked_ref())?;

    click_closure.forget();
    key_closure.forget();

    Ok(())
}
