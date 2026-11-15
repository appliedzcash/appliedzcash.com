// Watch-only keys demo: one seed gives the spending key, the UFVK and the UIVK of account 0.

import { loadDemo } from "./wasm.js";
import { showValues } from "./values.js";

const $ = (id) => document.getElementById(id);
const seed = $("seed-input");
const randomButton = $("random-button");
const shopButton = $("shop-button");
const values = $("key-values");
const result = $("address-result");
const status = $("demo-status");

const SHOP_SEED = "07".repeat(32);
const randomHex = (bytes) =>
  Array.from(crypto.getRandomValues(new Uint8Array(bytes)), (b) => b.toString(16).padStart(2, "0")).join("");

try {
  const demo = await loadDemo();
  const update = () => {
    result.className = "result";
    try {
      const keys = demo.viewing_keys(seed.value);
      showValues(values, [
        ["Spending key (secret)", keys.spending_key],
        ["UFVK", keys.ufvk],
        ["UIVK", keys.uivk],
        ["Address 0 from the spending key", keys.wallet_address],
        ["Address 0 from the UIVK", keys.watch_address],
      ]);
      const same = keys.wallet_address === keys.watch_address;
      result.textContent = same
        ? "The two addresses are the same. The UIVK makes the addresses of the wallet."
        : "The two addresses are different.";
      result.classList.add(same ? "match" : "mismatch");
      status.textContent = "The WASM module is loaded.";
    } catch (error) {
      values.replaceChildren();
      result.textContent = "";
      status.textContent = `Error: ${error.message}`;
    }
  };

  randomButton.addEventListener("click", () => {
    seed.value = randomHex(32);
    update();
  });
  shopButton.addEventListener("click", () => {
    seed.value = SHOP_SEED;
    update();
  });
  seed.addEventListener("input", update);
  for (const el of [seed, randomButton, shopButton]) el.disabled = false;
  seed.value = SHOP_SEED;
  update();
} catch (error) {
  status.textContent = `Error: the WASM module failed to load (${error.message}).`;
}
