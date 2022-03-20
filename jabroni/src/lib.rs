#![warn(missing_docs)]
//! Type-safe JavaScript-like embedded language

#[macro_use]
extern crate pest_derive;

mod binding;
pub mod errors;
mod state;
mod utils;
mod value;
pub use binding::{Binding, BindingMap};
pub use state::Jabroni;
pub use value::{Subroutine, Value};
