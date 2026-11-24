//! Memo stones: a toy record of research claims.
//!
//! A memo has a topic, a salt and a body. The body is the hash of a text, or the text. The
//! leaf of a memo is BLAKE2b-256 of the memo bytes. A service puts the Merkle root of the
//! leaves of one record, the stone, in a transparent `OP_RETURN`.
//! A receipt is the memo and the path from its leaf to the root.

use blake2b_simd::Params;
use wasm_bindgen::prelude::*;

/// The personalization of a leaf hash.
const LEAF_PERSONAL: &[u8; 16] = b"MemoStones__Leaf";

/// The personalization of a node hash.
const NODE_PERSONAL: &[u8; 16] = b"MemoStones__Node";

/// The personalization of a text hash.
const TEXT_PERSONAL: &[u8; 16] = b"MemoStones__Text";

/// The first words of each memo. A wallet shows them to the person before the payment.
const PREFIX: &str = "DO NOT MODIFY:{ms1/";

/// The tag and the version at the start of each record.
const RECORD_HEAD: &[u8; 5] = b"MST1\x01";

/// The length of a record: tag, version, record number, leaf count and root.
const RECORD_LENGTH: usize = 43;

const MEMO_LIMIT: usize = 512;

fn blake2b(personal: &[u8; 16], parts: &[&[u8]]) -> [u8; 32] {
    let mut state = Params::new().hash_length(32).personal(personal).to_state();
    for part in parts {
        state.update(part);
    }
    state.finalize().as_bytes().try_into().expect("hash_length is 32")
}

/// leaf = BLAKE2b-256("MemoStones__Leaf", memo).
fn leaf_hash(memo: &[u8]) -> [u8; 32] {
    blake2b(LEAF_PERSONAL, &[memo])
}

/// node = BLAKE2b-256("MemoStones__Node", min(a, b) || max(a, b)). The comparison is of the
/// raw bytes, in lexicographic sequence.
fn node_hash(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let (low, high) = if a <= b { (a, b) } else { (b, a) };
    blake2b(NODE_PERSONAL, &[low, high])
}

fn is_lower_hex(text: &str) -> bool {
    text.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn check_topic(topic: &str) -> Result<(), String> {
    let good = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
    if topic.is_empty() || topic.len() > 32 || !topic.bytes().all(good) {
        return Err("The topic must have 1 to 32 characters: a to z, 0 to 9 and the hyphen.".into());
    }
    Ok(())
}

fn check_salt(salt: &str) -> Result<(), String> {
    if salt.len() != 64 || !is_lower_hex(salt) {
        return Err("The salt must be 64 hexadecimal characters in lowercase.".into());
    }
    Ok(())
}

/// A wallet removes the zero bytes at the end of a memo. Thus, a text with U+0000 is invalid.
fn check_text(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Err("Type a text.".into());
    }
    if text.contains('\0') {
        return Err("The text must not contain the character U+0000.".into());
    }
    Ok(())
}

/// text hash = BLAKE2b-256("MemoStones__Text", text salt || text), as hexadecimal characters.
fn text_hash(text_salt: &str, text: &str) -> Result<String, String> {
    check_salt(text_salt)?;
    check_text(text)?;
    let salt = decode32(text_salt)?;
    Ok(hex::encode(blake2b(TEXT_PERSONAL, &[&salt, text.as_bytes()])))
}

/// Writes the memo of a topic, a salt and a body. The body is a text hash or a text.
fn write_memo(topic: &str, salt: &str, body: &str) -> Result<String, String> {
    check_topic(topic)?;
    check_salt(salt)?;
    check_text(body)?;
    let memo = format!("{PREFIX}{topic},{salt}}}\n{body}");
    if memo.len() > MEMO_LIMIT {
        return Err(format!("The memo is {} bytes. The limit is 512 bytes.", memo.len()));
    }
    Ok(memo)
}

