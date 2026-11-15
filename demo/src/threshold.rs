//! Threshold keys: FROST (RedPallas) with a 2-of-3 group nested in a 2-of-2 object key.
//!
//! A group of three members shares a long-term secret `a_G`, any two of whom sign. Each
//! object gets a public tweak t, so the group's key for the object is `A_G + [t]G`, and the
//! members stay offline when an object is registered. An owner holds the other share of a
//! FROST 2-of-2 key: `ak = ±([2]A_O − (A_G + [t]G))`, interpolated at zero over identifiers
//! 1 (owner) and 2 (group).
//!
//! To sign, two members each send the hash of their nonce commitments, then reveal them;
//! each member checks every reveal against its hash. The group's outer commitment is their
//! sum. Each member answers `z_j = d_j + ρ₂·e_j + λ₂·c·(±μ_j·σ_j)`, the coordinator checks
//! each answer, sums them and adds `λ₂·c·(±t + α)` once. frost-core then aggregates the sum
//! as identifier 2's share. This follows the nested FROST recipe of the `frostito` crate.
//! It is unaudited and experimental.

use std::collections::BTreeMap;

use blake2b_simd::Params;
use frost_core::{
    self as fc, compute_binding_factor_list, compute_group_commitment, derive_interpolating_value,
};
use frost_rerandomized::{RandomizedParams, Randomizer};
use pasta_curves::{
    group::ff::{FromUniformBytes, PrimeField},
    pallas,
};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use reddsa::frost::redpallas::{
    self as frost, Field, Group, Identifier, PallasBlake2b512, PallasGroup, PallasScalarField,
    Signature, SigningPackage, VerifyingKey,
    keys::{generate_with_dealer, IdentifierList, KeyPackage, PublicKeyPackage, SigningShare, VerifyingShare},
    round1::{NonceCommitment, SigningCommitments, SigningNonces},
    round2::SignatureShare,
};
use wasm_bindgen::prelude::*;

type Scalar = <PallasScalarField as Field>::Scalar;
type Point = <PallasGroup as Group>::Element;

const TWEAK_PERSONAL: &[u8; 16] = b"ToyKeys_tweak_v1";
const MESSAGE_PERSONAL: &[u8; 16] = b"ToyKeys_msg_v1__";
const SESSION_PERSONAL: &[u8; 16] = b"ToyKeys_sess_v1_";
const NEST_PERSONAL: &[u8; 16] = b"ToyKeys_nest_v1_";
const SET_PERSONAL: &[u8; 16] = b"ToyKeys_nset_v1_";

const OWNER: u16 = 1;
const GROUP: u16 = 2;
const THRESHOLD: u16 = 2;
const MEMBERS: u16 = 3;

fn id(n: u16) -> Identifier {
    Identifier::try_from(n).expect("nonzero identifiers are valid")
}

fn bytes(point: &Point) -> Result<[u8; 32], String> {
    PallasGroup::serialize(point).map_err(|_| "A point is the identity.".to_string())
}

fn point(bytes: &[u8; 32]) -> Result<Point, String> {
    PallasGroup::deserialize(bytes).map_err(|_| "A point is invalid.".to_string())
}

fn scalar_bytes(s: &Scalar) -> [u8; 32] {
    PallasScalarField::serialize(s)
}

fn blake2b<const N: usize>(personal: &[u8; 16], data: &[u8]) -> [u8; N] {
    Params::new().hash_length(N).personal(personal).hash(data).as_bytes().try_into().expect("hash_length is N")
}

/// The public tweak t of an object: ToScalar(BLAKE2b-512("ToyKeys_tweak_v1", object id)).
fn tweak(object_id: &str) -> Scalar {
    let wide: [u8; 64] = blake2b(TWEAK_PERSONAL, object_id.as_bytes());
    let reduced = pallas::Scalar::from_uniform_bytes(&wide).to_repr();
    PallasScalarField::deserialize(&reduced).expect("a reduced scalar is canonical")
}

