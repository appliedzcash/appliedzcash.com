//! Concepts: inspect a Zcash address, and build a unified address from its receivers.
//!
//! A unified address holds one receiver for each pool that it accepts. The demo decodes an
//! address and lists its receivers, so a reader sees which pools a payment can reach. For a
//! unified address, the demo also shows the three steps of ZIP 316: the bytes of the items and
//! the padding, the F4Jumble of these bytes, and the Bech32m string.

use wasm_bindgen::prelude::*;
use zcash_address::{
    unified::{self, Container, Encoding, Item, ParseError, Receiver, Uitem},
    ConversionError, TryFromAddress, ZcashAddress,
};
use zcash_protocol::consensus::NetworkType;

/// The parts of one address. `names` and `values` are parallel lists, one row per receiver.
#[wasm_bindgen(getter_with_clone)]
pub struct Inspection {
    pub unified: bool,
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
            other => format!("The demo cannot decode the address: {other}."),
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
        unified: false,
        kind: kind.into(),
        network: network(net),
        memo,
        names: vec![format!("{name} ({} bytes)", data.len())],
        values: vec![hex::encode(data)],
    }
}

/// The color key, the name and the data of one receiver.
fn describe(receiver: &Receiver) -> (&'static str, String, &[u8]) {
    match receiver {
        Receiver::Orchard(data) => ("orchard", "Orchard receiver".into(), data),
        Receiver::Sapling(data) => ("sapling", "Sapling receiver".into(), data),
        Receiver::P2pkh(data) => ("transparent", "Transparent P2PKH receiver".into(), data),
        Receiver::P2sh(data) => ("transparent", "Transparent P2SH receiver".into(), data),
        Receiver::Unknown { typecode, data } => ("other", format!("Unknown receiver, typecode {typecode}"), data),
    }
}

/// The CompactSize encoding of `n`.
fn compact_size(n: usize) -> Vec<u8> {
    match n {
        0..=0xfc => vec![n as u8],
        0xfd..=0xffff => [&[0xfd][..], &(n as u16).to_le_bytes()].concat(),
        _ => [&[0xfe][..], &(n as u32).to_le_bytes()].concat(),
    }
}

/// The length of the padding in a unified address (ZIP 316).
const PADDING_LEN: usize = 16;

const NO_SHIELDED: &str = "A unified address must have a shielded receiver. Select a shielded receiver.";

/// The construction of one unified address (ZIP 316). `kinds`, `names`, `heads` and `data` are
/// parallel lists with one row for each item, in encoding order. The last row is the padding.
/// A head is the typecode and the length of an item. All byte strings are hex.
#[wasm_bindgen(getter_with_clone)]
pub struct Construction {
    pub address: String,
    pub prefix: String,
    pub kinds: Vec<String>,
    pub names: Vec<String>,
    pub heads: Vec<String>,
    pub data: Vec<String>,
    pub jumbled: String,
}

/// Builds a unified address from the items of `address`. `skip` lists the items to leave out,
/// by their index in the encoding order of `address`.
#[wasm_bindgen]
pub fn construct(address: &str, skip: &[u32]) -> Result<Construction, JsError> {
    build(address, skip).map_err(|error| JsError::new(&error))
}

