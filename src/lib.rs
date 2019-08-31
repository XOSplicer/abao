#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

//! # abao
//!
//! Append only array backed data structures
//!

// TODO: move to no_std

mod errors;
mod utils;
mod vec;

pub use errors::OomError;
pub use vec::AbaoVec;
