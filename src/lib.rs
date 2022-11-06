#![feature(let_else)]

pub mod bibwt;
pub mod bwt;
pub mod canvas;
pub mod grid;
pub mod suffix_array;

// Deps for command line tool

#[cfg(not(feature = "wasm"))]
pub mod cli;
#[cfg(not(feature = "wasm"))]
pub mod sdl;

#[cfg(not(feature = "wasm"))]
#[macro_use]
extern crate lazy_static;

// Deps for HTML canvas rendering
#[cfg(feature = "wasm")]
pub mod html;
