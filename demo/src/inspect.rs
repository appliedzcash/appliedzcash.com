//! Concepts: inspect a Zcash address.
//!
//! A unified address holds one receiver for each pool that it accepts. The demo decodes an
//! address and lists its receivers, so a reader sees which pools a payment can reach.

use wasm_bindgen::prelude::*;
use zcash_address::{
    unified::{self, Container, Receiver},
    ConversionError, TryFromAddress, ZcashAddress,
};
use zcash_protocol::consensus::NetworkType;

/// The parts of one address. `names` and `values` are parallel lists, one row per receiver.
#[wasm_bindgen(getter_with_clone)]
pub struct Inspection {
    pub kind: String,
    pub network: String,
    pub memo: bool,
    pub names: Vec<String>,
    pub values: Vec<String>,
}

/// Decodes `address` and lists its receivers.
#[wasm_bindgen]
pub fn inspect(address: &str) -> Result<Inspection, JsError> {
    parse(address).map_err(|error| JsError::new(&error))
}

fn parse(address: &str) -> Result<Inspection, String> {
    let address = address.trim();
    if address.is_empty() {
        return Err("Type or paste an address.".into());
    }
    ZcashAddress::try_from_encoded(address)
        .map_err(|_| "This text is not a valid Zcash address.".to_string())?
        .convert::<Inspection>()
        .map_err(|error| match error {
            ConversionError::Unsupported(kind) => format!("The demo cannot show this address type: {kind}."),
            other => format!("The address failed to decode: {other}."),
        })
}

fn network(net: NetworkType) -> String {
    match net {
        NetworkType::Main => "Mainnet",
        NetworkType::Test => "Testnet",
        NetworkType::Regtest => "Regtest",
    }
    .into()
}

fn single(net: NetworkType, kind: &str, memo: bool, name: &str, data: &[u8]) -> Inspection {
    Inspection {
        kind: kind.into(),
        network: network(net),
        memo,
        names: vec![format!("{name} ({} bytes)", data.len())],
        values: vec![hex::encode(data)],
    }
}

impl TryFromAddress for Inspection {
    type Error = String;

    fn try_from_sprout(net: NetworkType, data: [u8; 64]) -> Result<Self, ConversionError<String>> {
        Ok(single(net, "Sprout address (old, closed pool)", true, "Sprout receiver", &data))
    }

    fn try_from_sapling(net: NetworkType, data: [u8; 43]) -> Result<Self, ConversionError<String>> {
        Ok(single(net, "Sapling address", true, "Sapling receiver", &data))
    }

    fn try_from_unified(net: NetworkType, data: unified::Address) -> Result<Self, ConversionError<String>> {
        let (names, values) = data
            .items()
            .iter()
            .map(|receiver| match receiver {
                Receiver::Orchard(bytes) => (format!("Orchard receiver ({} bytes)", bytes.len()), hex::encode(bytes)),
                Receiver::Sapling(bytes) => (format!("Sapling receiver ({} bytes)", bytes.len()), hex::encode(bytes)),
                Receiver::P2pkh(bytes) => (format!("Transparent P2PKH receiver ({} bytes)", bytes.len()), hex::encode(bytes)),
                Receiver::P2sh(bytes) => (format!("Transparent P2SH receiver ({} bytes)", bytes.len()), hex::encode(bytes)),
                Receiver::Unknown { typecode, data } => (format!("Unknown receiver, typecode {typecode}"), hex::encode(data)),
            })
            .unzip();
        Ok(Inspection {
            kind: "Unified address".into(),
            network: network(net),
            memo: data.can_receive_memo(),
            names,
            values,
        })
    }

    fn try_from_transparent_p2pkh(net: NetworkType, data: [u8; 20]) -> Result<Self, ConversionError<String>> {
        Ok(single(net, "Transparent address (P2PKH)", false, "Public key hash", &data))
    }

    fn try_from_transparent_p2sh(net: NetworkType, data: [u8; 20]) -> Result<Self, ConversionError<String>> {
        Ok(single(net, "Transparent address (P2SH)", false, "Script hash", &data))
    }

    fn try_from_tex(net: NetworkType, data: [u8; 20]) -> Result<Self, ConversionError<String>> {
        Ok(single(net, "TEX address (transparent, ZIP 320)", false, "Public key hash", &data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The donation address in the footer of the site.
    const DONATE: &str = "u1j8gmxccf6g59x2er94h7ym959ehr7dmavhsk5vu9lu3wmtltffcwdlc7pa32mj0dxtrcans43hr6sl8d3w4mhn6klxry4w3kz6lkzwphvthrhwak506ah6fzmxzlsaeu7ghky3xcq23nt6fhwrnwc8rtupggntn4hfhjprfxtvaz6nkx";

    #[test]
    fn unified_address_lists_its_receivers() {
        let parts = parse(DONATE).unwrap();
        assert_eq!(parts.kind, "Unified address");
        assert_eq!(parts.network, "Mainnet");
        assert!(parts.memo);
        assert!(parts.names.iter().any(|name| name.starts_with("Orchard receiver")));
        assert_eq!(parts.names.len(), parts.values.len());
    }

    #[test]
    fn demo_shop_address_is_testnet_orchard() {
        let address = crate::testnet_address(&crate::demo_shop_key().address_at(0u32, orchard::keys::Scope::External));
        let parts = parse(&address).unwrap();
        assert_eq!(parts.network, "Testnet");
        assert_eq!(parts.names, vec!["Orchard receiver (43 bytes)"]);
    }

    #[test]
    fn transparent_address_takes_no_memo() {
        let parts = parse("t1Hsc1LR8yKnbbe3twRp88p6vFfC5t7DLbs").unwrap();
        assert_eq!(parts.kind, "Transparent address (P2PKH)");
        assert!(!parts.memo);
    }

    #[test]
    fn bad_text_fails() {
        assert!(parse("").is_err());
        assert!(parse("zcash").is_err());
        assert!(parse(&DONATE[..DONATE.len() - 1]).is_err());
    }
}