/// Reads the topic, the salt and the body of a memo. Only the format of `write_memo` is valid.
fn read_memo(memo: &str) -> Result<(&str, &str, &str), String> {
    const BAD: &str = "The memo does not have the format of a memo stone.";
    if memo.len() > MEMO_LIMIT {
        return Err(format!("The memo is {} bytes. The limit is 512 bytes.", memo.len()));
    }
    let rest = memo.strip_prefix(PREFIX).ok_or(BAD)?;
    let (head, body) = rest.split_once("}\n").ok_or(BAD)?;
    let (topic, salt) = head.split_once(',').ok_or(BAD)?;
    check_topic(topic)?;
    check_salt(salt)?;
    if body.is_empty() || body.contains('\0') {
        return Err(BAD.into());
    }
    Ok((topic, salt, body))
}

fn decode32(hex_str: &str) -> Result<[u8; 32], String> {
    let mut bytes = [0; 32];
    if hex_str.len() != 64 || !is_lower_hex(hex_str) || hex::decode_to_slice(hex_str, &mut bytes).is_err() {
        return Err("Each hash must be 64 hexadecimal characters in lowercase.".into());
    }
    Ok(bytes)
}

/// Decodes the leaves of a period and puts them in sequence by value: the raw bytes, in
/// lexicographic sequence. Two payments with the same memo give two equal leaves. A record
/// has each leaf one time only, so this function drops the copies.
fn sorted_leaves(leaves: &[String]) -> Result<Vec<[u8; 32]>, String> {
    let mut out = leaves.iter().map(|leaf| decode32(leaf)).collect::<Result<Vec<_>, _>>()?;
    out.sort_unstable();
    out.dedup();
    if out.len() > usize::from(u16::MAX) {
        return Err("A record has 65,535 leaves at most.".into());
    }
    Ok(out)
}

/// The levels of the tree, from the sorted leaves to the root. Each level joins its nodes
/// in pairs from the left. An odd last node goes up unchanged.
fn levels(leaves: Vec<[u8; 32]>) -> Vec<Vec<[u8; 32]>> {
    let mut out = vec![leaves];
    while out.last().expect("one level or more").len() > 1 {
        let level = out.last().expect("one level or more");
        let mut next: Vec<[u8; 32]> = level.chunks_exact(2).map(|pair| node_hash(&pair[0], &pair[1])).collect();
        if level.len() % 2 == 1 {
            next.push(*level.last().expect("an odd level has a last node"));
        }
        out.push(next);
    }
    out
}

/// The path of the leaf at `index`: the other input of each node from the leaf to the root.
fn path_of(levels: &[Vec<[u8; 32]>], mut index: usize) -> Vec<[u8; 32]> {
    let mut path = Vec::new();
    for level in &levels[..levels.len() - 1] {
        if let Some(other) = level.get(index ^ 1) {
            path.push(*other);
        }
        index /= 2;
    }
    path
}

fn root_from(leaf: [u8; 32], path: &[[u8; 32]]) -> [u8; 32] {
    path.iter().fold(leaf, |node, other| node_hash(&node, other))
}

fn record_bytes(number: u32, count: u16, root: &[u8; 32]) -> Vec<u8> {
    let mut record = RECORD_HEAD.to_vec();
    record.extend_from_slice(&number.to_be_bytes());
    record.extend_from_slice(&count.to_be_bytes());
    record.extend_from_slice(root);
    record
}

/// Reads the record number, the leaf count and the root of a record.
fn read_record(record: &str) -> Result<(u32, u16, [u8; 32]), String> {
    const BAD: &str = "The record must be 43 bytes with the tag MST1 and version 1.";
    let bytes = hex::decode(record).map_err(|_| BAD)?;
    if bytes.len() != RECORD_LENGTH || &bytes[..5] != RECORD_HEAD {
        return Err(BAD.into());
    }
    let number = u32::from_be_bytes(bytes[5..9].try_into().expect("4 bytes"));
    let count = u16::from_be_bytes(bytes[9..11].try_into().expect("2 bytes"));
    Ok((number, count, bytes[11..].try_into().expect("32 bytes")))
}

