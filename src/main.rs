#![cfg(feature = "bin")]
#![feature(duration_constants)]

use suffix_array_construction::{
    bibwt, bwt,
    canvas::CanvasBox,
    cli::{Algorithm, ARGS},
    interaction::Interaction,
    sdl::new_canvas,
    suffix_array as sa,
    viz::Viz,
};

fn main() -> ! {
    let mut s = ARGS
        .input
        .clone()
        .unwrap_or("GTCCCGATGTCATGTCAGGA".to_owned());
    s.push('$');
    let s = s.into_bytes();
    let q = ARGS
        .query
        .clone()
        .unwrap_or("GTCC".to_string())
        .into_bytes();

    let alg = match ARGS.algorithm {
        Algorithm::SuffixArray => Box::new(sa::SA::new(s)) as Box<dyn Viz>,
        Algorithm::BWT => Box::new(bwt::BWT::new(s, q)) as Box<dyn Viz>,
        Algorithm::BiBWT => Box::new(bibwt::BiBWT::new(s, q)) as Box<dyn Viz>,
    };

    let (w, h) = alg.canvas_size();
    let ref mut canvas = Box::new(new_canvas(w, h)) as CanvasBox;
    let mut interaction = Interaction::new(alg.num_states());
    loop {
        if alg.draw(interaction.get(), canvas) {
            canvas.present();
            interaction.wait();
        } else {
            interaction.step();
        }
    }
}
