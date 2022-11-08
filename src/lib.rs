#![feature(let_else, duration_constants)]

pub mod bibwt;
pub mod bwt;
pub mod canvas;
pub mod grid;
pub mod interaction;
pub mod suffix_array;

// Deps for command line tool

#[cfg(feature = "bin")]
pub mod cli;
#[cfg(feature = "bin")]
pub mod sdl;

#[cfg(feature = "bin")]
#[macro_use]
extern crate lazy_static;

// Deps for HTML canvas rendering
#[cfg(feature = "wasm")]
pub mod html;
