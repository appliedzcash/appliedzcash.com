//! Canonical action pairing: a keyed sort sets the dummy notes and the pairs of a bundle.
//!
//! The pairing seed d comes from the pairing key, a context and the real nullifiers. Each
//! dummy value comes from a PRF keyed by d. The spends sort by a keyed hash of their
//! nullifiers, and the outputs by a keyed hash of their rseeds. Action a holds spend a and
//! output a. Each output then takes its ρ from the nullifier in its action.
//!
//! The demo builds Ironwood-pool notes (lead byte 0x03). A "standard" build uses the same
//! steps with a random seed, as a builder that samples fresh randomness on each build.

use blake2b_simd::Params;
use orchard::{
    keys::{FullViewingKey, Scope, SpendingKey},
    note::{ExtractedNoteCommitment, Note, NoteVersion, RandomSeed, Rho},
    value::NoteValue,
    Address,
};
use pasta_curves::{
    group::ff::{FromUniformBytes, PrimeField},
    pallas,
};
use wasm_bindgen::prelude::*;

const SEED_PERSONAL: &[u8; 16] = b"Zcash_PairingSd_";
const PRF_PERSONAL: &[u8; 16] = b"Zcash_PairingPRF";
const ORDER_PERSONAL: &[u8; 16] = b"Zcash_PairingOrd";

/// The personalization of the demo values for the real spends and outputs.
const DEMO_PERSONAL: &[u8; 16] = b"ToyPair_demo_v1_";

const TAG_DUMMY_SPEND_KEY: u8 = 0x01;
const TAG_DUMMY_SPEND_RHO: u8 = 0x02;
const TAG_DUMMY_SPEND_RSEED: u8 = 0x03;
const TAG_OUTPUT_RSEED: u8 = 0x04;
const TAG_DUMMY_OUTPUT_KEY: u8 = 0x05;
const TAG_DUMMY_OUTPUT_RSEED: u8 = 0x06;

/// The most actions in a demo bundle.
const MAX_ACTIONS: u32 = 8;

fn blake2b<const N: usize>(personal: &[u8; 16], parts: &[&[u8]]) -> [u8; N] {
    let mut state = Params::new().hash_length(N).personal(personal).to_state();
    for part in parts {
        state.update(part);
    }
    state.finalize().as_bytes().try_into().expect("hash_length is N")
}

fn left32(bytes: &[u8; 64]) -> [u8; 32] {
    bytes[..32].try_into().expect("64 bytes hold 32")
}

/// d = BLAKE2b-256("Zcash_PairingSd_", k || I2LEOSP_64(len(ctx)) || ctx || I2LEOSP_32(m) || sorted nfs).
fn pairing_seed(key: &[u8; 32], ctx: &[u8], nullifiers: &[[u8; 32]]) -> [u8; 32] {
    let mut sorted = nullifiers.to_vec();
    sorted.sort();
    let ctx_len = (ctx.len() as u64).to_le_bytes();
    let m = (sorted.len() as u32).to_le_bytes();
    let mut parts: Vec<&[u8]> = vec![key, &ctx_len, ctx, &m];
    parts.extend(sorted.iter().map(|nf| nf.as_slice()));
    blake2b(SEED_PERSONAL, &parts)
}

/// PRF^pair_d(t, i, c) = BLAKE2b-512("Zcash_PairingPRF", d || [t] || I2LEOSP_32(i) || I2LEOSP_32(c)).
fn prf(d: &[u8; 32], tag: u8, i: u32, c: u32) -> [u8; 64] {
    blake2b(PRF_PERSONAL, &[d, &[tag], &i.to_le_bytes(), &c.to_le_bytes()])
}

/// PairingKey_d(t, i): the first valid spending key over c = 0, 1, 2, ...
fn pairing_key(d: &[u8; 32], tag: u8, i: u32) -> FullViewingKey {
    (0..)
        .find_map(|c| SpendingKey::from_bytes(left32(&prf(d, tag, i, c))).into_option())
        .map(|sk| FullViewingKey::from(&sk))
        .expect("a valid key occurs with probability near 1")
}

fn address(fvk: &FullViewingKey) -> Address {
    fvk.address_at(0u32, Scope::External)
}

/// ToBase^Orchard: a 64-byte string, little-endian, reduced modulo the Pallas base field.
fn to_base(bytes: &[u8; 64]) -> Rho {
    let base = pallas::Base::from_uniform_bytes(bytes);
    Rho::from_bytes(&base.to_repr()).expect("a reduced field element is canonical")
}

fn note(recipient: Address, rho: Rho, rseed: [u8; 32]) -> Option<Note> {
    let rseed = RandomSeed::from_bytes(rseed, &rho).into_option()?;
    Note::from_parts(recipient, NoteValue::ZERO, rho, rseed, NoteVersion::V3).into_option()
}

