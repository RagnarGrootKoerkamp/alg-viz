pub type Color = (u8, u8, u8);
pub const BLACK: Color = (0, 0, 0);
pub const WHITE: Color = (255, 255, 255);
pub const RED: Color = (255, 0, 0);
pub const GREEN: Color = (0, 255, 0);
pub const BLUE: Color = (0, 0, 255);
pub const CYAN: Color = (0, 255, 255);

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

pub trait Canvas {
    fn fill_background(&mut self, color: Color);
    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color);
    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color);

    fn write_text(&mut self, x: i32, y: i32, ha: HAlign, va: VAlign, text: &str);

    fn save(&mut self);
}
