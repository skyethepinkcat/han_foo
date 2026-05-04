use std::{cell::RefCell, rc::Rc};

use crate::han_foo as hf;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use serde_json::json;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlDocument, HtmlElement, HtmlInputElement};

use crate::han_foo::Agari;

pub static DEFAULT_PARAM: f32 = 0.5;

#[cfg(target_arch = "wasm32")]
#[cfg(debug_assertions)]
use web_sys::console;

#[macro_export]
macro_rules! debug_log {
    ($x:expr) => {
        #[cfg(debug_assertions)]
        {
            #[cfg(target_arch = "wasm32")]
            console::log_1(&($x).to_string().into());
            #[cfg(not(target_arch = "wasm32"))]
            eprintln!("{}", $x)
        }
    };
}

trait HtmlSelector {
    fn query_html_selector<T: JsCast>(&self, selector: &str) -> Result<T, JsValue>;
    fn query_html_selector_all<T: JsCast>(&self, selector: &str) -> Result<Vec<T>, JsValue>;
    fn get_html_by_id<T: JsCast>(&self, id: &str) -> Result<T, JsValue>;
}

impl HtmlSelector for HtmlDocument {
    fn query_html_selector<T: JsCast>(&self, selector: &str) -> Result<T, JsValue> {
        Ok(self
            .query_selector(selector)?
            .unwrap()
            .dyn_into::<T>()
            .unwrap())
    }

    fn query_html_selector_all<T: JsCast>(&self, selector: &str) -> Result<Vec<T>, JsValue> {
        let nodes = self.query_selector_all(selector)?;
        let mut out = vec![];
        for i in 0..nodes.length() {
            let node = nodes.get(i).unwrap();
            let input: T = node.dyn_into()?;
            out.push(input);
        }
        Ok(out)
    }

    fn get_html_by_id<T: JsCast>(&self, id: &str) -> Result<T, JsValue> {
        match self.get_element_by_id(id).unwrap().dyn_into::<T>() {
            Ok(t) => Ok(t),
            Err(_) => Err(JsValue::from_str("Wrong Element Type!")),
        }
    }
}

impl HtmlSelector for HtmlElement {
    fn query_html_selector<T: JsCast>(&self, selector: &str) -> Result<T, JsValue> {
        Ok(self
            .query_selector(selector)?
            .unwrap()
            .dyn_into::<T>()
            .unwrap())
    }

    fn query_html_selector_all<T: JsCast>(&self, selector: &str) -> Result<Vec<T>, JsValue> {
        let nodes = self.query_selector_all(selector)?;
        let mut out = vec![];
        for i in 0..nodes.length() {
            let node = nodes.get(i).unwrap();
            let input: T = node.dyn_into()?;
            out.push(input);
        }
        Ok(out)
    }

