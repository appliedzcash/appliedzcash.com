//! Payment links: a ZIP 321 payment request URI and its QR code.
//!
//! The `zip321` crate writes the URI. It encodes the memo as base64url and escapes the
//! label and the message. It refuses a memo to a transparent address.

use wasm_bindgen::prelude::*;
use zcash_address::ZcashAddress;
use zcash_protocol::{memo::MemoBytes, value::Zatoshis};
use zip321::{Payment, TransactionRequest};

/// A payment request: the URI and the QR code of the URI as SVG.
#[wasm_bindgen(getter_with_clone)]
pub struct PaymentLink {
    pub uri: String,
    pub qr_svg: String,
    pub memo_length: u32,
}

/// Makes a ZIP 321 URI for one payment. Empty fields stay out of the URI.
#[wasm_bindgen]
pub fn payment_link(
    address: &str,
    amount: &str,
    memo: &str,
    label: &str,
    message: &str,
) -> Result<PaymentLink, JsError> {
    build(address, amount, memo, label, message).map_err(|error| JsError::new(&error))
}

/// Parses a ZEC amount with 8 decimal places or fewer into zatoshis.
fn parse_zec(amount: &str) -> Result<Zatoshis, String> {
    const BAD: &str = "Type the amount in ZEC, for example 0.25. Use 8 decimal places or fewer.";
    let (whole, fraction) = amount.split_once('.').unwrap_or((amount, ""));
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if whole.is_empty() || fraction.len() > 8 || !digits(whole) || !digits(fraction) || whole.len() > 8 {
        return Err(BAD.into());
    }
    let zatoshis = whole.parse::<u64>().map_err(|_| BAD)? * 100_000_000
        + format!("{fraction:0<8}").parse::<u64>().map_err(|_| BAD)?;
    Zatoshis::from_u64(zatoshis).map_err(|_| "The amount is more than the supply of ZEC.".into())
}

fn some(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn build(address: &str, amount: &str, memo: &str, label: &str, message: &str) -> Result<PaymentLink, String> {
    let recipient = ZcashAddress::try_from_encoded(address.trim())
        .map_err(|_| "The address is not a valid Zcash address.".to_string())?;
    let amount = some(amount).map(|amount| parse_zec(&amount)).transpose()?;
    let memo_length = memo.len() as u32;
    let memo = (!memo.is_empty())
        .then(|| MemoBytes::from_bytes(memo.as_bytes()))
        .transpose()
        .map_err(|_| format!("The memo is {memo_length} bytes. A memo holds 512 bytes or fewer."))?;

    let payment = Payment::new(recipient, amount, memo, some(label), some(message), vec![])
        .map_err(|_| "A transparent address cannot receive a memo. Clear the memo, or use a shielded address.".to_string())?;
    let uri = TransactionRequest::new(vec![payment])
        .map_err(|error| format!("The request failed: {error}."))?
        .to_uri();
    let qr_svg = crate::qr_svg(&uri)?;

    Ok(PaymentLink { uri, qr_svg, memo_length })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHOP: &str = "u1j8gmxccf6g59x2er94h7ym959ehr7dmavhsk5vu9lu3wmtltffcwdlc7pa32mj0dxtrcans43hr6sl8d3w4mhn6klxry4w3kz6lkzwphvthrhwak506ah6fzmxzlsaeu7ghky3xcq23nt6fhwrnwc8rtupggntn4hfhjprfxtvaz6nkx";

    #[test]
    fn amounts_parse_to_zatoshis() {
        assert_eq!(parse_zec("1").unwrap().into_u64(), 100_000_000);
        assert_eq!(parse_zec("0.25").unwrap().into_u64(), 25_000_000);
        assert_eq!(parse_zec("0.00000001").unwrap().into_u64(), 1);
        assert!(parse_zec("0.000000001").is_err());
        assert!(parse_zec(".5").is_err());
        assert!(parse_zec("1,5").is_err());
        assert!(parse_zec("21000001").is_err());
    }

    #[test]
    fn uri_round_trips() {
        let link = build(SHOP, "0.25", "Order 1042", "Coffee shop", "Two coffees").unwrap();
        let request = TransactionRequest::from_uri(&link.uri).unwrap();
        let payment = &request.payments()[&0];
        assert_eq!(payment.amount().unwrap().into_u64(), 25_000_000);
        assert_eq!(payment.label().unwrap(), "Coffee shop");
        assert_eq!(payment.message().unwrap(), "Two coffees");
        assert_eq!(&payment.memo().unwrap().as_slice()[..10], b"Order 1042");
        assert!(link.qr_svg.starts_with("<?xml"));
    }

    #[test]
    fn empty_fields_stay_out() {
        let link = build(SHOP, "", "", "", "").unwrap();
        assert_eq!(link.uri, format!("zcash:{SHOP}"));
    }

    #[test]
    fn bad_input_fails() {
        assert!(build("zcash", "1", "", "", "").is_err());
        assert!(build(SHOP, "0.1.2", "", "", "").is_err());
        assert!(build(SHOP, "1", &"x".repeat(513), "", "").is_err());
        assert!(build("t1Hsc1LR8yKnbbe3twRp88p6vFfC5t7DLbs", "1", "hello", "", "").is_err());
    }
}
