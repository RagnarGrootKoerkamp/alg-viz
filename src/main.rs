#![feature(duration_constants)]
use std::time::Duration;

use suffix_array_construction::{
    bibwt, bwt,
    cli::{Algorithm, ARGS},
    grid::canvas_size,
    sdl::{new_canvas, wait_for_key, KeyboardAction},
    suffix_array as sa,
};

struct Interaction {
    len: usize,
    idx: usize,
    forward: bool,
    spf: Duration,
    playing: bool,
}

impl Interaction {
    fn new<State>(states: &Vec<State>) -> Self {
        Self {
            len: states.len(),
            idx: 0,
            forward: true,
            spf: Duration::SECOND,
            playing: true,
        }
    }
    fn prev(&mut self) -> bool {
        self.forward = false;
        let r = self.idx > 0;
        self.idx = self.idx.saturating_sub(1);
        r
    }
    fn next(&mut self) -> bool {
        self.forward = true;
        if self.idx + 1 < self.len {
            self.idx += 1;
            true
        } else {
            false
        }
    }
    fn step(&mut self) -> bool {
        if self.forward {
            self.next()
        } else {
            self.prev()
        }
    }
    fn toend(&mut self) {
        self.idx = self.len - 1;
    }
    fn get<State: Clone>(&self, states: &Vec<State>) -> State {
        states[self.idx].clone()
    }
    fn faster(&mut self) {
        self.spf = self.spf.div_f32(1.5);
    }
    fn slower(&mut self) {
        self.spf = self.spf.mul_f32(1.5);
    }
    fn pauseplay(&mut self) {
        self.playing = !self.playing;
    }

    fn wait(&mut self) {
        match wait_for_key(if self.playing {
            self.spf
        } else {
            Duration::MAX
        }) {
            KeyboardAction::Next => {
                self.next();
            }
            KeyboardAction::Prev => {
                self.prev();
            }
            KeyboardAction::PausePlay => self.pauseplay(),
            KeyboardAction::Faster => self.faster(),
            KeyboardAction::Slower => self.slower(),
            KeyboardAction::ToEnd => self.toend(),
            KeyboardAction::Exit => {
                eprintln!("Aborted by user!");
                std::process::exit(1);
            }
            KeyboardAction::None => {
                if self.playing {
                    self.step();
                }
            }
        }
    }
}

fn main() -> ! {
    let mut s = ARGS
        .input
        .clone()
        .unwrap_or("GTCCCGATGTCATGTCAGGA".to_owned());
    s.push('$');
    let s = s.as_bytes();
    let n = s.len();

    match ARGS.algorithm {
        Algorithm::SuffixArray => {
            let (w, h) = sa::canvas_size(s);
            let (w, h) = canvas_size(w, h);
            let ref mut canvas = new_canvas(w, h);
            let states = sa::states(n);
            let mut interaction = Interaction::new(&states);
            loop {
                if sa::draw(s, interaction.get(&states), canvas) {
                    canvas.present();
                    interaction.wait();
                } else {
                    interaction.step();
                }
            }
        }
        Algorithm::BWT => {
            let q = ARGS.query.clone().unwrap_or("GTCC".to_string());
            let q = q.as_bytes();

            let bwt = bwt::BWT::new(s, q);
            let (w, h) = bwt.canvas_size();
            let (w, h) = canvas_size(w, h);
            let ref mut canvas = new_canvas(w, h);
            let states = bwt.states();
            let mut interaction = Interaction::new(&states);
            loop {
                if bwt.draw(interaction.get(&states), canvas) {
                    canvas.present();
                    interaction.wait();
                } else {
                    interaction.step();
                }
            }
        }

        Algorithm::BiBWT => {
            let q = ARGS.query.clone().unwrap_or("GTCC".to_string());
            let q = q.as_bytes();

            let bibwt = bibwt::BiBWT::new(s, q);
            let (w, h) = bibwt.canvas_size();
            let (w, h) = canvas_size(w, h);
            let ref mut canvas = new_canvas(w, h);
            let states = bibwt.states();
            let mut interaction = Interaction::new(&states);
            loop {
                if bibwt.draw(interaction.get(&states), canvas) {
                    canvas.present();
                    interaction.wait();
                } else {
                    interaction.step();
                }
            }
        }
    }
}
