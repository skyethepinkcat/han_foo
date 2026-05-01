use rand::prelude::*;
use wasm_bindgen::prelude::*;

mod han_foo;
mod web;

use han_foo as hf;
use web_sys::{HtmlDocument, HtmlElement};

static DEFAULT_PARAM: f32 = 0.5;

const FU_VALS: [u32; 12] = [20, 25, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120];

const HAN_VALS: [u32; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

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
    flipped: bool,
    back: Back,
    front: Front,
}

impl Card {
    pub fn new(document: web_sys::HtmlDocument) -> Self {
        Card {
            back: Back {
                root: document
                    .get_element_by_id("back")
                    .unwrap()
                    .dyn_into()
                    .unwrap(),
            },
            flipped: false,
            front: Front {
                dealer: get_html_element(&document, "dealer").unwrap(),
                win_type: get_html_element(&document, "win_type").unwrap(),
                root: get_html_element(&document, "front").unwrap(),
                han_section: get_html_element(&document, "han_section").unwrap(),
                han_num: get_html_element(&document, "han_count").unwrap(),
                fu_section: get_html_element(&document, "fu_section").unwrap(),
                fu_num: get_html_element(&document, "fu_count").unwrap(),
            },
        }
    }
    pub fn flip(&mut self) {
        if self.flipped {
            self.back.hide();
            self.front.show();
        } else {
            self.front.hide();
            self.back.show();
        }
    }
    fn update(&mut self, han_num: u32, fu_num: u32, dealer: bool, tsumo: bool) {
        self.front.update(han_num,fu_num,dealer,tsumo);
        if self.flipped {
            self.flip();
        }
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
    const NON_DEALER_TEXT: &str = "NON-DEALER";
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
}
impl Back {
    fn update(&self, string: &str) {
        self.root.set_text_content(Some(string));
    }
}
pub trait CardSide {
    fn root(&mut self) -> &mut HtmlElement;
    fn hide(&mut self) {
        self.root().set_hidden(true);
    }
    fn show(&mut self) {
        self.root().set_hidden(false);
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
    flipped: bool,
    score: hf::Score,
    tsumo: bool,
    dealer: bool,
    random_param: f32,
    rng: ThreadRng,
}

impl State {
    pub fn new(document: web_sys::HtmlDocument, random_param: f32) -> Self {
        let mut rng = rand::rng();
        Self {
            card: Card::new(document),
            dealer: rng.random_bool(0.5),
            tsumo: rng.random_bool(0.5),
            flipped: false,
            score: hf::random_score(&mut rng, random_param),
            random_param: random_param,
            rng: rng,
        }
    }
    pub fn generate(&mut self) {
        self.dealer = self.rng.random_bool(0.5);
        self.tsumo = self.rng.random_bool(0.5);
        self.flipped = false;
        self.score = hf::random_score(&mut self.rng, self.random_param);
        self.card.front.update(self.score.han, self.score.fu, self.dealer, self.tsumo);
    }
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

    let mut state = State::new(document, DEFAULT_PARAM);

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.

    let card = document.get_element_by_id("card").unwrap();
    let result = document.get_element_by_id("result").unwrap();
    let question = document.get_element_by_id("question").unwrap();
    let han_elem = document.get_element_by_id("han_count").unwrap();
    let fu_elem = document.get_element_by_id("fu_count").unwrap();
    let tsumo_elem = document.get_element_by_id("win_type").unwrap();
    let dealer_elem = document.get_element_by_id("dealer").unwrap();
    let fu_section = document.get_element_by_id("fu_section").unwrap();

    let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
        if state.flipped {
            let rand_tsumo = rng.random_bool(0.5);
            let rand_dealer = rng.random_bool(0.5);
            state.score = hf::random_score(&mut state.rng, 0.5);
            state.flipped = false;
            han_elem.set_text_content(Some(&state.score.han.to_string()));
            fu_elem.set_text_content(Some(&state.score.fu.to_string()));
            dealer_elem.set_text_content(match rand_dealer {
                true => Some("DEALER"),
                false => Some("NON-DEALER"),
            });
            tsumo_elem.set_text_content(match rand_tsumo {
                true => Some("TSUMO"),
                false => Some("RON"),
            });
            fu_section.class_list().remove_1("hidden").unwrap();
            if state.score.han >= 5 {
                fu_section.class_list().add_1("hidden").unwrap();
            }
            result.class_list().add_1("hidden").unwrap();
            question.class_list().remove_1("hidden").unwrap();
        } else {
            question.class_list().add_1("hidden").unwrap();
            result.class_list().remove_1("hidden").unwrap();
            result.set_text_content(Some(
                &state.score.points(false, false, true).unwrap().to_string(),
            ));
            state.flipped = true;
        }
    });

    card.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;

    closure.forget();

    Ok(())
}
