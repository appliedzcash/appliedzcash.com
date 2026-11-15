// Payment links demo: the zip321 crate writes the URI, and the demo crate draws its QR code.

import { loadDemo } from "./wasm.js";
import { setStatus, showValues } from "./values.js";

const $ = (id) => document.getElementById(id);
const inputs = ["address-input", "amount-input", "memo-input", "label-input", "message-input"].map($);
const [address, amount, memo, label, message] = inputs;
const values = $("link-values");
const qr = $("link-qr");
const status = $("demo-status");

try {
  const demo = await loadDemo();
  const update = () => {
    try {
      const link = demo.payment_link(address.value, amount.value, memo.value, label.value, message.value);
      const rows = [["ZIP 321 URI", link.uri]];
      if (memo.value) rows.push(["Memo length", `${link.memo_length} bytes of 512`]);
      showValues(values, rows);
      qr.innerHTML = link.qr_svg; // SVG from the demo crate: the QR code of the URI.
      setStatus(status);
    } catch (error) {
      values.replaceChildren();
      qr.replaceChildren();
      setStatus(status, error.message);
    }
  };

  address.value = demo.customer_addresses(0, 1).addresses[0];
  for (const input of inputs) {
    input.addEventListener("input", update);
    input.disabled = false;
  }
  update();
} catch (error) {
  setStatus(status, `the WASM module failed to load (${error.message}).`);
}