/// μ_j: member `j`'s Lagrange coefficient at zero over `signers`.
fn lagrange(j: u16, signers: &[u16]) -> Result<Scalar, String> {
    let x = Scalar::from(u64::from(j));
    let (mut num, mut den) = (PallasScalarField::one(), PallasScalarField::one());
    for &other in signers.iter().filter(|&&o| o != j) {
        let o = Scalar::from(u64::from(other));
        num *= o;
        den *= o - x;
    }
    Ok(num * PallasScalarField::invert(&den).map_err(|_| "Two signers share an identifier.")?)
}

/// The hash a member sends before it reveals its commitments, so that no member can choose
/// its nonces after it sees the nonces of the others.
fn commitment_hash(session: &[u8; 32], j: u16, commitments: &SigningCommitments) -> Result<[u8; 32], String> {
    let encoded = commitments.serialize().map_err(|_| "A commitment failed to encode.")?;
    Ok(blake2b(NEST_PERSONAL, &[session.as_slice(), &j.to_le_bytes(), &encoded].concat()))
}

/// The digest of the commitment set: each chosen member and its hash. Each member signs it
/// at reveal, so the coordinator shows one set to all members.
fn set_digest(session: &[u8; 32], hashes: &BTreeMap<u16, [u8; 32]>) -> [u8; 32] {
    let mut data = session.to_vec();
    for (j, h) in hashes {
        data.extend_from_slice(&j.to_le_bytes());
        data.extend_from_slice(h);
    }
    blake2b(SET_PERSONAL, &data)
}

/// Each member's check at reveal: every revealed commitment matches the hash sent before it,
/// and the set is exactly the hashed set.
fn check_reveal(
    session: &[u8; 32],
    hashes: &BTreeMap<u16, [u8; 32]>,
    revealed: &BTreeMap<u16, SigningCommitments>,
) -> Result<(), String> {
    if !hashes.keys().eq(revealed.keys()) {
        return Err("The revealed set differs from the hashed set.".into());
    }
    for (j, c) in revealed {
        if commitment_hash(session, *j, c)? != hashes[j] {
            return Err(format!("The commitments of member {j} differ from its hash."));
        }
    }
    Ok(())
}

/// The public side of the object key.
struct ObjectKey {
    /// ak, with sign bit 0 as Orchard requires.
    ak: Point,
    /// The owner's verifying share A_O.
    owner: Point,
    /// The group's verifying share for this object: A_G + [t]G.
    group: Point,
    /// True when both sides sign negated, so that ak has sign bit 0.
    negated: bool,
}

impl ObjectKey {
    fn derive(owner: Point, group_key: Point, t: Scalar) -> Result<Self, String> {
        let group = group_key + PallasGroup::generator() * t;
        // Lagrange interpolation at zero over identifiers 1 and 2: coefficients 2 and −1.
        let interpolated = owner + owner - group;
        let negated = bytes(&interpolated)?[31] & 0x80 != 0;
        let ak = if negated { -interpolated } else { interpolated };
        Ok(ObjectKey { ak, owner, group, negated })
    }

    fn sign(&self) -> Scalar {
        if self.negated { -PallasScalarField::one() } else { PallasScalarField::one() }
    }

    fn share(&self, p: Point) -> Result<VerifyingShare, String> {
        let p = if self.negated { -p } else { p };
        VerifyingShare::deserialize(&bytes(&p)?).map_err(|_| "A verifying share is invalid.".to_string())
    }

    fn verifying_key(&self) -> Result<VerifyingKey, String> {
        VerifyingKey::deserialize(&bytes(&self.ak)?).map_err(|_| "ak is invalid.".to_string())
    }

    fn public_package(&self) -> Result<PublicKeyPackage, String> {
        let shares = BTreeMap::from([(id(OWNER), self.share(self.owner)?), (id(GROUP), self.share(self.group)?)]);
        Ok(PublicKeyPackage::new(shares, self.verifying_key()?, Some(2)))
    }
}

fn fail<E>(_: E) -> String {
    "The outer round failed.".to_string()
}

