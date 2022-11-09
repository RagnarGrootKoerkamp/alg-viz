pub mod bibwt;
pub mod bwt;
pub mod suffix_array;

use crate::canvas::Canvas;

pub trait Viz {
    fn canvas_size(&self) -> (usize, usize);
    fn num_states(&self) -> usize;
    fn draw(&self, state: usize, canvas: &mut Box<dyn Canvas>) -> bool;
}
