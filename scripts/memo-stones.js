// Memo stones demo: a toy record of research claims. The demo crate writes the memo, calculates
// the leaf, makes the record of four leaves and examines the receipt. The salt is from the browser.

import { loadDemo } from "./wasm.js";
import { memoMeter, setStatus, showValues } from "./values.js";

const AMOUNT = "0.001"; // ZEC
const RECORD_NUMBER = 7;

// The memos of three other agents. Their leaves are in the same record.
const OTHERS = [
  ["22".repeat(32), "Agent B: test 7 fails with seed 3."],
  ["33".repeat(32), "Agent C: the search of range 4 is complete. No result."],
  ["44".repeat(32), "Agent D: a new record of 2,318 gates."],
];

const $ = (id) => document.getElementById(id);
const topicInput = $("topic-input");
const textInput = $("text-input");
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

const randomHex = (bytes) =>
  Array.from(crypto.getRandomValues(new Uint8Array(bytes)), (b) => b.toString(16).padStart(2, "0")).join("");

try {
  const demo = await loadDemo();
  const serviceAddress = demo.customer_addresses(0, 1).addresses[0];
  const otherLeaves = OTHERS.map(([salt, text]) => demo.stone_memo("research", salt, text).leaf);
  let record = ""; // The 43 bytes of the record on the toy chain, as hex.
  let receipt = ""; // The correct receipt for the inputs of this moment.

  const clearCheck = () => {
    checkValues.replaceChildren();
    result.className = "result";
    result.textContent = "";
  };

  // Examines the receipt in the box against the record, as a verifier does.
  const check = () => {
    clearCheck();
    if (!record) return;
    let memo, path;
    try {
      ({ memo, path } = JSON.parse(receiptInput.value));
      if (typeof memo !== "string" || !Array.isArray(path) || path.some((hash) => typeof hash !== "string")) {
        throw new Error("format");
      }
    } catch {
      return setStatus(status, "The receipt must be JSON with a memo and a path.");
    }
    try {
      const checked = demo.stone_check(memo, path, record);
      showValues(checkValues, [
        ["Topic", checked.topic],
        ["Text", checked.text],
        ["Leaf", checked.leaf],
        ["Root from the receipt", checked.root],
        ["Root in the record", checked.record_root],
      ]);
      result.className = `result ${checked.equal ? "match" : "mismatch"}`;
      result.textContent = checked.equal
        ? `The two roots are equal. Record ${checked.record_number} contains this memo.`
        : "The two roots are different. The record does not contain this memo.";
      setStatus(status);
    } catch (error) {
      setStatus(status, error.message);
    }
  };

  const fail = (message) => {
    record = receipt = "";
    memoValues.replaceChildren();
    qr.replaceChildren();
    recordValues.replaceChildren();
    receiptInput.value = "";
    clearCheck();
    setStatus(status, message);
  };

  const update = () => {
    try {
      const mine = demo.stone_memo(topicInput.value.trim(), saltInput.value.trim(), textInput.value);
      const link = demo.payment_link(serviceAddress, AMOUNT, mine.memo, "", "");
      const leaves = [mine.leaf, ...otherLeaves];
      const made = demo.stone_record(RECORD_NUMBER, leaves);
      const path = demo.stone_path(leaves, mine.leaf);
      record = made.record;
      receipt = JSON.stringify({ memo: mine.memo, record: RECORD_NUMBER, path }, null, 2);

      showValues(memoValues, [
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
      ]);
      receiptInput.value = receipt;
      check();
    } catch (error) {
      fail(error.message);
    }
  };

  saltButton.addEventListener("click", () => {
    saltInput.value = randomHex(32);
    update();
  });
  receiptButton.addEventListener("click", () => {
    receiptInput.value = receipt;
    check();
  });
  for (const input of [topicInput, textInput, saltInput]) input.addEventListener("input", update);
  receiptInput.addEventListener("input", check);

  saltInput.value = randomHex(32);
  for (const el of [topicInput, textInput, saltInput, saltButton, receiptInput, receiptButton]) el.disabled = false;
  update();
} catch (error) {
  setStatus(status, `the browser cannot load the WASM module (${error.message}).`);
}
