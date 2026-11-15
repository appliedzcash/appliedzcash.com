//! Demos for appliedzcash.com. Each pattern adds one module. CI builds the
//! crate for `wasm32-unknown-unknown` and runs `wasm-bindgen --target web`.

use wasm_bindgen::prelude::*;

/// The 16-byte BLAKE2b personalization for the pipeline check.
const PERSONAL: &[u8; 16] = b"appliedzcash.com";

/// Returns BLAKE2b-256 of `input` under the `appliedzcash.com` personalization, as hex.
#[wasm_bindgen]
pub fn hash(input: &str) -> String {
    blake2b_simd::Params::new()
        .hash_length(32)
        .personal(PERSONAL)
        .hash(input.as_bytes())
        .to_hex()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_reference_vector() {
        // Python: hashlib.blake2b(b"zcash", digest_size=32, person=b"appliedzcash.com").hexdigest()
        assert_eq!(
            hash("zcash"),
            "2b7d7fa7a29f741f377dfd277b1357a5ad3a2ec91896cbc935508e83a70aac87"
        );
    }
}