/// A memo and its leaf.
#[wasm_bindgen(getter_with_clone)]
pub struct StoneMemo {
    pub memo: String,
    pub leaf: String,
}

/// A record: the sorted leaves, the root and the 43 bytes of the `OP_RETURN`.
#[wasm_bindgen(getter_with_clone)]
pub struct StoneRecord {
    pub leaves: Vec<String>,
    pub root: String,
    pub record: String,
}

/// The result of a receipt check.
#[wasm_bindgen(getter_with_clone)]
pub struct StoneCheck {
    pub topic: String,
    /// The body of the memo: a text hash or a text.
    pub body: String,
    pub leaf: String,
    /// The root from the leaf and the path.
    pub root: String,
    /// The root in the record.
    pub record_root: String,
    pub record_number: u32,
    pub leaf_count: u32,
    pub equal: bool,
}

/// Calculates the text hash of a text salt and a text.
#[wasm_bindgen]
pub fn stone_text_hash(text_salt: &str, text: &str) -> Result<String, JsError> {
    text_hash(text_salt, text).map_err(|error| JsError::new(&error))
}

/// Writes the memo of a topic, a salt and a body, and calculates its leaf.
#[wasm_bindgen]
pub fn stone_memo(topic: &str, salt: &str, body: &str) -> Result<StoneMemo, JsError> {
    let memo = write_memo(topic, salt, body).map_err(|error| JsError::new(&error))?;
    let leaf = hex::encode(leaf_hash(memo.as_bytes()));
    Ok(StoneMemo { memo, leaf })
}

/// Makes record `number` from the leaves of a period. Record 0 has no leaves and a root of
/// 32 zero bytes. Each later record has one leaf or more.
#[wasm_bindgen]
pub fn stone_record(number: u32, leaves: Vec<String>) -> Result<StoneRecord, JsError> {
    build_record(number, &leaves).map_err(|error| JsError::new(&error))
}

fn build_record(number: u32, leaves: &[String]) -> Result<StoneRecord, String> {
    let sorted = sorted_leaves(leaves)?;
    if number == 0 && !sorted.is_empty() {
        return Err("Record 0 must have no leaves.".into());
    }
    if number > 0 && sorted.is_empty() {
        return Err("A record after record 0 must have one leaf or more.".into());
    }
    let count = sorted.len() as u16;
    let tree = levels(sorted);
    let root = tree.last().expect("one level or more").first().copied().unwrap_or([0; 32]);
    Ok(StoneRecord {
        leaves: tree[0].iter().map(hex::encode).collect(),
        root: hex::encode(root),
        record: hex::encode(record_bytes(number, count, &root)),
    })
}

/// The path of `leaf` in the record with these leaves.
#[wasm_bindgen]
pub fn stone_path(leaves: Vec<String>, leaf: &str) -> Result<Vec<String>, JsError> {
    find_path(&leaves, leaf).map_err(|error| JsError::new(&error))
}

fn find_path(leaves: &[String], leaf: &str) -> Result<Vec<String>, String> {
    let sorted = sorted_leaves(leaves)?;
    let index = sorted
        .binary_search(&decode32(leaf)?)
        .map_err(|_| "The leaf is not in this record.".to_string())?;
    Ok(path_of(&levels(sorted), index).iter().map(hex::encode).collect())
}

/// Examines a receipt: calculates the root from the memo and the path, and compares it
/// with the root in the record.
#[wasm_bindgen]
pub fn stone_check(memo: &str, path: Vec<String>, record: &str) -> Result<StoneCheck, JsError> {
    check_receipt(memo, &path, record).map_err(|error| JsError::new(&error))
}

