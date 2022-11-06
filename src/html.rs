use crate::canvas::Canvas;
use crate::canvas::Color;
use crate::canvas::BLACK;
use crate::suffix_array::draw_sa;
use crate::suffix_array::states;
use crate::suffix_array::State;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlInputElement;

struct HtmlCanvas {
    context: CanvasRenderingContext2d,
    element: HtmlCanvasElement,
}

fn jscol((r, g, b): Color) -> JsValue {
    JsValue::from_str(&format!("rgb({r},{g},{b})"))
}
fn jsstr(s: &str) -> JsValue {
    JsValue::from_str(s)
}
fn log(s: &str) {
    web_sys::console::log_1(&jsstr(s));
}

fn document() -> web_sys::Document {
    let window = web_sys::window().expect("no global `window` exists");
    window.document().expect("should have a document on window")
}

fn get<T: wasm_bindgen::JsCast>(id: &str) -> T {
    document()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<T>()
        .unwrap()
}

impl Canvas for HtmlCanvas {
    fn fill_background(&mut self, color: crate::canvas::Color) {
        //self.context.set_fill_style(&jscol(color));
        self.context.clear_rect(
            0.,
            0.,
            self.context.canvas().unwrap().width() as f64,
            self.context.canvas().unwrap().height() as f64,
        );
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: crate::canvas::Color) {
        self.context.set_fill_style(&jscol(color));
        self.context
            .fill_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: crate::canvas::Color) {
        self.context.begin_path();
        self.context.set_stroke_style(&jscol(color));
        self.context
            .stroke_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn write_text(
        &mut self,
        x: i32,
        y: i32,
        ha: crate::canvas::HAlign,
        va: crate::canvas::VAlign,
        text: &str,
    ) {
        self.context.set_fill_style(&jscol(BLACK));
        self.context.set_font("20px Arial");
        self.context.set_text_baseline("middle");
        self.context.set_text_align(match ha {
            crate::canvas::HAlign::Left => "left",
            crate::canvas::HAlign::Center => "center",
            crate::canvas::HAlign::Right => "right",
        });
        self.context.fill_text(text, x as f64, y as f64).unwrap();
    }

    // no-op
    fn present(&mut self) {}

    // no-op
    fn save(&mut self) {}
}

static mut STRING: String = String::new();
static mut STATE: usize = 0;
static mut FORWARD: bool = true;
static mut STATES: Vec<State> = vec![];

#[wasm_bindgen]
pub fn update_string() {
    unsafe {
        STRING = get::<HtmlInputElement>("string").value();
        STATE = 0;
        FORWARD = true;
    };
}

#[wasm_bindgen]
pub fn prev() {
    unsafe {
        if STATE > 0 {
            STATE -= 1;
        }
        FORWARD = false;
    };
}

#[wasm_bindgen]
pub fn next() {
    unsafe {
        if STATE + 1 < STATES.len() {
            STATE += 1;
        }
        FORWARD = true;
    }
}

#[wasm_bindgen]
pub fn draw() {
    let element = get::<HtmlCanvasElement>("drawing");
    let mut s = unsafe { STRING.as_bytes() };
    if s.is_empty() {
        s = "GTCCCGATGTCATGTCAGGA$".as_bytes();
    }
    log(&format!("STRING: {s:?}"));
    let n = s.len();

    unsafe {
        STATES = states(n);
    }

    let context = element
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    let mut canvas = HtmlCanvas { element, context };
    //let (w, h) = canvas_size(n + 4, n + 4);
    loop {
        let state_id = unsafe { STATE };
        let state = unsafe { STATES.get(state_id) };
        let Some(state) = state else {
            break;
        };
        log(&format!("state {state_id}: {state:?}"));
        if draw_sa(s, *state, &mut canvas) {
            break;
        }
        unsafe {
            if FORWARD {
                next();
            } else {
                prev();
            }
        };
    }
    canvas.context.fill();
}