/// ρ₂, c and λ₂ of the outer 2-of-2 round for identifier 2, over rk = ak + [α]G.
fn outer(
    key: &ObjectKey,
    package: &SigningPackage,
    alpha: Scalar,
) -> Result<(Scalar, Scalar, Scalar), String> {
    let rk = VerifyingKey::deserialize(&bytes(&(key.ak + PallasGroup::generator() * alpha))?).map_err(fail)?;
    let factors = compute_binding_factor_list::<PallasBlake2b512>(package, &rk, &[]).map_err(fail)?;
    let rho = PallasScalarField::deserialize(
        &factors.get(&id(GROUP)).ok_or("The group has no binding factor.")?.serialize().try_into().map_err(|_| "A binding factor is malformed.")?,
    )
    .map_err(fail)?;
    let commitment = compute_group_commitment(package, &factors).map_err(fail)?;
    let challenge = fc::challenge::<PallasBlake2b512>(&commitment.to_element(), &rk, package.message()).map_err(fail)?;
    let lambda = derive_interpolating_value(&id(GROUP), package).map_err(fail)?;
    Ok((rho, challenge.to_scalar(), lambda))
}

/// What one run shows, as hex.
#[wasm_bindgen(getter_with_clone)]
pub struct Threshold {
    pub group_key: String,
    pub member_keys: Vec<String>,
    pub tweak: String,
    pub object_group_key: String,
    pub owner_key: String,
    pub ak: String,
    pub negated: bool,
    pub commitment_hashes: Vec<String>,
    pub set_digest: String,
    pub alpha: String,
    pub rk: String,
    pub signature: String,
    pub verified: bool,
}

/// Deals a 2-of-3 group key and an owner share from `key_seed`, derives the object key for
/// `object_id`, and signs `message` with the owner and the members in `signers`. The nonces
/// and the randomizer α come from `nonce_seed`.
#[wasm_bindgen]
pub fn threshold(
    key_seed: &str,
    nonce_seed: &str,
    object_id: &str,
    signers: Vec<u16>,
    message: &str,
) -> Result<Threshold, JsError> {
    run(key_seed, nonce_seed, object_id, &signers, message).map_err(|e| JsError::new(&e))
}

fn seed(hex_str: &str) -> Result<[u8; 32], String> {
    let mut out = [0; 32];
    hex::decode_to_slice(hex_str.trim(), &mut out).map_err(|_| "A seed must be 64 hexadecimal characters.")?;
    Ok(out)
}

