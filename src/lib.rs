use clap::Parser;
pub use sdl2::pixels::Color;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    rect::Rect,
    surface::Surface,
    ttf::{Font, Sdl2TtfContext},
    video::Window,
    Sdl,
};
use std::{
    cell::RefCell, collections::HashMap, path::PathBuf, sync::atomic::AtomicUsize, time::Duration,
};

pub type Canvas = sdl2::render::Canvas<Window>;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref ARGS: Cli = Cli::parse();
    static ref TTF_CONTEXT: Sdl2TtfContext = sdl2::ttf::init().unwrap();
}
thread_local! {
    static SDL_CONTEXT: Sdl = sdl2::init().unwrap();
    static FONT: Font<'static, 'static> = TTF_CONTEXT
        .load_font("/usr/share/fonts/TTF/OpenSans-Regular.ttf", 24)
        .unwrap();
}

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

pub fn to_label(c: u8) -> String {
    String::from_utf8(vec![c]).unwrap()
}
pub fn make_label(text: &str, val: impl ToString) -> String {
    text.to_string() + &val.to_string()
}

// Cell size
const CS: u32 = 30;
const BACKGROUND: Color = Color::WHITE;
const LETTER_COLOUR: Color = Color::BLACK;

#[derive(Parser)]
#[clap(author, about)]
pub struct Cli {
    /// String to run on.
    #[clap()]
    pub input: Option<String>,

    /// Query string for BWT.
    #[clap(short, long)]
    pub query: Option<String>,

    /// Where to optionally save image files.
    #[clap(short, long, parse(from_os_str))]
    pub save: Option<PathBuf>,
}

pub fn canvas(w: usize, h: usize) -> Canvas {
    let video_subsystem = SDL_CONTEXT.with(|sdl| sdl.video().unwrap());
    video_subsystem.gl_attr().set_double_buffer(true);

    video_subsystem
        .window("Suffix array extension", w as u32 * CS, h as u32 * CS)
        //.borderless()
        .build()
        .unwrap()
        .into_canvas()
        .build()
        .unwrap()
}

pub fn draw_background(canvas: &mut Canvas) {
    canvas.set_draw_color(BACKGROUND);
    canvas
        .fill_rect(Rect::new(
            0,
            0,
            canvas.output_size().unwrap().0,
            canvas.output_size().unwrap().1,
        ))
        .unwrap();
}

fn write_label(x: i32, y: i32, ha: HAlign, va: VAlign, text: &str, canvas: &mut Canvas) {
    thread_local! {
        static SURFACE_CACHE: RefCell<HashMap<String, Surface<'static>>> = RefCell::new(HashMap::new());
    }

    SURFACE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let surface = cache
            .entry(text.to_string())
            .or_insert(FONT.with(|front| front.render(text).blended(canvas.draw_color()).unwrap()));

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
    });
}

pub fn draw_label(x: usize, y: usize, label: &str, canvas: &mut Canvas) {
    let x = x as i32 * CS as i32;
    let y = y as i32 * CS as i32;
    canvas.set_draw_color(LETTER_COLOUR);
    write_label(
        x + CS as i32 / 2,
        y + CS as i32 / 2,
        HAlign::Center,
        VAlign::Center,
        label,
        canvas,
    );
}
pub fn draw_text(x: usize, y: usize, label: &str, canvas: &mut Canvas) {
    let x = x as i32 * CS as i32;
    let y = y as i32 * CS as i32;
    canvas.set_draw_color(LETTER_COLOUR);
    write_label(
        x,
        y + CS as i32 / 2,
        HAlign::Left,
        VAlign::Center,
        label,
        canvas,
    );
}

