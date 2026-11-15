//! Wallet-native orders: a toy poll.
//!
//! An order is a ZIP 321 payment of the order amount to the operator's Inbox. Its memo is
//! one line of printable ASCII with the commands and a MAC under the participant's session
//! key. The demo Inbox is a Testnet address, so a Mainnet wallet refuses the request.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use blake2b_simd::Params;
use orchard::keys::{FullViewingKey, Scope, SpendingKey};
use wasm_bindgen::prelude::*;

/// The scheme token of the toy poll.
const SCHEME: &str = "toypoll1";

/// The MAC personalization of the scheme.
const MAC_PERSONAL: &[u8; 16] = b"ToyPoll_ordmac_1";

/// The round identifier personalization of the scheme.
const ROUND_PERSONAL: &[u8; 16] = b"ToyPoll_rules_v1";

/// The rules document of the demo round. Its hash is the round identifier.
const RULES: &[u8] = b"toypoll1 demo round: questions q1 to q9, YES or NO, 1000 zatoshi per order";

/// The order amount of the demo round, in ZEC (1,000 zatoshi).
const ORDER_AMOUNT: &str = "0.00001";

/// The most commands that one memo holds.
const MAX_COMMANDS: usize = 9;

/// The `r` tag: the first 4 bytes of the round identifier, as lowercase hex.
fn round_tag() -> String {
    let round_id = Params::new().hash_length(32).personal(ROUND_PERSONAL).hash(RULES);
    hex::encode(&round_id.as_bytes()[..4])
}

/// The demo Inbox: a Testnet unified address with one Orchard receiver.
fn inbox() -> String {
    let sk = SpendingKey::from_bytes([9; 32]).expect("a valid spending key");
    crate::testnet_address(&FullViewingKey::from(&sk).address_at(0u32, Scope::External))
}

/// A toy poll command: `q` and a question number from 1 to 9, a space, then YES or NO.
fn valid_command(command: &str) -> bool {
    let Some((question, answer)) = command.split_once(' ') else {
        return false;
    };
    let digits = question.strip_prefix('q').unwrap_or("");
    digits.len() == 1 && matches!(digits.as_bytes()[0], b'1'..=b'9') && matches!(answer, "YES" | "NO")
}

/// MAC_k(P_mac, prefix): keyed BLAKE2b with an 8-byte output, as lowercase hex.
fn mac(key: &[u8; 32], prefix: &str) -> String {
    Params::new()
        .hash_length(8)
        .key(key)
        .personal(MAC_PERSONAL)
        .hash(prefix.as_bytes())
        .to_hex()
        .to_string()
}

/// The values of one order.
#[wasm_bindgen(getter_with_clone)]
pub struct Order {
    pub inbox: String,
    pub memo: String,
    pub uri: String,
    pub qr_svg: String,
}

/// Makes the order memo for `commands` (one command per line), its ZIP 321 URI, and the
/// QR code of the URI as SVG.
#[wasm_bindgen]
pub fn order(
    session_key: &str,
    participant: u32,
    nonce: &str,
    expiry: u32,
    commands: &str,
) -> Result<Order, JsError> {
    build(session_key, participant, nonce, expiry, commands).map_err(|error| JsError::new(&error))
}

fn build(
    session_key: &str,
    participant: u32,
    nonce: &str,
    expiry: u32,
    commands: &str,
) -> Result<Order, String> {
    let mut key = [0; 32];
    hex::decode_to_slice(session_key.trim(), &mut key)
        .map_err(|_| "The session key must be 64 hexadecimal characters.".to_string())?;
    let nonce_ok = nonce.len() == 8 && nonce.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
    if !nonce_ok {
        return Err("The nonce must be 8 lowercase hexadecimal characters.".into());
    }
    let commands: Vec<&str> = commands.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    if commands.is_empty() {
        return Err("Add one command or more.".into());
    }
    if commands.len() > MAX_COMMANDS {
        return Err("A memo holds 9 commands or fewer.".into());
    }
    if let Some(bad) = commands.iter().find(|command| !valid_command(command)) {
        return Err(format!("Use q1 to q9, then YES or NO. This line has an error: {bad}"));
    }

    let prefix = format!(
        "{SCHEME} r={} t={participant} n={nonce} x={expiry} | {}",
        round_tag(),
        commands.join(" | ")
    );
    let memo = format!("{prefix} | mac={}", mac(&key, &prefix));
    let inbox = inbox();
    let uri = format!(
        "zcash:{inbox}?amount={ORDER_AMOUNT}&memo={}",
        URL_SAFE_NO_PAD.encode(memo.as_bytes())
    );
    let qr_svg = crate::qr_svg(&uri)?;

    Ok(Order { inbox, memo, uri, qr_svg })
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "0101010101010101010101010101010101010101010101010101010101010101";

    #[test]
    fn mac_matches_reference_vector() {
        // Python: hashlib.blake2b(b"abc", digest_size=8, key=bytes([1] * 32),
        //                         person=b"ToyPoll_ordmac_1").hexdigest()
        assert_eq!(mac(&[1; 32], "abc"), "4f3ee0b1321dbaec");
    }

    #[test]
    fn memo_has_the_grammar_of_the_zip() {
        let order = build(KEY, 7, "9c1e4a7b", 4577200, "q1 YES\nq3 NO").unwrap();
        let prefix = format!("toypoll1 r={} t=7 n=9c1e4a7b x=4577200 | q1 YES | q3 NO", round_tag());
        assert_eq!(order.memo, format!("{prefix} | mac={}", mac(&[1; 32], &prefix)));
        assert!(order.memo.len() <= 512);
        assert!(order.memo.bytes().all(|b| (0x20..=0x7e).contains(&b)));
    }

    #[test]
    fn uri_carries_the_memo_and_a_testnet_inbox() {
        let order = build(KEY, 7, "9c1e4a7b", 4577200, "q1 YES").unwrap();
        assert!(order.inbox.starts_with("utest1"));
        let memo = order.uri.split("&memo=").nth(1).unwrap();
        assert_eq!(URL_SAFE_NO_PAD.decode(memo).unwrap(), order.memo.as_bytes());
        assert!(order.qr_svg.starts_with("<?xml"));
    }

    #[test]
    fn bad_input_fails() {
        assert!(build("00", 7, "9c1e4a7b", 1, "q1 YES").is_err());
        assert!(build(KEY, 7, "9C1E4A7B", 1, "q1 YES").is_err());
        assert!(build(KEY, 7, "9c1e4a7b", 1, "").is_err());
        assert!(build(KEY, 7, "9c1e4a7b", 1, "q0 YES").is_err());
        assert!(build(KEY, 7, "9c1e4a7b", 1, "q1 yes").is_err());
        assert!(build(KEY, 7, "9c1e4a7b", 1, "q1 YES | mac=00").is_err());
    }
}