fn check_receipt(memo: &str, path: &[String], record: &str) -> Result<StoneCheck, String> {
    let (topic, _salt, body) = read_memo(memo)?;
    let path = path.iter().map(|hash| decode32(hash)).collect::<Result<Vec<_>, _>>()?;
    let (record_number, leaf_count, record_root) = read_record(record)?;
    let leaf = leaf_hash(memo.as_bytes());
    let root = root_from(leaf, &path);
    Ok(StoneCheck {
        topic: topic.to_string(),
        body: body.to_string(),
        leaf: hex::encode(leaf),
        root: hex::encode(root),
        record_root: hex::encode(record_root),
        record_number,
        leaf_count: u32::from(leaf_count),
        equal: root == record_root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXTS: [&str; 5] = [
        "Agent A found a proof of lemma 12. Evidence: 9c41e07b.",
        "Agent B: test 7 fails with seed 3.",
        "Agent C: the search of range 4 is complete. No result.",
        "Agent D: a new record of 2,318 gates.",
        "Agent E: the result of run 19 does not repeat.",
    ];

    /// Memo `i` of the reference vectors: topic "research", a salt of 32 equal bytes
    /// (0x11, 0x22, ...) and text `i`.
    fn memo(i: usize) -> String {
        let salt = hex::encode([0x11 * (i as u8 + 1); 32]);
        write_memo("research", &salt, TEXTS[i]).unwrap()
    }

    fn leaves(count: usize) -> Vec<String> {
        (0..count).map(|i| hex::encode(leaf_hash(memo(i).as_bytes()))).collect()
    }

    // The reference values are from a Python implementation:
    //   leaf = hashlib.blake2b(memo, digest_size=32, person=b"MemoStones__Leaf")
    //   node = hashlib.blake2b(min(a, b) + max(a, b), digest_size=32, person=b"MemoStones__Node")

    #[test]
    fn leaf_matches_reference_vector() {
        assert_eq!(memo(0).len(), 148);
        assert_eq!(leaves(1)[0], "be22cbcbc86b7d07cc611f83e55b327608d8410d8d90da07bba90d05b8cd00c9");
    }

    #[test]
    fn records_match_reference_vectors() {
        let roots = [
            (1, "be22cbcbc86b7d07cc611f83e55b327608d8410d8d90da07bba90d05b8cd00c9"),
            (2, "5f827617752d0341026ad6af0d63ab7d590fbd39c323898987ff4f5fb2c07227"),
            (3, "f861439b8b8f1e579c37c9419dcdb6bcfeded02de1db6049246e8156c5818984"),
            (5, "f14ab0ffc24a1102bd0db2df7b49d4b2f1415b4598563bc3c746211dcc5fe02f"),
        ];
        for (count, root) in roots {
            let record = build_record(7, &leaves(count)).unwrap();
            assert_eq!(record.root, root, "{count} leaves");
            assert_eq!(record.record, format!("4d5354310100000007{count:04x}{root}"));
        }
    }

    #[test]
    fn path_matches_reference_vector() {
        let all = leaves(3);
        let path = find_path(&all, &all[0]).unwrap();
        assert_eq!(path, ["f5aa3cb75598d332b6f74d2698650b842608a64f58fe1646a2a1225ff8472b2c"]);
    }

    #[test]
    fn each_receipt_is_correct() {
        for count in 1..=5 {
            let all = leaves(count);
            let record = build_record(1, &all).unwrap();
            for (i, leaf) in all.iter().enumerate() {
                let path = find_path(&all, leaf).unwrap();
                let check = check_receipt(&memo(i), &path, &record.record).unwrap();
                assert!(check.equal, "leaf {i} of {count}");
                assert_eq!(check.body, TEXTS[i]);
                assert_eq!(check.leaf_count as usize, count);
            }
        }
    }

    #[test]
    fn a_changed_receipt_fails() {
        let all = leaves(3);
        let record = build_record(7, &all).unwrap();
        let path = find_path(&all, &all[0]).unwrap();

        let changed_text = memo(0).replace("lemma 12", "lemma 13");
        assert!(!check_receipt(&changed_text, &path, &record.record).unwrap().equal);

        let changed_salt = memo(0).replacen("11", "12", 1);
        assert!(!check_receipt(&changed_salt, &path, &record.record).unwrap().equal);

        let other_path = vec!["00".repeat(32)];
        assert!(!check_receipt(&memo(0), &other_path, &record.record).unwrap().equal);

        let other_record = build_record(8, &leaves(2)).unwrap();
        assert!(!check_receipt(&memo(0), &path, &other_record.record).unwrap().equal);
    }

    #[test]
    fn bad_memos_fail() {
        let salt = "11".repeat(32);
        assert!(write_memo("", &salt, "x").is_err());
        assert!(write_memo("Suggestions", &salt, "x").is_err());
        assert!(write_memo(&"a".repeat(33), &salt, "x").is_err());
        assert!(write_memo("box", &salt.to_uppercase().replace('1', "A"), "x").is_err());
        assert!(write_memo("box", &salt[..62], "x").is_err());
        assert!(write_memo("box", &salt, "").is_err());
        assert!(write_memo("box", &salt, "x\0").is_err());
        // The header is 118 bytes at most. Thus, 394 bytes or more are free for the body.
        assert!(write_memo(&"a".repeat(32), &salt, &"x".repeat(394)).is_ok());
        assert!(write_memo(&"a".repeat(32), &salt, &"x".repeat(395)).is_err());

        let good = write_memo("box", &salt, "x").unwrap();
        assert!(read_memo(&good).is_ok());
        assert!(read_memo(&good.replace("ms1", "ms2")).is_err());
        assert!(read_memo(&good.replace("}\n", "} ")).is_err());
        assert!(read_memo(&good.replace("box,", "box,,")).is_err());
        assert!(read_memo(&good[..good.len() - 1]).is_err());
        assert!(read_memo(&format!("{good}\0")).is_err());
    }

    #[test]
    fn text_hash_matches_reference_vector() {
        // text hash = hashlib.blake2b(text_salt + text, digest_size=32, person=b"MemoStones__Text")
        let hash = text_hash(&"aa".repeat(32), TEXTS[0]).unwrap();
        assert_eq!(hash, "56e45febb21d6cec78018fd36fbaf2f3be28b07df868600340f8d9f1fededa91");
        let memo = write_memo("research", &"11".repeat(32), &hash).unwrap();
        assert_eq!(memo.len(), 158);
        assert_eq!(
            hex::encode(leaf_hash(memo.as_bytes())),
            "69878e1810d29b72eaadaf9554335d5ef7c6f36c3e83738f443557b52addb483"
        );
        assert_ne!(text_hash(&"ab".repeat(32), TEXTS[0]).unwrap(), hash);
        assert!(text_hash(&"aa".repeat(31), TEXTS[0]).is_err());
        assert!(text_hash(&"aa".repeat(32), "").is_err());
        assert!(text_hash(&"aa".repeat(32), "x\0").is_err());
    }

    #[test]
    fn equal_leaves_of_one_period_are_one_leaf() {
        // An author pays the same link two times. The record has the leaf one time, and the
        // receipts of all the authors stay correct.
        let all = leaves(3);
        let mut period = all.clone();
        period.push(all[1].clone());
        let record = build_record(7, &period).unwrap();
        assert_eq!(record.leaves.len(), 3);
        assert_eq!(record.record, build_record(7, &all).unwrap().record);
        for (i, leaf) in all.iter().enumerate() {
            let path = find_path(&period, leaf).unwrap();
            assert!(check_receipt(&memo(i), &path, &record.record).unwrap().equal);
        }
    }

    #[test]
    fn record_0_has_no_leaves() {
        let record = build_record(0, &[]).unwrap();
        assert_eq!(record.record, format!("4d53543101{}", "00".repeat(38)));
        assert_eq!(read_record(&record.record).unwrap(), (0, 0, [0; 32]));
        assert!(build_record(0, &leaves(1)).is_err());
        assert!(build_record(1, &[]).is_err());
    }

    #[test]
    fn bad_records_fail() {
        assert!(build_record(1, &["00".repeat(31)]).is_err());
        assert!(read_record(&"00".repeat(43)).is_err());
        assert!(read_record("4d53543101").is_err());
    }
}
