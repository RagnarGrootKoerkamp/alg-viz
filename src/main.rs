#![cfg(feature = "bin")]
#![feature(duration_constants)]

use suffix_array_construction::{
    bibwt, bwt,
    cli::{Algorithm, ARGS},
    grid::canvas_size,
    interaction::Interaction,
    sdl::new_canvas,
    suffix_array as sa,
};

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
            let mut interaction = Interaction::new(states.len());
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
            let mut interaction = Interaction::new(states.len());
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
            let mut interaction = Interaction::new(states.len());
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