/// A demo value that stands for real data: 32 bytes, the top two bits zero, so the value
/// is a canonical Pallas base field element.
fn demo_value(label: &[u8], i: u32) -> [u8; 32] {
    let mut bytes: [u8; 32] = blake2b(DEMO_PERSONAL, &[label, &i.to_le_bytes()]);
    bytes[31] &= 0x3f;
    bytes
}

/// The demo recipient of the real outputs.
fn demo_recipient() -> Address {
    let sk = SpendingKey::from_bytes([11; 32]).expect("a valid spending key");
    address(&FullViewingKey::from(&sk))
}

struct Spend {
    label: String,
    nf: [u8; 32],
}

struct Output {
    label: String,
    recipient: Address,
    rseed: [u8; 32],
}

/// The actions of one build, as parallel lists of hex values and labels.
#[wasm_bindgen(getter_with_clone)]
pub struct Bundle {
    pub seed: String,
    pub spend_labels: Vec<String>,
    pub nullifiers: Vec<String>,
    pub output_labels: Vec<String>,
    pub cmxs: Vec<String>,
}

/// Builds a bundle of `actions` actions with `spends` real spends and `outputs` real
/// outputs. With `canonical`, the seed comes from the pairing key, the context and the real
/// nullifiers. Without it, the seed is `random`, as in a standard build.
#[wasm_bindgen]
pub fn pair(
    pairing_key_hex: &str,
    ctx: &str,
    spends: u32,
    outputs: u32,
    actions: u32,
    canonical: bool,
    random: &str,
) -> Result<Bundle, JsError> {
    build(pairing_key_hex, ctx, spends, outputs, actions, canonical, random)
        .map_err(|error| JsError::new(&error))
}

fn decode32(hex_str: &str, name: &str) -> Result<[u8; 32], String> {
    let mut bytes = [0; 32];
    hex::decode_to_slice(hex_str.trim(), &mut bytes)
        .map_err(|_| format!("The {name} must be 64 hexadecimal characters."))?;
    Ok(bytes)
}

fn build(
    pairing_key_hex: &str,
    ctx: &str,
    spends: u32,
    outputs: u32,
    actions: u32,
    canonical: bool,
    random: &str,
) -> Result<Bundle, String> {
    if actions > MAX_ACTIONS {
        return Err(format!("Use {MAX_ACTIONS} actions or fewer."));
    }
    if actions < spends.max(outputs).max(2) {
        return Err("Use 2 actions or more, and one action or more for each real spend and each real output.".into());
    }

    let real_nfs: Vec<[u8; 32]> = (0..spends).map(|i| demo_value(b"spend", i)).collect();
    let key = if canonical { Some(decode32(pairing_key_hex, "pairing key")?) } else { None };
    let random = decode32(random, "random seed")?;

    // Failed values: discard the build and repeat it with ctx || I2LEOSP_8(r), r = 1, 2, ...
    // Each party that follows this rule gets the same bundle.
    for r in 0..=u8::MAX {
        let d = match key {
            Some(key) => {
                let mut ctx = ctx.as_bytes().to_vec();
                if r > 0 {
                    ctx.push(r);
                }
                pairing_seed(&key, &ctx, &real_nfs)
            }
            // A standard build draws a new random seed for each try.
            None if r == 0 => random,
            None => blake2b(SEED_PERSONAL, &[&random, &[r]]),
        };
        match attempt(&d, &real_nfs, outputs, actions) {
            Ok(bundle) => return Ok(bundle),
            Err(Failure::Retry) => continue,
            Err(Failure::Fatal(error)) => return Err(error),
        }
    }
    Err("All 256 tries failed.".into())
}

enum Failure {
    /// A derived value failed: an esk of zero, or a note commitment of ⊥.
    Retry,
    Fatal(String),
}