fn run(key_seed: &str, nonce_seed: &str, object_id: &str, signers: &[u16], message: &str) -> Result<Threshold, String> {
    let mut signers = signers.to_vec();
    signers.sort_unstable();
    signers.dedup();
    if signers.iter().any(|&j| j == 0 || j > MEMBERS) {
        return Err("Members are 1, 2 and 3.".into());
    }
    if signers.len() < usize::from(THRESHOLD) {
        return Err(format!("The group needs {THRESHOLD} members to sign. Select {THRESHOLD} or more."));
    }

    // Keygen: a trusted dealer splits a_G 2-of-3. A real group uses a DKG.
    let mut keys = ChaCha20Rng::from_seed(seed(key_seed)?);
    let (shares, public) = generate_with_dealer(MEMBERS, THRESHOLD, IdentifierList::Default, &mut keys)
        .map_err(|_| "Keygen failed.")?;
    let group_key = public.verifying_key().to_element();
    let member_secret = |j: u16| -> Result<Scalar, String> {
        Ok(shares.get(&id(j)).ok_or("A member share is missing.")?.signing_share().to_scalar())
    };
    let member_public = |j: u16| -> Result<Point, String> {
        Ok(public.verifying_shares().get(&id(j)).ok_or("A member key is missing.")?.to_element())
    };
    let owner_secret = loop {
        let s = PallasScalarField::random(&mut keys);
        if s != PallasScalarField::zero() {
            break s;
        }
    };

    // The object key: public tweak, then the 2-of-2 interpolation.
    let t = tweak(object_id);
    let key = ObjectKey::derive(PallasGroup::generator() * owner_secret, group_key, t)?;
    let sighash: [u8; 32] = blake2b(MESSAGE_PERSONAL, message.as_bytes());

    // Round 1: the owner and each chosen member make a nonce pair and its commitments.
    let mut rng = ChaCha20Rng::from_seed(seed(nonce_seed)?);
    let alpha = PallasScalarField::random(&mut rng);
    let mut session_seed = [0u8; 32];
    rand_core::RngCore::fill_bytes(&mut rng, &mut session_seed);
    let session: [u8; 32] = blake2b(SESSION_PERSONAL, &[sighash, session_seed].concat());
    let owner_share = SigningShare::deserialize(&scalar_bytes(&owner_secret)).map_err(|_| "The owner share is invalid.")?;
    let (owner_nonces, owner_commit) = frost::round1::commit(&owner_share, &mut rng);
    let mut nonces: BTreeMap<u16, SigningNonces> = BTreeMap::new();
    let mut commitments: BTreeMap<u16, SigningCommitments> = BTreeMap::new();
    for &j in &signers {
        let share = SigningShare::deserialize(&scalar_bytes(&member_secret(j)?)).map_err(|_| "A share is invalid.")?;
        let (n, c) = frost::round1::commit(&share, &mut rng);
        nonces.insert(j, n);
        commitments.insert(j, c);
    }

    // Inside the group: each member first sends the hash of its commitments.
    let hashes: BTreeMap<u16, [u8; 32]> =
        commitments.iter().map(|(j, c)| Ok((*j, commitment_hash(&session, *j, c)?))).collect::<Result<_, String>>()?;
    let digest = set_digest(&session, &hashes);
    // Then each member reveals its commitments. Each member checks every commitment against
    // its hash before it continues.
    let revealed = commitments;
    check_reveal(&session, &hashes, &revealed)?;

    // The group's outer commitment: the sum of the revealed commitments.
    let (mut hiding, mut binding) = (PallasGroup::identity(), PallasGroup::identity());
    for c in revealed.values() {
        hiding += c.hiding().value();
        binding += c.binding().value();
    }
    let group_commit = SigningCommitments::new(NonceCommitment::new(hiding), NonceCommitment::new(binding));
    let package = SigningPackage::new(BTreeMap::from([(id(OWNER), owner_commit), (id(GROUP), group_commit)]), &sighash);

    // Round 2, inside the group: each member answers, and the coordinator checks the answer.
    let (rho, c, lambda) = outer(&key, &package, alpha)?;
    let mut sum = PallasScalarField::zero();
    for &j in &signers {
        let mu = lagrange(j, &signers)?;
        let n = &nonces[&j];
        let z = n.hiding().to_scalar() + n.binding().to_scalar() * rho + lambda * c * key.sign() * mu * member_secret(j)?;
        let r = &revealed[&j];
        let expected = r.hiding().value() + r.binding().value() * rho + member_public(j)? * (lambda * c * key.sign() * mu);
        if PallasGroup::generator() * z != expected {
            return Err(format!("The answer of member {j} failed its check."));
        }
        sum += z;
    }
    let group_share = SignatureShare::deserialize(&scalar_bytes(&(sum + lambda * c * (key.sign() * t + alpha))))
        .map_err(|_| "The group share is invalid.")?;

    // Round 2, outside: the owner signs as identifier 1; frost-core aggregates both shares.
    let owner_scalar = if key.negated { -owner_secret } else { owner_secret };
    let owner_package = KeyPackage::new(
        id(OWNER),
        SigningShare::deserialize(&scalar_bytes(&owner_scalar)).map_err(|_| "The owner share is invalid.")?,
        key.share(key.owner)?,
        key.verifying_key()?,
        2,
    );
    let randomizer = Randomizer::deserialize(&scalar_bytes(&alpha)).map_err(|_| "α is invalid.")?;
    #[allow(deprecated)]
    let owner_sig = frost_rerandomized::sign(&package, &owner_nonces, &owner_package, randomizer)
        .map_err(|_| "The owner share failed.")?;
    let params = RandomizedParams::from_randomizer(&key.verifying_key()?, randomizer);
    let signature = frost::rerandomized::aggregate(
        &package,
        &BTreeMap::from([(id(OWNER), owner_sig), (id(GROUP), group_share)]),
        &key.public_package()?,
        &params,
    )
    .map_err(|_| "The aggregate signature failed.")?;

    // Anyone checks the signature under rk = ak + [α]G.
    let rk = key.ak + PallasGroup::generator() * alpha;
    let rk_key = VerifyingKey::deserialize(&bytes(&rk)?).map_err(|_| "rk is invalid.")?;
    let signature_bytes = signature.serialize().map_err(|_| "The signature failed to encode.")?;
    let verified = rk_key.verify(&sighash, &Signature::deserialize(&signature_bytes).map_err(|_| "Bad signature.")?).is_ok();

    Ok(Threshold {
        group_key: hex::encode(bytes(&group_key)?),
        member_keys: (1..=MEMBERS).map(|j| Ok(hex::encode(bytes(&member_public(j)?)?))).collect::<Result<_, String>>()?,
        tweak: hex::encode(scalar_bytes(&t)),
        object_group_key: hex::encode(bytes(&key.group)?),
        owner_key: hex::encode(bytes(&key.owner)?),
        ak: hex::encode(bytes(&key.ak)?),
        negated: key.negated,
        commitment_hashes: hashes.iter().map(|(j, h)| format!("member {j}: {}", hex::encode(h))).collect(),
        set_digest: hex::encode(digest),
        alpha: hex::encode(scalar_bytes(&alpha)),
        rk: hex::encode(bytes(&rk)?),
        signature: hex::encode(signature_bytes),
        verified,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: &str = "0505050505050505050505050505050505050505050505050505050505050505";
    const NONCES: &str = "0606060606060606060606060606060606060606060606060606060606060606";

    #[test]
    fn owner_and_any_two_members_sign() {
        let mut seen = [false; 2];
        for object in 0..16 {
            let object = format!("object {object}");
            for signers in [vec![1, 2], vec![1, 3], vec![2, 3], vec![1, 2, 3]] {
                let out = run(KEYS, NONCES, &object, &signers, "transfer").unwrap();
                assert!(out.verified, "{object}, {signers:?}");
                assert_eq!(hex::decode(&out.ak).unwrap()[31] & 0x80, 0);
                seen[usize::from(out.negated)] = true;
            }
        }
        // Both key signs occur, so the negation path runs.
        assert_eq!(seen, [true, true]);
    }

    #[test]
    fn a_changed_reveal_fails_its_hash() {
        let mut rng = ChaCha20Rng::from_seed([7; 32]);
        let share = SigningShare::deserialize(&scalar_bytes(&PallasScalarField::random(&mut rng))).unwrap();
        let (_, first) = frost::round1::commit(&share, &mut rng);
        let (_, second) = frost::round1::commit(&share, &mut rng);
        let session = [1; 32];
        let hashes = BTreeMap::from([(1, commitment_hash(&session, 1, &first).unwrap())]);
        assert!(check_reveal(&session, &hashes, &BTreeMap::from([(1, first)])).is_ok());
        assert!(check_reveal(&session, &hashes, &BTreeMap::from([(1, second)])).is_err());
        assert!(check_reveal(&session, &hashes, &BTreeMap::from([(2, first)])).is_err());
    }

    #[test]
    fn one_member_cannot_sign() {
        assert!(run(KEYS, NONCES, "object", &[2], "transfer").is_err());
    }

    #[test]
    fn the_tweak_is_public_and_per_object() {
        let a = run(KEYS, NONCES, "object a", &[1, 2], "m").unwrap();
        let b = run(KEYS, NONCES, "object b", &[1, 2], "m").unwrap();
        assert_eq!(a.group_key, b.group_key);
        assert_ne!(a.object_group_key, b.object_group_key);
        // A_G + [t]G from public values alone.
        let expected = point(&hex::decode(&a.group_key).unwrap().try_into().unwrap()).unwrap()
            + PallasGroup::generator() * tweak("object a");
        assert_eq!(a.object_group_key, hex::encode(bytes(&expected).unwrap()));
    }
}
