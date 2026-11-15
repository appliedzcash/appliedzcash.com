// Concepts demo: decode a Zcash address and list its receivers.

import { loadDemo } from "./wasm.js";
import { showValues } from "./values.js";

const $ = (id) => document.getElementById(id);
const input = $("address-input");
const values = $("address-values");
const status = $("demo-status");
const examples = document.querySelectorAll("[data-example]");

try {
  const demo = await loadDemo();
  const update = () => {
    try {
      const parts = demo.inspect(input.value);
      showValues(values, [
        ["Type", parts.kind],
        ["Network", parts.network],
        ["Memo", parts.memo ? "Yes, a payment can carry a memo." : "No. A payment to this address has no memo."],
        ...parts.names.map((name, i) => [name, parts.values[i]]),
      ]);
      status.textContent = "The WASM module is loaded.";
    } catch (error) {
      values.replaceChildren();
      status.textContent = `Error: ${error.message}`;
    }
  };

  input.addEventListener("input", update);
  for (const button of examples) {
    button.addEventListener("click", () => {
      input.value = button.dataset.example;
      update();
    });
  }
  for (const el of [input, ...examples]) el.disabled = false;
  update();
} catch (error) {
  status.textContent = `Error: the WASM module failed to load (${error.message}).`;
}
