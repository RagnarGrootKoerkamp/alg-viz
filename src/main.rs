use std::{path::PathBuf, sync::atomic::AtomicUsize, time::Duration};

use clap::Parser;
use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas,
    ttf::Sdl2TtfContext, video::Window, Sdl,
};

#[macro_use]
extern crate lazy_static;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HAlign {
    Left,
    Center,
    Right,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}

fn _make_label(text: &str, val: impl ToString) -> String {
    text.to_string() + &val.to_string()
}

const BACKGROUND: Color = Color::WHITE;
const LETTER_COLOUR: Color = Color::BLACK;
const SMALL_COLOUR: Color = Color::GREEN;
const LARGE_COLOUR: Color = Color::RGB(244, 113, 116);

#[derive(Parser)]
#[clap(author, about)]
struct Cli {
    /// String to run on.
    #[clap()]
    input: Option<String>,

    /// Where to optionally save image files.
    #[clap(short, long, parse(from_os_str))]
    save: Option<PathBuf>,
}

lazy_static! {
    static ref ARGS: Cli = Cli::parse();
    static ref TTF_CONTEXT: Sdl2TtfContext = sdl2::ttf::init().unwrap();
}

fn main() {
    let mut s = ARGS
        .input
        .clone()
        .unwrap_or("GTCCCGATGTCATGTCAGGA".to_owned());
    s.push('$');
    let s = s.as_bytes();
    let n = s.len();

    let sdl_context = &sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.gl_attr().set_double_buffer(true);

    let w = CS * (n as u32 + 4);
    let h = CS * (n as u32 + 5);

    let canvas = &mut video_subsystem
        .window("Suffix array extension", w as u32, h as u32)
        //.borderless()
        .build()
        .unwrap()
        .into_canvas()
        .build()
        .unwrap();
    let font = TTF_CONTEXT
        .load_font("/usr/share/fonts/TTF/OpenSans-Regular.ttf", 24)
        .unwrap();

    let write_label =
        |x: i32, y: i32, ha: HAlign, va: VAlign, canvas: &mut Canvas<Window>, text: &str| {
            let surface = font.render(text).blended(canvas.draw_color()).unwrap();
            let w = surface.width();
            let h = surface.height();
            let x = match ha {
                HAlign::Left => x,
                HAlign::Center => x - w as i32 / 2,
                HAlign::Right => x - w as i32,
            };
            let y = match va {
                VAlign::Top => y,
                VAlign::Center => y - h as i32 / 2,
                VAlign::Bottom => y - h as i32,
            };
            let texture_creator = canvas.texture_creator();
            canvas
                .copy(
                    &surface.as_texture(&texture_creator).unwrap(),
                    None,
                    Some(Rect::new(x, y, w, h)),
                )
                .unwrap();
        };

    let draw_background = |canvas: &mut Canvas<Window>| {
        canvas.set_draw_color(BACKGROUND);
        canvas.fill_rect(Rect::new(0, 0, w, h)).unwrap();
    };

    // Cell size
    const CS: u32 = 30;

    let draw_label = |x: usize, y: usize, label: &str, canvas: &mut Canvas<Window>| {
        let x = x as i32 * CS as i32;
        let y = y as i32 * CS as i32;
        canvas.set_draw_color(LETTER_COLOUR);
        write_label(
            x + CS as i32 / 2,
            y + CS as i32 / 2,
            HAlign::Center,
            VAlign::Center,
            canvas,
            label,
        );
    };

    let draw_char_box = |x: usize, y: usize, c: u8, color: Color, canvas: &mut Canvas<Window>| {
        let x = x as i32 * CS as i32;
        let y = y as i32 * CS as i32;
        canvas.set_draw_color(color);
        canvas.fill_rect(Rect::new(x, y, CS, CS)).unwrap();
        canvas.set_draw_color(Color::BLACK);
        canvas.draw_rect(Rect::new(x, y, CS, CS)).unwrap();
        canvas.set_draw_color(LETTER_COLOUR);
        let c: String = String::from_utf8(vec![c]).unwrap();
        write_label(
            x + CS as i32 / 2,
            y + CS as i32 / 2,
            HAlign::Center,
            VAlign::Center,
            canvas,
            &c,
        );
    };

    let is_small = |i| i == n - 1 || s[i..] < s[i + 1..];

    let draw_s = |canvas: &mut Canvas<Window>| {
        draw_label(2, 0, "i", canvas);
        draw_label(2, 1, "S", canvas);
        draw_char_box(0, 0, 'S' as u8, SMALL_COLOUR, canvas);
        draw_char_box(0, 1, 'L' as u8, LARGE_COLOUR, canvas);
        for (i, &c) in s.iter().enumerate() {
            draw_label(3 + i, 0, &i.to_string(), canvas);
            draw_char_box(
                3 + i,
                1,
                c,
                if is_small(i) {
                    SMALL_COLOUR
                } else {
                    LARGE_COLOUR
                },
                canvas,
            );
        }
    };

    let mut buckets = [0 as usize; 256];
    for &c in s.iter() {
        buckets[c as usize] += 1;
    }
    for i in 0..254 {
        buckets[i + 1] += buckets[i];
    }

    let mut final_sa: Vec<usize> = (0..n).collect();
    final_sa.sort_by_key(|&i| &s[i..]);

    let mut sa = vec![None; n];
    for j in 0..n {
        if is_small(final_sa[j]) {
            sa[j] = Some(final_sa[j]);
        }
    }

    let draw_sa = |sa: &Vec<Option<usize>>, canvas: &mut Canvas<Window>| {
        draw_label(0, 3, "j", canvas);
        draw_label(1, 3, "SA", canvas);
        for j in 0..n {
            draw_label(0, 4 + j, &j.to_string(), canvas);
            if let Some(i) = sa[j] {
                draw_label(1, 4 + j, &i.to_string(), canvas);
                for i2 in i..n {
                    draw_char_box(
                        3 + i2 - i,
                        4 + j,
                        s[i2],
                        if is_small(i2) {
                            SMALL_COLOUR
                        } else {
                            LARGE_COLOUR
                        },
                        canvas,
                    );
                }
            } else {
                // Find the first letter for this bucket
                let bucket = buckets
                    .iter()
                    .enumerate()
                    .find(|&(_, &cnt)| cnt > j)
                    .unwrap()
                    .0 as u8;
                draw_label(1, 4 + j, &'-'.to_string(), canvas);
                draw_char_box(3, 4 + j, bucket, LARGE_COLOUR, canvas);
            }
        }
    };

    let draw_highlight = |x: usize, y: usize, canvas: &mut Canvas<Window>| {
        let x = x as i32 * CS as i32;
        let y = y as i32 * CS as i32;
        let margin = 2;
        canvas
            .draw_rect(Rect::new(
                x + margin as i32,
                y + margin as i32,
                CS - 2 * margin,
                CS - 2 * margin,
            ))
            .unwrap();
        let margin = 1;
        canvas
            .draw_rect(Rect::new(
                x + margin as i32,
                y + margin as i32,
                CS - 2 * margin,
                CS - 2 * margin,
            ))
            .unwrap();
    };

    draw_background(canvas);
    draw_s(canvas);
    draw_sa(&sa, canvas);
    wait_for_key(false, canvas, sdl_context);

    for j in 0..n {
        canvas.clear();
        draw_background(canvas);
        draw_s(canvas);
        draw_sa(&sa, canvas);

        // highlight the current index i and the one before
        let i = sa[j].unwrap();
        canvas.set_draw_color(Color::RED);
        draw_highlight(1, 4 + j, canvas);
        draw_highlight(3 + i, 0, canvas);
        canvas.set_draw_color(Color::BLUE);
        draw_highlight(3 + i - 1, 0, canvas);

        wait_for_key(false, canvas, sdl_context);

        if i == 0 || is_small(i - 1) {
            continue;
        }

        // copied from above
        {
            canvas.clear();
            draw_background(canvas);
            draw_s(canvas);
            draw_sa(&sa, canvas);

            // highlight the current index i and the one before
            let i = sa[j].unwrap();
            canvas.set_draw_color(Color::RED);
            draw_highlight(1, 4 + j, canvas);
            draw_highlight(3 + i, 0, canvas);
            canvas.set_draw_color(Color::BLUE);
            draw_highlight(3 + i - 1, 0, canvas);
        }

        // highlight the new character, and the first empty position in that bucket
        canvas.set_draw_color(Color::BLUE);
        let c = s[i - 1];
        draw_highlight(3 + i - 1, 1, canvas);
        let new_j = (buckets[c as usize - 1]..buckets[c as usize])
            .find(|&j| sa[j].is_none())
            .unwrap();
        draw_highlight(1, 4 + new_j, canvas);

        wait_for_key(false, canvas, sdl_context);

        sa[new_j] = Some(i - 1);

        canvas.clear();
        draw_background(canvas);
        draw_s(canvas);
        draw_sa(&sa, canvas);

        wait_for_key(false, canvas, sdl_context);
    }

    canvas.clear();
    draw_background(canvas);
    draw_s(canvas);
    draw_sa(&sa, canvas);

    wait_for_key(true, canvas, sdl_context);
}