pub fn draw_char_box(x: usize, y: usize, c: u8, color: Color, canvas: &mut Canvas) {
    let x = x as i32 * CS as i32;
    let y = y as i32 * CS as i32;
    // background
    canvas.set_draw_color(color);
    canvas.fill_rect(Rect::new(x, y, CS, CS)).unwrap();
    // border
    canvas.set_draw_color(Color::BLACK);
    canvas.draw_rect(Rect::new(x, y, CS, CS)).unwrap();
    // letter
    canvas.set_draw_color(LETTER_COLOUR);
    write_label(
        x + CS as i32 / 2,
        y + CS as i32 / 2,
        HAlign::Center,
        VAlign::Center,
        &to_label(c),
        canvas,
    );
}

pub fn draw_highlight_box(
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: Color,
    canvas: &mut Canvas,
) {
    canvas.set_draw_color(color);
    let x = x as i32 * CS as i32;
    let y = y as i32 * CS as i32;
    if h == 0 {
        for margin in 0..=2 {
            canvas
                .draw_rect(Rect::new(
                    x as i32,
                    y - margin as i32,
                    w as u32 * CS,
                    2 * margin,
                ))
                .unwrap();
        }
    } else {
        for margin in 0..=2 {
            canvas
                .draw_rect(Rect::new(
                    x + margin as i32,
                    y + margin as i32,
                    w as u32 * CS - 2 * margin,
                    h as u32 * CS - 2 * margin,
                ))
                .unwrap();
        }
    }
}

// Draw a box around a cell.
pub fn draw_highlight(x: usize, y: usize, color: Color, canvas: &mut Canvas) {
    draw_highlight_box(x, y, 1, 1, color, canvas);
}

pub fn draw_string(
    x: usize,
    y: usize,
    s: &[u8],
    color: impl Fn(usize) -> Color,
    canvas: &mut Canvas,
) {
    for (i, &c) in s.iter().enumerate() {
        draw_char_box(x + i, y, c, color(i), canvas);
    }
}

pub fn draw_string_with_labels(
    x: usize,
    y: usize,
    s: &[u8],
    color: impl Fn(usize) -> Color,
    canvas: &mut Canvas,
) {
    draw_label(2, 0, "i", canvas);
    for i in 0..s.len() {
        draw_label(x + i, y - 1, &i.to_string(), canvas);
    }
    draw_label(x - 1, y, "S", canvas);
    draw_string(x, y, s, color, canvas);
}

static mut SKIP_TO_END: bool = false;

pub fn wait_for_key(canvas: &mut Canvas) {
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
        // NOTE: We can not use zero-padded ints since ffmpeg can't handle it.
        path.push(format!("{frame}"));
        path.set_extension("bmp");
        surf.save_bmp(path).unwrap();
        FRAME.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
    canvas.present();

    if unsafe { SKIP_TO_END } {
        return;
    }
    //Keyboard events
    let sleep_duration = 0.1;
    SDL_CONTEXT.with(|sdl| 'outer: loop {
        for event in sdl.event_pump().unwrap().poll_iter() {
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
                        break 'outer;
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
    });
}

pub fn wait_for_end() {
    // if let Some(mut path) = ARGS.save.clone() {
    //     static FRAME: AtomicUsize = AtomicUsize::new(0);

    //     let pixel_format = canvas.default_pixel_format();
    //     let mut pixels = canvas.read_pixels(canvas.viewport(), pixel_format).unwrap();
    //     let (width, height) = canvas.output_size().unwrap();
    //     let pitch = pixel_format.byte_size_of_pixels(width as usize);
    //     let surf = sdl2::surface::Surface::from_data(
    //         pixels.as_mut_slice(),
    //         width,
    //         height,
    //         pitch as u32,
    //         pixel_format,
    //     )
    //     .unwrap();

    //     std::fs::create_dir_all(&path).unwrap();
    //     let frame = FRAME.load(std::sync::atomic::Ordering::Acquire);
    //     path.push(format!("{frame:02}"));
    //     path.set_extension("bmp");
    //     surf.save_bmp(path).unwrap();
    //     FRAME.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    // }

    //Keyboard events
    let sleep_duration = 0.1;
    SDL_CONTEXT.with(|sdl| 'outer: loop {
        for event in sdl.event_pump().unwrap().poll_iter() {
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
                        // Nothing; this is the last frame.
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
    });
}
