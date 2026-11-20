//! Bound notes: a toy timestamp registry.
//!
//! Each stamp is a zero-value note to the registry address. Its `rseed` is
//! BLAKE2b-256(D, kind || len(fields) || fields || prev), where the fields are the hash
//! of a document and `prev` is the `cmx` of the stamp before it. ZIP 212 and ZIP 2005
//! derive ψ and rcm from `rseed`, so the note commitment binds the document.

use blake2b_simd::Params;
use orchard::{
    keys::{FullViewingKey, Scope, SpendingKey},
    note::{ExtractedNoteCommitment, Note, NoteVersion, RandomSeed, Rho},
    value::NoteValue,
};
use wasm_bindgen::prelude::*;

/// The binding domain D of the registry.
const DOMAIN: &[u8; 16] = b"TimeStamp_v1____";

/// The personalization of the document hash.
const DOC_PERSONAL: &[u8; 16] = b"TimeStamp_doc_v1";

/// The personalization of the demo value for the ρ of the first stamp.
const RHO_PERSONAL: &[u8; 16] = b"TimeStamp_rho_v1";

/// The kind of a stamp record.
const KIND_STAMP: u8 = 1;

fn blake2b(personal: &[u8; 16], parts: &[&[u8]]) -> [u8; 32] {
    let mut state = Params::new().hash_length(32).personal(personal).to_state();
    for part in parts {
        state.update(part);
    }
    state.finalize().as_bytes().try_into().expect("hash_length is 32")
}

/// rseed = BLAKE2b-256(D, I2LEOSP_8(kind) || I2LEOSP_64(len) || fields || prev).
fn bind_rseed(kind: u8, fields: &[u8], prev: &[u8; 32]) -> [u8; 32] {
    let len = (fields.len() as u64).to_le_bytes();
    blake2b(DOMAIN, &[&[kind], &len, fields, prev])
}

/// The demo key of the registry. A real registry publishes its full viewing key.
fn registry_key() -> FullViewingKey {
    let sk = SpendingKey::from_bytes([7; 32]).expect("a valid spending key");
    FullViewingKey::from(&sk)
}

fn decode32(hex_str: &str) -> Result<[u8; 32], JsError> {
    let mut bytes = [0; 32];
    hex::decode_to_slice(hex_str.trim(), &mut bytes)
        .map_err(|_| JsError::new("Use 64 hexadecimal characters."))?;
    Ok(bytes)
}

/// The values of one stamp, as hex.
#[wasm_bindgen(getter_with_clone)]
pub struct Stamp {
    pub doc_hash: String,
    pub rseed: String,
    pub cmx: String,
    /// The nullifier of the stamp. The next stamp spends this note in the same action,
    /// so this value is also the ρ of the next stamp.
    pub nf: String,
}

/// A demo value for the ρ of the first stamp: the nullifier of the note that its action
/// spends. The top two bits are zero, so the value is less than the Pallas base field modulus.
#[wasm_bindgen]
pub fn genesis_rho() -> String {
    let mut rho = blake2b(RHO_PERSONAL, &[b"genesis"]);
    rho[31] &= 0x3f;
    hex::encode(rho)
}

/// Makes the stamp of `document` with predecessor `prev` and ρ `rho`. `ironwood` selects
/// note plaintext lead byte 0x03 (Ironwood pool) or 0x02 (Orchard pool).
#[wasm_bindgen]
pub fn stamp(document: &str, prev: &str, rho: &str, ironwood: bool) -> Result<Stamp, JsError> {
    let prev = decode32(prev)?;
    let rho = Rho::from_bytes(&decode32(rho)?)
        .into_option()
        .ok_or_else(|| JsError::new("ρ is outside the Pallas base field."))?;
    let doc_hash = blake2b(DOC_PERSONAL, &[document.as_bytes()]);
    let rseed = bind_rseed(KIND_STAMP, &doc_hash, &prev);
    let version = if ironwood { NoteVersion::V3 } else { NoteVersion::V2 };

    let fvk = registry_key();
    let note = RandomSeed::from_bytes(rseed, &rho)
        .into_option()
        .and_then(|seed| {
            let recipient = fvk.address_at(0u32, Scope::External);
            Note::from_parts(recipient, NoteValue::ZERO, rho, seed, version).into_option()
        })
        .ok_or_else(|| JsError::new("No valid note has this rseed. Change the document."))?;

    Ok(Stamp {
        doc_hash: hex::encode(doc_hash),
        rseed: hex::encode(rseed),
        cmx: hex::encode(ExtractedNoteCommitment::from(note.commitment()).to_bytes()),
        nf: hex::encode(note.nullifier(&fvk).to_bytes()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENESIS_PREV: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn rseed_matches_reference_vector() {
        // Python: hashlib.blake2b(bytes([1]) + (3).to_bytes(8, "little") + b"abc" + bytes(32),
        //                         digest_size=32, person=b"TimeStamp_v1____").hexdigest()
        assert_eq!(
            hex::encode(bind_rseed(1, b"abc", &[0; 32])),
            "d2f399f655fe3681d102710498fda085bd3b00b7a1cccd69e191cc3c0fc79f03"
        );
    }

    #[test]
    fn stamp_is_deterministic() {
        let a = stamp("hello", GENESIS_PREV, &genesis_rho(), true).unwrap();
        let b = stamp("hello", GENESIS_PREV, &genesis_rho(), true).unwrap();
        assert_eq!(a.cmx, b.cmx);
        assert_eq!(a.nf, b.nf);
    }

    #[test]
    fn cmx_binds_document_prev_and_version() {
        let rho = genesis_rho();
        let base = stamp("hello", GENESIS_PREV, &rho, true).unwrap().cmx;
        assert_ne!(base, stamp("hellp", GENESIS_PREV, &rho, true).unwrap().cmx);
        assert_ne!(base, stamp("hello", &"01".repeat(32), &rho, true).unwrap().cmx);
        assert_ne!(base, stamp("hello", GENESIS_PREV, &rho, false).unwrap().cmx);
    }

    #[test]
    fn chain_links_prev_and_rho() {
        let first = stamp("one", GENESIS_PREV, &genesis_rho(), true).unwrap();
        let second = stamp("two", &first.cmx, &first.nf, true).unwrap();
        assert_ne!(first.cmx, second.cmx);
        assert_eq!(
            second.rseed,
            hex::encode(bind_rseed(
                KIND_STAMP,
                &decode32(&second.doc_hash).unwrap(),
                &decode32(&first.cmx).unwrap()
            ))
        );
    }
}