static mut SKIP_TO_END: bool = false;

fn wait_for_key(last: bool, canvas: &mut Canvas<Window>, sdl_context: &Sdl) {
    if let Some(mut path) = ARGS.save.clone() {
        static FRAME: AtomicUsize = AtomicUsize::new(0);

        let pixel_format = canvas.default_pixel_format();
        let mut pixels = canvas.read_pixels(canvas.viewport(), pixel_format).unwrap();
        let (width, height) = canvas.output_size().unwrap();
        let pitch = pixel_format.byte_size_of_pixels(width as usize);
        let surf = sdl2::surface::Surface::from_data(
            pixels.as_mut_slice(),
            width,
            height,
            pitch as u32,
            pixel_format,
        )
        .unwrap();

        std::fs::create_dir_all(&path).unwrap();
        let frame = FRAME.load(std::sync::atomic::Ordering::Acquire);
        path.push(format!("{frame}"));
        path.set_extension("bmp");
        surf.save_bmp(path).unwrap();
        FRAME.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
    canvas.present();

    if !last && unsafe { SKIP_TO_END } {
        return;
    }
    //Keyboard events
    let sleep_duration = 0.1;
    'outer: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => {
                    panic!("Running aborted by user!");
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Escape | Keycode::Space => {
                        if !last {
                            break 'outer;
                        }
                    }
                    Keycode::Q => {
                        unsafe {
                            SKIP_TO_END = true;
                        }
                        break 'outer;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::from_secs_f32(sleep_duration));
    }
}
