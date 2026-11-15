//! Demos for appliedzcash.com. Each pattern adds one module. CI builds the
//! crate for `wasm32-unknown-unknown` and runs `wasm-bindgen --target web`.

mod bound_notes;
mod orders;

pub use bound_notes::{genesis_rho, stamp, Stamp};
pub use orders::{order, Order};