    fn get_html_by_id<T: JsCast>(&self, id: &str) -> Result<T, JsValue> {
        self.query_html_selector::<T>(&format!("#{}", id))
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Options {
    kiriage: bool,
    random_param: f32,
}

pub struct Menu {
    root: HtmlElement,
    button: HtmlElement,
    open: bool,
    kiriage: HtmlInputElement,
    modes: Vec<HtmlInputElement>,
}

impl Menu {
    pub fn new(document: &HtmlDocument) -> Self {
        let root: HtmlElement = document
            .get_html_by_id::<HtmlElement>("options_menu")
            .unwrap();
        Self {
            open: false,
            button: document
                .get_html_by_id::<HtmlElement>("options_button")
                .unwrap(),
            kiriage: root
                .get_html_by_id::<HtmlInputElement>("kiriage_input")
                .unwrap(),
            modes: root
                .query_html_selector_all::<HtmlInputElement>("input[name=\"mode\"]")
                .unwrap(),
            root,
        }
    }

    pub fn toggle(&mut self) {
        if self.open {
            self.root.set_hidden(true);
            self.open = false;
        } else {
            self.root.set_hidden(false);
            self.open = true;
        }
    }

    pub fn button(&self) -> &HtmlElement {
        &self.button
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn root(&self) -> &HtmlElement {
        &self.root
    }

    fn load(&self, options: &Options) {
        self.kiriage.set_checked(options.kiriage);

        for i in &self.modes {
            if (i.value().parse::<f32>().unwrap()) == options.random_param {
                i.set_checked(true);
            } else {
                i.set_checked(false);
            }
        }
    }
    pub fn save(&self, options: &mut Options) {
        options.kiriage = self.kiriage.checked();

        let radios = self
            .root
            .query_html_selector_all::<HtmlInputElement>("input[name=\"mode\"]:checked")
            .unwrap();

        options.random_param = radios.first().unwrap().value().parse().unwrap();
        #[cfg(debug_assertions)]
        {
            debug_log!(format!("Kiriage: {}", options.kiriage));
            debug_log!(format!("Parameter: {}", options.random_param));
            debug_log!(format!("value: {}", radios.first().unwrap().value()));
        }
    }
}

pub struct Card {
    root: HtmlElement,
    flipped: bool,
    back: Back,
    front: Front,
}

impl Card {
    pub fn new(document: &web_sys::HtmlDocument) -> Self {
        Card {
            root: document.get_html_by_id::<HtmlElement>("card").unwrap(),
            back: Back {
                root: document.get_html_by_id::<HtmlElement>("back").unwrap(),
                points: document.get_html_by_id::<HtmlElement>("points").unwrap(),
            },
            flipped: false,
            front: Front {
                dealer: document.get_html_by_id::<HtmlElement>("dealer").unwrap(),
                win_type: document.get_html_by_id::<HtmlElement>("win_type").unwrap(),
                root: document.get_html_by_id::<HtmlElement>("front").unwrap(),
                _han_section: document
                    .get_html_by_id::<HtmlElement>("han_section")
                    .unwrap(),
                han_num: document.get_html_by_id::<HtmlElement>("han_count").unwrap(),
                fu_section: document
                    .get_html_by_id::<HtmlElement>("fu_section")
                    .unwrap(),
                fu_num: document.get_html_by_id::<HtmlElement>("fu_count").unwrap(),
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

    fn update(&mut self, agari: Agari, kiriage: bool) {
        self.front
            .update(agari.score.han, agari.score.fu, agari.dealer, agari.tsumo);
        self.back
            .update(&agari.points(kiriage).unwrap().to_string())
    }

    pub fn root(&self) -> &HtmlElement {
        &self.root
    }
}

pub struct Front {
    #[allow(dead_code)]
    root: HtmlElement,
    dealer: HtmlElement,
    win_type: HtmlElement,
    _han_section: HtmlElement,
    han_num: HtmlElement,
    fu_section: HtmlElement,
    fu_num: HtmlElement,
}

impl Front {
    const DEALER_TEXT: &str = "DEALER";
    const NON_DEALER_TEXT: &str = "NON\u{2011}DEALER";
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

pub struct Back {
    #[allow(dead_code)]
    root: HtmlElement,
    points: HtmlElement,
}
impl Back {
    fn update(&self, string: &str) {
        self.points.set_text_content(Some(string));
    }
}

pub struct State {
    card: Card,
    menu: Menu,
    agari: hf::Agari,
    rng: ThreadRng,
    options: Options,
}

impl State {
    pub fn new(document: &web_sys::HtmlDocument) -> Self {
        let mut rng = rand::rng();
        let mut s = Self {
            card: Card::new(document),
            menu: Menu::new(document),
            options: Options {
                kiriage: true,
                random_param: DEFAULT_PARAM,
            },
            agari: hf::random_agari(&mut rng, DEFAULT_PARAM),

            rng,
        };

        s.card.update(s.agari, s.options.kiriage);
        s.menu.load(&s.options);

        s
    }

    pub fn generate(&mut self) {
        let last = self.agari;
        self.agari = hf::random_agari(&mut self.rng, self.options.random_param);


        debug_log!(format!(
            "Generated {:?} with {}",
            self.agari, self.options.random_param
        ));

        // Re-roll score if we got it last... Its no fun getting duplicates!
        if self.agari == last {
            self.generate();
        }
        // Re-roll scores that aren't possible.
        if (self.agari.score.fu == 20 && !self.agari.tsumo)
            || self.agari.score.fu == 25 && self.agari.score.han == 2 && self.agari.tsumo
        {
            self.generate();
        }
    }

    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn menu_mut(&mut self) -> &mut Menu {
        &mut self.menu
    }

    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    // Read the options from the UI
    pub fn save_options(&mut self) {
        self.menu.save(&mut self.options);

        #[cfg(target_arch = "wasm32")]
        {
            debug_log!(format!("Saved options {:?}", self.options));
            wasm_cookies::set(
                "options",
                &json!(self.options).to_string(),
                &wasm_cookies::CookieOptions::default(),
            );
        }
    }

    // Load state options into the UI.
    pub fn load_options(&mut self) {
        self.menu.load(&self.options);
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn set_options(&mut self, options: Options) {
        self.options = options;
        self.load_options();
    }
}

pub fn flip_card(state: &Rc<RefCell<State>>) {
    let mut state = state.borrow_mut();
    if state.card.flipped {
        state.generate();
        state.card.front.update(
            state.agari.score.han,
            state.agari.score.fu,
            state.agari.dealer,
            state.agari.tsumo,
        );
        state.card.flip();
    } else {
        state.card.back.update(
            &state
                .agari
                .points(state.options.kiriage)
                .unwrap()
                .to_string(),
        );
        state.card.flip();
    }
}
