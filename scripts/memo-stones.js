// Memo stones demo: a toy record of research claims. The demo crate calculates the text hash,
// writes the memo, calculates the leaf, makes the record of four leaves and examines the receipt.
// The salts are from the browser. The toy chain is a table in this script.

import { loadDemo } from "./wasm.js";
import { memoMeter, setStatus, showValues } from "./values.js";

const AMOUNT = "0.001"; // ZEC
const RECORD_NUMBER = 7;

// The memos of three other agents: salt, text salt and text. Their leaves are in the same record.
const OTHERS = [
  ["22".repeat(32), "bb".repeat(32), "Agent B: test 7 fails with seed 3."],
  ["33".repeat(32), "cc".repeat(32), "Agent C: the search of range 4 is complete. No result."],
  ["44".repeat(32), "dd".repeat(32), "Agent D: a new record of 2,318 gates."],
];

const $ = (id) => document.getElementById(id);
const topicInput = $("topic-input");
const textInput = $("text-input");
const plainInput = $("plain-input");
const saltInput = $("salt-input");
const saltButton = $("salt-button");
const memoValues = $("memo-values");
const qr = $("link-qr");
const recordValues = $("record-values");
const receiptInput = $("receipt-input");
const receiptButton = $("receipt-button");
const checkValues = $("check-values");
const result = $("check-result");
const status = $("demo-status");

const toHex = (bytes) => Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
const randomHex = (bytes) => toHex(crypto.getRandomValues(new Uint8Array(bytes)));

// The toy transaction ID of a record: SHA-256 of the 43 bytes. A real ID is the ID of the
// record transaction on the chain.
const toyTxid = async (record) => {
  const bytes = Uint8Array.from(record.match(/../g), (pair) => parseInt(pair, 16));
  return toHex(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)));
};

try {
  const demo = await loadDemo();
  const serviceAddress = demo.customer_addresses(0, 1).addresses[0];
  const otherLeaves = OTHERS.map(
    ([salt, textSalt, text]) => demo.stone_memo("research", salt, demo.stone_text_hash(textSalt, text)).leaf,
  );
  const chain = new Map(); // The toy chain: transaction ID -> the 43 bytes of the record, as hex.
  let textSalt = randomHex(32);
  let receipt = ""; // The correct receipt for the inputs of this moment.
  let turn = 0; // The number of the last update. An older update stops.

  const clearCheck = () => {
    checkValues.replaceChildren();
    result.className = "result";
    result.textContent = "";
  };

  const show = (good, message) => {
    result.className = `result ${good ? "match" : "mismatch"}`;
    result.textContent = message;
  };

  // Examines the receipt in the box against the toy chain, as a verifier does.
  const check = () => {
    clearCheck();
    if (!chain.size) return;
    let memo, txid, path, text, text_salt;
    try {
      ({ memo, txid, path, text, text_salt } = JSON.parse(receiptInput.value));
      const strings = [memo, txid, ...path];
      const disclosure = [text, text_salt].filter((value) => value !== undefined);
      if (strings.some((value) => typeof value !== "string") || disclosure.some((value) => typeof value !== "string")) {
        throw new Error("format");
      }
      if (disclosure.length === 1) throw new Error("format");
    } catch {
      return setStatus(status, "The receipt must be JSON with a memo, a txid and a path. A text needs a text_salt.");
    }
    const record = chain.get(txid);
    if (!record) {
      setStatus(status);
      return show(false, "The toy chain has no record transaction with this ID.");
    }
    try {
      const checked = demo.stone_check(memo, path, record);
      const hash = text === undefined ? "" : demo.stone_text_hash(text_salt, text);
      showValues(checkValues, [
        ["Topic", checked.topic],
        ["Body of the memo", checked.body],
        ...(hash ? [["Text hash from the receipt", hash]] : []),
        ["Leaf", checked.leaf],
        ["Root from the receipt", checked.root],
        ["Root in the record", checked.record_root],
      ]);
      const recordText = `Record ${checked.record_number} contains this memo.`;
      if (!checked.equal) show(false, "The two roots are different. The record does not contain this memo.");
      else if (!hash) show(true, `The two roots are equal. ${recordText}`);
      else if (hash === checked.body) show(true, `${recordText} The memo contains the hash of this text.`);
      else show(false, `${recordText} The text and the text salt do not give the hash in the memo.`);
      setStatus(status);
    } catch (error) {
      setStatus(status, error.message);
    }
  };

  const fail = (message) => {
    chain.clear();
    receipt = "";
    memoValues.replaceChildren();
    qr.replaceChildren();
    recordValues.replaceChildren();
    receiptInput.value = "";
    clearCheck();
    setStatus(status, message);
  };

  const update = async () => {
    const thisTurn = ++turn;
    try {
      const text = textInput.value;
      const plain = plainInput.checked;
      const body = plain ? text : demo.stone_text_hash(textSalt, text);
      const mine = demo.stone_memo(topicInput.value.trim(), saltInput.value.trim(), body);
      const link = demo.payment_link(serviceAddress, AMOUNT, mine.memo, "", "");
      const leaves = [mine.leaf, ...otherLeaves];
      const made = demo.stone_record(RECORD_NUMBER, leaves);
      const path = demo.stone_path(leaves, mine.leaf);
      const txid = await toyTxid(made.record);
      if (thisTurn !== turn) return;
      chain.clear();
      chain.set(txid, made.record);
      receipt = JSON.stringify({ memo: mine.memo, txid, path, ...(plain ? {} : { text_salt: textSalt, text }) }, null, 2);

      showValues(memoValues, [
        ...(plain ? [] : [["Text salt", textSalt], ["Text hash", body]]),
        ["Memo", mine.memo],
        ["Memo length", memoMeter(mine.memo)],
        ["Leaf", mine.leaf],
        ["ZIP 321 URI", link.uri],
      ]);
      qr.innerHTML = link.qr_svg; // SVG from the demo crate: the QR code of the URI.
      showValues(recordValues, [
        ...made.leaves.map((leaf, i) => [leaf === mine.leaf ? `Leaf ${i + 1} (your memo)` : `Leaf ${i + 1}`, leaf]),
        ["Root (the stone)", made.root],
        ["Record (43 bytes)", made.record],
        ["Toy transaction ID", txid],
      ]);
      receiptInput.value = receipt;
      check();
    } catch (error) {
      if (thisTurn === turn) fail(error.message);
    }
  };

  saltButton.addEventListener("click", () => {
    saltInput.value = randomHex(32);
    textSalt = randomHex(32);
    update();
  });
  receiptButton.addEventListener("click", () => {
    receiptInput.value = receipt;
    check();
  });
  for (const input of [topicInput, textInput, saltInput]) input.addEventListener("input", update);
  plainInput.addEventListener("change", update);
  receiptInput.addEventListener("input", check);

  saltInput.value = randomHex(32);
  for (const el of [topicInput, textInput, plainInput, saltInput, saltButton, receiptInput, receiptButton]) {
    el.disabled = false;
  }
  await update();
} catch (error) {
  setStatus(status, `the browser cannot load the WASM module (${error.message}).`);
}