fn attempt(d: &[u8; 32], real_nfs: &[[u8; 32]], outputs: u32, actions: u32) -> Result<Bundle, Failure> {
    let d = *d;
    let spends = real_nfs.len() as u32;

    // Real spends, then dummy spends from the PRF.
    let mut all_spends: Vec<Spend> = real_nfs
        .iter()
        .enumerate()
        .map(|(i, nf)| Spend { label: format!("real spend {}", i + 1), nf: *nf })
        .collect();
    for i in 0..actions - spends {
        let fvk = pairing_key(&d, TAG_DUMMY_SPEND_KEY, i);
        let rho = to_base(&prf(&d, TAG_DUMMY_SPEND_RHO, i, 0));
        let rseed = left32(&prf(&d, TAG_DUMMY_SPEND_RSEED, i, 0));
        let dummy = note(address(&fvk), rho, rseed).ok_or(Failure::Retry)?;
        all_spends.push(Spend { label: format!("dummy spend {}", i + 1), nf: dummy.nullifier(&fvk).to_bytes() });
    }

    // Real outputs with PRF seeds, then dummy outputs.
    let mut all_outputs: Vec<Output> = (0..outputs)
        .map(|j| Output {
            label: format!("real output {}", j + 1),
            recipient: demo_recipient(),
            rseed: left32(&prf(&d, TAG_OUTPUT_RSEED, j, 0)),
        })
        .collect();
    for i in 0..actions - outputs {
        all_outputs.push(Output {
            label: format!("dummy output {}", i + 1),
            recipient: address(&pairing_key(&d, TAG_DUMMY_OUTPUT_KEY, i)),
            rseed: left32(&prf(&d, TAG_DUMMY_OUTPUT_RSEED, i, 0)),
        });
    }

    // Sort each list by its keyed sort key. Equal keys fail the build.
    let sort_key = |kind: u8, value: &[u8; 32]| -> [u8; 32] { blake2b(ORDER_PERSONAL, &[&d, &[kind], value]) };
    all_spends.sort_by_cached_key(|spend| sort_key(0x00, &spend.nf));
    all_outputs.sort_by_cached_key(|output| sort_key(0x01, &output.rseed));
    let distinct = |keys: Vec<[u8; 32]>| keys.windows(2).all(|pair| pair[0] != pair[1]);
    if !distinct(all_spends.iter().map(|s| sort_key(0x00, &s.nf)).collect())
        || !distinct(all_outputs.iter().map(|o| sort_key(0x01, &o.rseed)).collect())
    {
        return Err(Failure::Fatal("Two sort keys are equal. Thus, the build is invalid.".into()));
    }

    // Action a: spend a and output a. The output takes ρ from the spend's nullifier.
    let mut cmxs = Vec::new();
    for (spend, output) in all_spends.iter().zip(&all_outputs) {
        let rho = Rho::from_bytes(&spend.nf)
            .into_option()
            .ok_or_else(|| Failure::Fatal("A nullifier is outside the field.".into()))?;
        let note = note(output.recipient, rho, output.rseed).ok_or(Failure::Retry)?;
        cmxs.push(hex::encode(ExtractedNoteCommitment::from(note.commitment()).to_bytes()));
    }

    Ok(Bundle {
        seed: hex::encode(d),
        spend_labels: all_spends.iter().map(|s| s.label.clone()).collect(),
        nullifiers: all_spends.iter().map(|s| hex::encode(s.nf)).collect(),
        output_labels: all_outputs.into_iter().map(|o| o.label).collect(),
        cmxs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "0202020202020202020202020202020202020202020202020202020202020202";
    const RANDOM_A: &str = "0303030303030303030303030303030303030303030303030303030303030303";
    const RANDOM_B: &str = "0404040404040404040404040404040404040404040404040404040404040404";

    #[test]
    fn seed_matches_reference_vector() {
        // Python: hashlib.blake2b(bytes([2] * 32) + (3).to_bytes(8, "little") + b"abc"
        //                         + (1).to_bytes(4, "little") + bytes(32),
        //                         digest_size=32, person=b"Zcash_PairingSd_").hexdigest()
        assert_eq!(hex::encode(pairing_seed(&[2; 32], b"abc", &[[0; 32]])), "d9b66525aa590998f6450b0230a3070e5598b543240404978ced8a9fa70eca69");
    }

    #[test]
    fn canonical_builds_repeat() {
        let a = build(KEY, "epoch 1", 2, 1, 4, true, RANDOM_A).unwrap();
        let b = build(KEY, "epoch 1", 2, 1, 4, true, RANDOM_B).unwrap();
        assert_eq!(a.nullifiers, b.nullifiers);
        assert_eq!(a.cmxs, b.cmxs);
        assert_eq!(a.cmxs.len(), 4);
    }

    #[test]
    fn standard_builds_change() {
        let a = build(KEY, "epoch 1", 2, 1, 4, false, RANDOM_A).unwrap();
        let b = build(KEY, "epoch 1", 2, 1, 4, false, RANDOM_B).unwrap();
        assert_ne!(a.cmxs, b.cmxs);
    }

    #[test]
    fn key_and_context_change_the_bundle() {
        let base = build(KEY, "epoch 1", 2, 1, 4, true, RANDOM_A).unwrap().cmxs;
        assert_ne!(base, build(KEY, "epoch 2", 2, 1, 4, true, RANDOM_A).unwrap().cmxs);
        assert_ne!(base, build(RANDOM_B, "epoch 1", 2, 1, 4, true, RANDOM_A).unwrap().cmxs);
    }

    #[test]
    fn real_spends_keep_their_nullifiers() {
        let bundle = build(KEY, "epoch 1", 2, 1, 4, true, RANDOM_A).unwrap();
        for i in 0..2 {
            let label = format!("real spend {}", i + 1);
            let at = bundle.spend_labels.iter().position(|l| *l == label).unwrap();
            assert_eq!(bundle.nullifiers[at], hex::encode(demo_value(b"spend", i)));
        }
    }

    #[test]
    fn bad_sizes_fail() {
        assert!(build(KEY, "", 3, 1, 2, true, RANDOM_A).is_err());
        assert!(build(KEY, "", 0, 0, 1, true, RANDOM_A).is_err());
        assert!(build(KEY, "", 1, 1, 9, true, RANDOM_A).is_err());
    }
}
