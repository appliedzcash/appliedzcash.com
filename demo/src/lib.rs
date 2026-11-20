//! Demos for appliedzcash.com. Each pattern and each basic adds one module. CI builds the
//! crate for `wasm32-unknown-unknown` and runs `wasm-bindgen --target web`.

mod bound_notes;
mod customer_addresses;
mod inspect;
mod orders;
mod pairing;
mod payment_link;
mod threshold;
mod viewing_keys;

pub use bound_notes::{genesis_rho, stamp, Stamp};
pub use customer_addresses::{customer_addresses, find_customer, CustomerAddresses};
pub use inspect::{inspect, Inspection};
pub use orders::{order, Order};
pub use pairing::{pair, Bundle};
pub use payment_link::{payment_link, PaymentLink};
pub use threshold::{threshold, Threshold};
pub use viewing_keys::{viewing_keys, ViewingKeys};

use orchard::keys::{FullViewingKey, SpendingKey};
use qrcode::{render::svg, EcLevel, QrCode};
use zcash_address::unified::{self, Encoding, Receiver, Revision, Uitem};
use zcash_protocol::consensus::NetworkType;

/// The QR code of `text` as SVG.
pub(crate) fn qr_svg(text: &str) -> Result<String, String> {
    Ok(QrCode::with_error_correction_level(text.as_bytes(), EcLevel::L)
        .map_err(|error| format!("The QR code failed: {error}."))?
        .render::<svg::Color>()
        .min_dimensions(256, 256)
        .build())
}

/// The ZIP 32 coin type of Testnet. All basics demos use Testnet keys.
pub(crate) const TESTNET_COIN_TYPE: u32 = 1;

/// The Orchard spending key of account 0 for a ZIP 32 seed, on Testnet.
pub(crate) fn testnet_spending_key(seed: &[u8]) -> Result<SpendingKey, String> {
    SpendingKey::from_zip32_seed(seed, TESTNET_COIN_TYPE, zip32::AccountId::ZERO)
        .map_err(|error| format!("The demo cannot make the keys: {error}."))
}

/// A Testnet unified address with one Orchard receiver.
pub(crate) fn testnet_address(address: &orchard::Address) -> String {
    let receiver = Receiver::Orchard(address.to_raw_address_bytes());
    unified::Address::try_from_items(Revision::R0, vec![Uitem::Data(receiver)])
        .expect("one Orchard receiver is a valid unified address")
        .encode(&NetworkType::Test)
}

/// The demo shop key of the basics pages. Its seed is public, so it is a demo key only.
pub(crate) fn demo_shop_key() -> FullViewingKey {
    let sk = testnet_spending_key(&[7; 32]).expect("a 32-byte seed is valid");
    FullViewingKey::from(&sk)
}
