#![feature(let_else, duration_constants, local_key_cell_methods)]

pub mod alg;
pub mod canvas;
pub mod interaction;

// Deps for command line tool

#[cfg(feature = "bin")]
pub mod cli;

#[cfg(feature = "bin")]
#[macro_use]
extern crate lazy_static;