fn build(address: &str, skip: &[u32]) -> Result<Construction, String> {
    let (net, revision, source) =
        unified::Address::decode(address.trim()).map_err(|_| "This text is not a valid unified address.".to_string())?;
    let items = source
        .items_as_parsed()
        .iter()
        .enumerate()
        .filter(|(i, _)| !skip.contains(&(*i as u32)))
        .map(|(_, item)| item.clone())
        .collect::<Vec<_>>();
    let no_receiver = items.iter().all(|item| matches!(item, Uitem::Metadata(_)));
    let built = unified::Address::try_from_items(revision, items).map_err(|error| match error {
        ParseError::OnlyTransparent => NO_SHIELDED.to_string(),
        _ if no_receiver => NO_SHIELDED.to_string(),
        other => format!("The demo cannot make this address: {other}."),
    })?;
    let encoded = built.encode(&net);
    let prefix = encoded.rsplit_once('1').map_or("", |(prefix, _)| prefix).to_string();

    let (mut kinds, mut names, mut heads, mut data) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut raw = Vec::new();
    for item in built.items_as_parsed() {
        let (kind, name, head, bytes) = match item {
            Uitem::Data(receiver) => {
                let (kind, name, bytes) = describe(receiver);
                let typed = receiver.typed_encoding();
                (kind, name, typed[..typed.len() - bytes.len()].to_vec(), bytes.to_vec())
            }
            Uitem::Metadata(item) => {
                let typecode = u32::from(item.typecode());
                let bytes = item.data();
                let head = [compact_size(typecode as usize), compact_size(bytes.len())].concat();
                ("other", format!("Metadata item, typecode {typecode}"), head, bytes)
            }
        };
        raw.extend_from_slice(&head);
        raw.extend_from_slice(&bytes);
        kinds.push(kind.to_string());
        names.push(name);
        heads.push(hex::encode(head));
        data.push(hex::encode(bytes));
    }
    let mut padding = [0u8; PADDING_LEN];
    padding[..prefix.len()].copy_from_slice(prefix.as_bytes());
    raw.extend_from_slice(&padding);
    kinds.push("padding".into());
    names.push("Padding".into());
    heads.push(String::new());
    data.push(hex::encode(padding));

    let jumbled = f4jumble::f4jumble(&raw).map_err(|_| "The demo cannot apply F4Jumble to these bytes.".to_string())?;
    Ok(Construction { address: encoded, prefix, kinds, names, heads, data, jumbled: hex::encode(jumbled) })
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
            .map(|receiver| {
                let (_, name, bytes) = describe(receiver);
                (format!("{name} ({} bytes)", bytes.len()), hex::encode(bytes))
            })
            .unzip();
        Ok(Inspection {
            unified: true,
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

    /// An example mainnet Unified Address.
    const EXAMPLE: &str = "u1j8gmxccf6g59x2er94h7ym959ehr7dmavhsk5vu9lu3wmtltffcwdlc7pa32mj0dxtrcans43hr6sl8d3w4mhn6klxry4w3kz6lkzwphvthrhwak506ah6fzmxzlsaeu7ghky3xcq23nt6fhwrnwc8rtupggntn4hfhjprfxtvaz6nkx";

    /// A test address with a transparent, a Sapling and an Orchard receiver, from
    /// `zcash_test_vectors/unified_address.py` in zcash/zcash-test-vectors.
    const THREE: &str = "u1pg2aaph7jp8rpf6yhsza25722sg5fcn3vaca6ze27hqjw7jvvhhuxkpcg0ge9xh6drsgdkda8qjq5chpehkcpxf87rnjryjqwymdheptpvnljqqrjqzjwkc2ma6hcq666kgwfytxwac8eyex6ndgr6ezte66706e3vaqrd25dzvzkc69kw0jgywtd0cmq52q5lkw6uh7hyvzjse8ksx";

    #[test]
    fn unified_address_lists_its_receivers() {
        let parts = parse(EXAMPLE).unwrap();
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

    /// The payload of a Bech32m string, to compare with the F4Jumble output of the demo.
    fn payload(address: &str) -> String {
        let parsed = bech32::primitives::decode::CheckedHrpstring::new::<unified::Bech32mZip316>(address).unwrap();
        hex::encode(parsed.byte_iter().collect::<Vec<u8>>())
    }

    #[test]
    fn construction_gives_the_same_address() {
        let built = build(EXAMPLE, &[]).unwrap();
        assert_eq!(built.address, EXAMPLE);
        assert_eq!(built.jumbled, payload(EXAMPLE));
        assert_eq!(built.kinds.last().unwrap(), "padding");
        assert_eq!(built.data.last().unwrap(), &format!("{:0<32}", hex::encode(&built.prefix)));
    }

    #[test]
    fn construction_leaves_out_a_receiver() {
        let full = build(EXAMPLE, &[]).unwrap();
        let orchard = full.kinds.iter().position(|kind| kind == "orchard").unwrap() as u32;
        let others: Vec<u32> = (0..full.kinds.len() as u32 - 1).filter(|i| *i != orchard).collect();
        let built = build(EXAMPLE, &others).unwrap();
        assert_eq!(built.names, vec!["Orchard receiver", "Padding"]);
        assert_eq!(built.heads[0], "032b");
        assert_ne!(built.address, EXAMPLE);
        assert_eq!(built.jumbled, payload(&built.address));
        assert!(parse(&built.address).unwrap().unified);
    }

    #[test]
    fn construction_needs_a_shielded_receiver() {
        let full = build(THREE, &[]).unwrap();
        assert_eq!(full.address, THREE);
        assert_eq!(full.jumbled, payload(THREE));
        assert_eq!(full.kinds, vec!["transparent", "sapling", "orchard", "padding"]);
        assert_eq!(full.heads, vec!["0014", "022b", "032b", ""]);
        assert!(build(THREE, &[0]).is_ok());
        assert!(build(THREE, &[1, 2]).err().unwrap().contains("shielded receiver"));
        assert_eq!(build(THREE, &[0, 1, 2]).err().unwrap(), NO_SHIELDED);
        assert!(build("t1Hsc1LR8yKnbbe3twRp88p6vFfC5t7DLbs", &[]).is_err());
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
        assert!(parse(&EXAMPLE[..EXAMPLE.len() - 1]).is_err());
    }
}
