// Address per customer demo: addresses of the demo shop key, and the index of a pasted address.

import { loadDemo } from "./wasm.js";
import { setStatus, showValues } from "./values.js";

const $ = (id) => document.getElementById(id);
const start = $("start-input");
const count = $("count-input");
const values = $("address-values");
const find = $("find-input");
const result = $("find-result");
const status = $("demo-status");

try {
  const demo = await loadDemo();
  const list = () => {
    try {
      const addresses = demo.customer_addresses(Number(start.value) >>> 0, Number(count.value) >>> 0);
      showValues(
        values,
        Array.from(addresses.indexes, (j, i) => [`Customer j = ${j}`, addresses.addresses[i]]),
      );
      setStatus(status);
      return addresses;
    } catch (error) {
      values.replaceChildren();
      setStatus(status, error.message);
    }
  };
  const lookup = () => {
    result.className = "result";
    if (!find.value.trim()) {
      result.textContent = "";
      return;
    }
    try {
      const j = demo.find_customer(find.value);
      if (j === undefined) {
        result.textContent = "A different key made this address.";
        result.classList.add("mismatch");
      } else {
        result.textContent = `The shop key made this address. It is customer j = ${j}.`;
        result.classList.add("match");
      }
    } catch (error) {
      result.textContent = error.message;
      result.classList.add("mismatch");
    }
  };

  start.addEventListener("input", list);
  count.addEventListener("input", list);
  find.addEventListener("input", lookup);
  for (const el of [start, count, find]) el.disabled = false;
  const first = list();
  find.value = first.addresses[first.addresses.length - 1];
  lookup();
} catch (error) {
  setStatus(status, `the WASM module failed to load (${error.message}).`);
}
