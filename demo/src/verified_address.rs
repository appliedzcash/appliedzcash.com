//! Verified address: a toy directory.
//!
//! After a correct login code, the service writes one record: a zero-value bound note in
//! its directory chain. The fields of the record are the claim, a hash of a secret salt and
//! the address. A holder of the salt and the address can calculate the `cmx` of the record
//! again.

use orchard::note::NoteVersion;
use wasm_bindgen::prelude::*;
use zcash_address::ZcashAddress;

use crate::bound_notes::{blake2b, bound_note, decode32};

/// The binding domain D of the directory.
const DOMAIN: &[u8; 16] = b"VerifiedAddr_v1_";

/// The personalization of the claim hash.
const CLAIM_PERSONAL: &[u8; 16] = b"VerifiedAddr_clm";

/// The kind of a "verified" record.
const KIND_VERIFIED: u8 = 1;

/// claim = BLAKE2b-256("VerifiedAddr_clm", salt || address). The salt has a fixed length,
/// so the encoding is injective.
fn claim(salt: &[u8; 32], address: &str) -> [u8; 32] {
    blake2b(CLAIM_PERSONAL, &[salt.as_slice(), address.as_bytes()])
}

/// The values of one record, as hex.
#[wasm_bindgen(getter_with_clone)]
pub struct VerifiedRecord {
    pub claim: String,
    pub rseed: String,
    pub cmx: String,
    /// The nullifier of the record. It is also the ρ of the next record.
    pub nf: String,
}

/// Makes the record of `address` with salt `salt`, predecessor `prev` and ρ `rho`. The
/// record is an Ironwood-pool note.
#[wasm_bindgen]
pub fn verified_record(address: &str, salt: &str, prev: &str, rho: &str) -> Result<VerifiedRecord, JsError> {
    build(address, salt, prev, rho).map_err(|error| JsError::new(&error))
}

fn build(address: &str, salt: &str, prev: &str, rho: &str) -> Result<VerifiedRecord, String> {
    let address = address.trim();
    ZcashAddress::try_from_encoded(address).map_err(|_| "The address is not a valid Zcash address.".to_string())?;
    let salt = decode32(salt).map_err(|_| "The salt must be 64 hexadecimal characters.".to_string())?;
    let claim = claim(&salt, address);
    let note = bound_note(DOMAIN, KIND_VERIFIED, &claim, &decode32(prev)?, rho, NoteVersion::V3)?
        .ok_or_else(|| "No valid note has this rseed. Make a new salt.".to_string())?;

    Ok(VerifiedRecord {
        claim: hex::encode(claim),
        rseed: hex::encode(note.rseed),
        cmx: hex::encode(note.cmx),
        nf: hex::encode(note.nf),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis_rho;

    const GENESIS_PREV: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    const SALT: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const USER: &str = "u1j8gmxccf6g59x2er94h7ym959ehr7dmavhsk5vu9lu3wmtltffcwdlc7pa32mj0dxtrcans43hr6sl8d3w4mhn6klxry4w3kz6lkzwphvthrhwak506ah6fzmxzlsaeu7ghky3xcq23nt6fhwrnwc8rtupggntn4hfhjprfxtvaz6nkx";

    #[test]
    fn claim_matches_reference_vector() {
        // Python: hashlib.blake2b(bytes(range(32)) + b"u1example-return-address",
        //                         digest_size=32, person=b"VerifiedAddr_clm").hexdigest()
        let salt: [u8; 32] = core::array::from_fn(|i| i as u8);
        assert_eq!(
            hex::encode(claim(&salt, "u1example-return-address")),
            "7a5ba9b49fa67d66c0f2c57ade10a349815cd5bc8d936d9d3e82bdcc34d7e56b"
        );
    }

    #[test]
    fn cmx_binds_address_salt_and_prev() {
        let rho = genesis_rho();
        let base = build(USER, SALT, GENESIS_PREV, &rho).unwrap();
        assert_eq!(base.cmx, build(&format!(" {USER} "), SALT, GENESIS_PREV, &rho).unwrap().cmx);
        assert_ne!(base.cmx, build(USER, &"01".repeat(32), GENESIS_PREV, &rho).unwrap().cmx);
        assert_ne!(base.cmx, build(USER, SALT, &"01".repeat(32), &rho).unwrap().cmx);
        assert_ne!(base.claim, build("t1Hsc1LR8yKnbbe3twRp88p6vFfC5t7DLbs", SALT, GENESIS_PREV, &rho).unwrap().claim);
    }

    #[test]
    fn chain_links_prev_and_rho() {
        let first = build(USER, SALT, GENESIS_PREV, &genesis_rho()).unwrap();
        let second = build(USER, &"02".repeat(32), &first.cmx, &first.nf).unwrap();
        assert_ne!(first.cmx, second.cmx);
        assert_ne!(first.nf, second.nf);
    }

    #[test]
    fn bad_input_fails() {
        let rho = genesis_rho();
        assert!(build("zcash", SALT, GENESIS_PREV, &rho).is_err());
        assert!(build(USER, "00", GENESIS_PREV, &rho).is_err());
        assert!(build(USER, SALT, "00", &rho).is_err());
        assert!(build(USER, SALT, GENESIS_PREV, &"ff".repeat(32)).is_err());
    }
}
