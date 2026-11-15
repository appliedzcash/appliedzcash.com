//! Address per customer: diversified Orchard addresses from one key.
//!
//! Index j of the key gives address j. Outsiders cannot link two addresses of the key. The
//! incoming viewing key finds the index of an address again, so the shop knows which
//! customer paid.

use orchard::keys::Scope;
use wasm_bindgen::prelude::*;
use zcash_address::unified::{self, Container, Encoding, Receiver};

/// The most addresses that one call returns.
const MAX_COUNT: u32 = 10;

/// A list of addresses of the demo shop key. `indexes` and `addresses` are parallel lists.
#[wasm_bindgen(getter_with_clone)]
pub struct CustomerAddresses {
    pub indexes: Vec<u32>,
    pub addresses: Vec<String>,
}

/// The addresses of the demo shop key at indexes `start` to `start + count - 1`.
#[wasm_bindgen]
pub fn customer_addresses(start: u32, count: u32) -> Result<CustomerAddresses, JsError> {
    list(start, count).map_err(|error| JsError::new(&error))
}

fn list(start: u32, count: u32) -> Result<CustomerAddresses, String> {
    if count == 0 || count > MAX_COUNT {
        return Err("Show 1 to 10 addresses.".into());
    }
    let end = start
        .checked_add(count - 1)
        .ok_or_else(|| "The first index is too large.".to_string())?;
    let fvk = crate::demo_shop_key();
    let indexes: Vec<u32> = (start..=end).collect();
    let addresses = indexes
        .iter()
        .map(|&j| crate::testnet_address(&fvk.address_at(j, Scope::External)))
        .collect();
    Ok(CustomerAddresses { indexes, addresses })
}

/// Finds the index of `address` under the incoming viewing key of the demo shop.
/// Returns `None` for an address of a different key.
#[wasm_bindgen]
pub fn find_customer(address: &str) -> Result<Option<u32>, JsError> {
    find(address).map_err(|error| JsError::new(&error))
}

fn find(address: &str) -> Result<Option<u32>, String> {
    let (_, _, ua) = unified::Address::decode(address.trim())
        .map_err(|_| "Paste a unified address from the list above.".to_string())?;
    let bytes = ua
        .items()
        .into_iter()
        .find_map(|receiver| match receiver {
            Receiver::Orchard(bytes) => Some(bytes),
            _ => None,
        })
        .ok_or_else(|| "The address has no Orchard receiver.".to_string())?;
    let receiver = Option::from(orchard::Address::from_raw_address_bytes(&bytes))
        .ok_or_else(|| "The Orchard receiver is not valid.".to_string())?;
    let ivk = crate::demo_shop_key().to_ivk(Scope::External);
    Ok(ivk.diversifier_index(&receiver).and_then(|j| u32::try_from(j).ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_are_distinct() {
        let list = list(0, 10).unwrap();
        assert_eq!(list.indexes, (0..10).collect::<Vec<_>>());
        let mut unique = list.addresses.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 10);
        assert!(list.addresses.iter().all(|a| a.starts_with("utest1")));
    }

    #[test]
    fn index_comes_back() {
        let list = list(40, 3).unwrap();
        for (j, address) in list.indexes.iter().zip(&list.addresses) {
            assert_eq!(find(address).unwrap(), Some(*j));
        }
    }

    #[test]
    fn foreign_address_has_no_index() {
        let sk = orchard::keys::SpendingKey::from_bytes([9; 32]).unwrap();
        let other = crate::testnet_address(&orchard::keys::FullViewingKey::from(&sk).address_at(0u32, Scope::External));
        assert_eq!(find(&other).unwrap(), None);
    }

    #[test]
    fn bad_input_fails() {
        assert!(list(0, 0).is_err());
        assert!(list(0, 11).is_err());
        assert!(list(u32::MAX, 2).is_err());
        assert!(find("t1Hsc1LR8yKnbbe3twRp88p6vFfC5t7DLbs").is_err());
    }
}
