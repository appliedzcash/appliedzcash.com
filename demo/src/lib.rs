//! Demos for appliedzcash.com. Each pattern adds one module. CI builds the
//! crate for `wasm32-unknown-unknown` and runs `wasm-bindgen --target web`.

mod bound_notes;
mod orders;
mod pairing;

pub use bound_notes::{genesis_rho, stamp, Stamp};
pub use orders::{order, Order};
pub use pairing::{pair, Bundle};
