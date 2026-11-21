// Wallet-native orders demo: a toy poll. The page draws the session key and the nonce.
// The WASM module writes the memo with its MAC, the ZIP 321 URI, and the QR code.

import { loadDemo } from "./wasm.js";
import { memoMeter, setStatus, showValues } from "./values.js";

const $ = (id) => document.getElementById(id);
const keyInput = $("key-input");
const participantInput = $("participant-input");
const expiryInput = $("expiry-input");
const commandsInput = $("commands-input");
const nonceButton = $("nonce-button");
const keyButton = $("key-button");
const values = $("order-values");
const qr = $("order-qr");
const status = $("demo-status");

const randomHex = (bytes) =>
  Array.from(crypto.getRandomValues(new Uint8Array(bytes)), (b) => b.toString(16).padStart(2, "0")).join("");

let nonce = randomHex(4);

try {
  const demo = await loadDemo();
  const update = () => {
    try {
      const order = demo.order(
        keyInput.value,
        Number(participantInput.value) >>> 0,
        nonce,
        Number(expiryInput.value) >>> 0,
        commandsInput.value,
      );
      showValues(values, [
        ["Inbox", order.inbox],
        ["Memo", order.memo],
        ["Memo length", memoMeter(order.memo)],
        ["ZIP 321 URI", order.uri],
      ]);
      qr.innerHTML = order.qr_svg; // SVG from the demo crate: the QR code of the URI.
      setStatus(status);
    } catch (error) {
      values.replaceChildren();
      qr.replaceChildren();
      setStatus(status, error.message);
    }
  };

  nonceButton.addEventListener("click", () => {
    nonce = randomHex(4);
    update();
  });
  keyButton.addEventListener("click", () => {
    keyInput.value = randomHex(32);
    update();
  });
  for (const input of [keyInput, participantInput, expiryInput, commandsInput]) {
    input.addEventListener("input", update);
  }

  keyInput.value = randomHex(32);
  for (const el of [keyInput, participantInput, expiryInput, commandsInput, nonceButton, keyButton]) {
    el.disabled = false;
  }
  update();
} catch (error) {
  setStatus(status, `the browser cannot load the WASM module (${error.message}).`);
}
