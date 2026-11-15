//! Watch-only keys: one seed, three keys.
//!
//! ZIP 32 derives the spending key from the seed. The full viewing key comes from the
//! spending key, and the incoming viewing key comes from the full viewing key. Each step
//! drops a power. The incoming viewing key still makes the same addresses as the wallet.

use orchard::keys::{FullViewingKey, Scope};
use wasm_bindgen::prelude::*;
use zcash_address::unified::{Encoding, Fvk, Ivk, Revision, Ufvk, Uitem, Uivk};
use zcash_protocol::consensus::NetworkType;

/// The keys of account 0 for one seed, on Testnet.
#[wasm_bindgen(getter_with_clone)]
pub struct ViewingKeys {
    pub spending_key: String,
    pub ufvk: String,
    pub uivk: String,
    pub wallet_address: String,
    pub watch_address: String,
}

/// Derives the spending key, the UFVK and the UIVK of account 0 from a hex seed. It also
/// makes address 0 two times: from the spending key, and from the UIVK only.
#[wasm_bindgen]
pub fn viewing_keys(seed: &str) -> Result<ViewingKeys, JsError> {
    derive(seed).map_err(|error| JsError::new(&error))
}

fn derive(seed: &str) -> Result<ViewingKeys, String> {
    let seed = hex::decode(seed.trim()).map_err(|_| "The seed must be hexadecimal characters.".to_string())?;
    if !(32..=252).contains(&seed.len()) {
        return Err("The seed must be 32 to 252 bytes (64 to 504 hexadecimal characters).".into());
    }
    let sk = crate::testnet_spending_key(&seed)?;
    let fvk = FullViewingKey::from(&sk);
    let ivk = fvk.to_ivk(Scope::External);

    let ufvk = Ufvk::try_from_items(Revision::R0, vec![Uitem::Data(Fvk::Orchard(fvk.to_bytes()))])
        .map_err(|error| format!("The UFVK failed: {error}."))?
        .encode(&NetworkType::Test);
    let uivk = Uivk::try_from_items(Revision::R0, vec![Uitem::Data(Ivk::Orchard(ivk.to_bytes()))])
        .map_err(|error| format!("The UIVK failed: {error}."))?
        .encode(&NetworkType::Test);

    Ok(ViewingKeys {
        spending_key: hex::encode(sk.to_bytes()),
        ufvk,
        uivk,
        wallet_address: crate::testnet_address(&fvk.address_at(0u32, Scope::External)),
        watch_address: crate::testnet_address(&ivk.address_at(0u32)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zcash_address::unified::Container;

    const SEED: &str = "0707070707070707070707070707070707070707070707070707070707070707";

    #[test]
    fn keys_encode_for_testnet() {
        let keys = derive(SEED).unwrap();
        assert!(keys.ufvk.starts_with("uviewtest1"));
        assert!(keys.uivk.starts_with("uivktest1"));
        assert_eq!(keys.spending_key.len(), 64);
    }

    #[test]
    fn watch_key_makes_the_wallet_address() {
        let keys = derive(SEED).unwrap();
        assert_eq!(keys.wallet_address, keys.watch_address);
        // The demo shop of the other basics uses the same seed.
        assert_eq!(
            keys.wallet_address,
            crate::testnet_address(&crate::demo_shop_key().address_at(0u32, Scope::External))
        );
    }

    #[test]
    fn ufvk_decodes_to_the_same_key() {
        let keys = derive(SEED).unwrap();
        let (net, _, ufvk) = Ufvk::decode(&keys.ufvk).unwrap();
        assert_eq!(net, NetworkType::Test);
        let sk = crate::testnet_spending_key(&[7; 32]).unwrap();
        assert!(matches!(ufvk.items()[..], [Fvk::Orchard(bytes)] if bytes == FullViewingKey::from(&sk).to_bytes()));
    }

    #[test]
    fn bad_seed_fails() {
        assert!(derive("00").is_err());
        assert!(derive("zz").is_err());
        assert!(derive(&"00".repeat(253)).is_err());
    }
}
