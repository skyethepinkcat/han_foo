use std::{cell::RefCell, rc::Rc};

use crate::han_foo as hf;
use rand::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlDocument, HtmlElement, HtmlInputElement, NodeList, console};

use crate::han_foo::Agari;

pub static DEFAULT_PARAM: f32 = 0.5;

fn doc() -> HtmlDocument {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .dyn_into()
        .unwrap()
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

fn nodes_to_radios(nodes: NodeList) -> Result<Vec<HtmlInputElement>, JsValue> {
    let mut out = vec![];
    for i in 0..nodes.length() {
        let node = nodes.get(i).unwrap();
        let input: HtmlInputElement = node.dyn_into()?;
        out.push(input);
    }
    Ok(out)
}

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
        let root: HtmlElement = document.get_html_by_id("options_menu").unwrap();
        Self {
            open: false,
            button: document.get_html_by_id("options_button").unwrap(),
            kiriage: document.get_html_by_id("kiriage").unwrap(),
            modes: nodes_to_radios(root.query_selector_all("input[name=\"mode\"]").unwrap())
                .unwrap(),
            root: root,
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
            if (i.value_as_number() as f32) == options.random_param {
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
            .query_html_selector_all::<HtmlInputElement>("input[name\"mode\"]:checked")
            .unwrap();

        options.random_param = radios.first().unwrap().value_as_number() as f32;
        #[cfg(debug_assertions)]
        {
            console::log_2(
                &format!("Kiriage: {}", options.kiriage).into(),
                &format!("Parameter: {}", options.random_param).into(),
            );
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
                root: document.get_html_by_id("back").unwrap(),
                points: document.get_html_by_id("points").unwrap(),
            },
            flipped: false,
            front: Front {
                dealer: document.get_html_by_id("dealer").unwrap(),
                win_type: document.get_html_by_id("win_type").unwrap(),
                root: document.get_html_by_id("front").unwrap(),
                _han_section: document.get_html_by_id("han_section").unwrap(),
                han_num: document.get_html_by_id("han_count").unwrap(),
                fu_section: document.get_html_by_id("fu_section").unwrap(),
                fu_num: document.get_html_by_id("fu_count").unwrap(),
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
            card: Card::new(&document.get_html_by_id("card").unwrap()),
            menu: Menu::new(&document.get_html_by_id("menu").unwrap()),
            options: Options {
                kiriage: true,
                random_param: DEFAULT_PARAM,
            },
            agari: Agari::new(
                hf::random_score(&mut rng, DEFAULT_PARAM),
                rng.random_bool(0.5),
                rng.random_bool(0.5),
            ),
            rng: rng,
        };

        s.card.update(s.agari, s.options.kiriage);
        s.menu.load(&s.options);

        s
    }

    pub fn generate(&mut self) {
        let last = self.agari;
        self.agari = Agari::new(
            hf::random_score(&mut self.rng, self.options.random_param),
            self.rng.random_bool(0.5),
            self.rng.random_bool(0.5),
        );
        // Re-roll score if we got it last... Its no fun getting duplicates!
        if self.agari == last {
            self.generate();
        }
        // Re-roll scores that aren't possible.
        if self.agari.score.fu == 20 && !self.agari.tsumo {
            self.generate();
        } else if self.agari.score.fu == 25 && self.agari.score.han == 2 && self.agari.tsumo {
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

    pub fn save_options(&mut self) {
        self.menu.save(&mut self.options);
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

#[cfg(test)]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn default_param_is_half() {
        assert_eq!(DEFAULT_PARAM, 0.5);
    }

    // Verify the impossible-hand conditions that State::generate() filters.
    // 20 fu only valid for tsumo (pinfu tsumo); ron with 20 fu can't occur.
    #[wasm_bindgen_test]
    fn fu20_ron_is_filtered_condition() {
        let agari = hf::Agari::new(hf::Score { han: 1, fu: 20 }, false, false);
        assert!(agari.score.fu == 20 && !agari.tsumo, "generate() must re-roll this");
    }

    #[wasm_bindgen_test]
    fn fu20_tsumo_is_not_filtered() {
        let agari = hf::Agari::new(hf::Score { han: 1, fu: 20 }, false, true);
        assert!(!(agari.score.fu == 20 && !agari.tsumo), "generate() must keep this");
    }

    // 25 fu = chiitoitsu; tsumo with exactly 2 han is impossible in standard rules.
    #[wasm_bindgen_test]
    fn chiitoitsu_tsumo_2han_is_filtered_condition() {
        let agari = hf::Agari::new(hf::Score { han: 2, fu: 25 }, false, true);
        assert!(
            agari.score.fu == 25 && agari.score.han == 2 && agari.tsumo,
            "generate() must re-roll this"
        );
    }

    #[wasm_bindgen_test]
    fn chiitoitsu_ron_2han_is_not_filtered() {
        let agari = hf::Agari::new(hf::Score { han: 2, fu: 25 }, false, false);
        assert!(
            !(agari.score.fu == 25 && agari.score.han == 2 && agari.tsumo),
            "generate() must keep this"
        );
    }

    // Agari::points delegates correctly — spot check dealer tsumo through gui's Agari wrapper.
    #[wasm_bindgen_test]
    fn agari_points_dealer_tsumo_mangan() {
        let agari = hf::Agari::new(hf::Score { han: 5, fu: 30 }, true, true);
        match agari.points(false).unwrap() {
            hf::RonOrTsumo::Tsumo(v) => assert_eq!(v, [4000, 4000]),
            _ => panic!("Expected Tsumo"),
        }
    }

    #[wasm_bindgen_test]
    fn agari_points_kiriage_affects_result() {
        let agari = hf::Agari::new(hf::Score { han: 4, fu: 30 }, false, false);
        let no_kiriage = match agari.points(false).unwrap() {
            hf::RonOrTsumo::Ron(v) => v,
            _ => panic!(),
        };
        let kiriage = match agari.points(true).unwrap() {
            hf::RonOrTsumo::Ron(v) => v,
            _ => panic!(),
        };
        assert_ne!(no_kiriage, kiriage);
        assert_eq!(kiriage, 8000);
    }
}

#[cfg(test)]
mod browser_tests {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Injects the same DOM structure as index.html into document.body.
    fn setup_dom() {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .set_inner_html(
                r#"
                <span id="options_button"></span>
                <div id="options_menu">
                  <input id="kiriage" type="checkbox">
                  <input id="chaos"   type="radio" name="mode" value="0.0">
                  <input id="normal"  type="radio" name="mode" value="0.5" checked>
                  <input id="unlucky" type="radio" name="mode" value="1.0">
                </div>
                <div id="card">
                  <div id="front">
                    <span id="dealer"></span>
                    <span id="win_type"></span>
                    <span id="han_section"><span id="han_count"></span></span>
                    <span id="fu_section"><span id="fu_count"></span></span>
                  </div>
                  <div id="back"><span id="points"></span></div>
                </div>
                "#,
            );
    }

    fn get_doc() -> HtmlDocument {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .dyn_into()
            .unwrap()
    }

    // Fails: State::new passes document.get_html_by_id("card") to Card::new,
    // which dyn_casts a div to HtmlDocument -> panic.
    #[wasm_bindgen_test]
    fn state_new_does_not_panic() {
        setup_dom();
        let _ = State::new(&get_doc());
    }

    // Fails: same panic in State::new before this assertion is reached.
    #[wasm_bindgen_test]
    fn state_menu_initially_closed() {
        setup_dom();
        let state = State::new(&get_doc());
        assert!(!state.menu.open, "menu starts closed");
    }

    // Fails: same panic in State::new before DOM is updated.
    // Once fixed, verifies menu.load() set kiriage checkbox to match Options default (true).
    #[wasm_bindgen_test]
    fn state_kiriage_checkbox_checked_by_default() {
        setup_dom();
        let doc = get_doc();
        let _ = State::new(&doc);
        let checkbox: HtmlInputElement = doc
            .get_element_by_id("kiriage")
            .unwrap()
            .dyn_into()
            .unwrap();
        assert!(checkbox.checked(), "kiriage defaults true → checkbox checked");
    }

    // Fails: same panic in State::new before DOM is updated.
    // Once fixed, verifies menu.load() selected the 0.5-param radio (DEFAULT_PARAM).
    #[wasm_bindgen_test]
    fn state_normal_mode_radio_selected_by_default() {
        setup_dom();
        let doc = get_doc();
        let _ = State::new(&doc);
        let normal: HtmlInputElement = doc
            .get_element_by_id("normal")
            .unwrap()
            .dyn_into()
            .unwrap();
        assert!(normal.checked(), "DEFAULT_PARAM=0.5 → normal radio checked");
    }

    // Fails: same panic in State::new.
    // Once fixed, verifies card starts unflipped.
    #[wasm_bindgen_test]
    fn state_card_not_flipped_initially() {
        setup_dom();
        let state = State::new(&get_doc());
        assert!(!state.card.flipped, "card starts face-down");
    }

    // Fails: same panic in State::new.
    // Once fixed, verifies points text was written to #points on init.
    #[wasm_bindgen_test]
    fn state_back_points_populated_on_init() {
        setup_dom();
        let doc = get_doc();
        let _ = State::new(&doc);
        let points_text = doc
            .get_element_by_id("points")
            .unwrap()
            .text_content()
            .unwrap_or_default();
        assert!(!points_text.is_empty(), "#points populated on init");
    }
}
