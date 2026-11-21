// Verified address demo: a toy directory. Each added record is fixed: the directory keeps its
// address, salt and values. The public list shows the claim and the cmx only.

import { loadDemo } from "./wasm.js";
import { setStatus, showValues } from "./values.js";

const GENESIS_PREV = "00".repeat(32);

const $ = (id) => document.getElementById(id);
const addressInput = $("address-input");
const saltInput = $("salt-input");
const saltButton = $("salt-button");
const nextValues = $("next-values");
const recordButton = $("record-button");
const resetButton = $("reset-button");
const chainList = $("chain-list");
const chainEmpty = $("chain-empty");
const verifySelect = $("verify-select");
const verifyAddress = $("verify-address");
const verifySalt = $("verify-salt");
const verifyValues = $("verify-values");
const verifyResult = $("verify-result");
const status = $("demo-status");

const chain = []; // { address, salt, record }

const randomHex = (bytes) =>
  Array.from(crypto.getRandomValues(new Uint8Array(bytes)), (b) => b.toString(16).padStart(2, "0")).join("");

// A record's prev is the cmx of the record before it. Its rho is the nullifier of that record.
function recordAfter(demo, last, address, salt) {
  return demo.verified_record(address, salt, last?.cmx ?? GENESIS_PREV, last?.nf ?? demo.genesis_rho());
}

function showChain() {
  chainList.replaceChildren(
    ...chain.map(({ record }) => {
      const li = document.createElement("li");
      const values = document.createElement("dl");
      values.className = "values";
      showValues(values, [
        ["claim", record.claim],
        ["cmx", record.cmx],
      ]);
      li.append(values);
      return li;
    }),
  );
  chainEmpty.hidden = chain.length > 0;
  verifySelect.replaceChildren(...chain.map((_, i) => new Option(`Record ${i + 1}`, String(i))));
  verifySelect.value = String(chain.length - 1);
  for (const el of [verifySelect, verifyAddress, verifySalt, resetButton]) el.disabled = chain.length === 0;
}

function run(demo) {
  const tip = chain.at(-1)?.record;
  const next = recordAfter(demo, tip, addressInput.value, saltInput.value);
  showValues(nextValues, [
    ["claim", next.claim],
    ["prev", tip?.cmx ?? GENESIS_PREV],
    ["ρ", tip?.nf ?? demo.genesis_rho()],
    ["rseed", next.rseed],
    ["cmx", next.cmx],
  ]);

  if (chain.length === 0) {
    verifyValues.replaceChildren();
    verifyResult.textContent = "";
    return;
  }
  const i = Number(verifySelect.value);
  const claimed = recordAfter(demo, chain[i - 1]?.record, verifyAddress.value, verifySalt.value);
  const onChain = chain[i].record.cmx;
  showValues(verifyValues, [
    ["cmx on the chain", onChain],
    ["cmx from this receipt", claimed.cmx],
  ]);
  const match = claimed.cmx === onChain;
  verifyResult.className = `result ${match ? "match" : "mismatch"}`;
  verifyResult.textContent = match
    ? "The two values are equal. This address and this salt are bound in the record."
    : "The two values are different. A different address or salt is bound in the record.";
}

try {
  const demo = await loadDemo();
  const update = () => {
    try {
      run(demo);
      setStatus(status);
    } catch (error) {
      setStatus(status, error.message);
    }
  };

  // The receipt fields show the address and the salt of the selected record.
  const showReceipt = () => {
    const entry = chain[Number(verifySelect.value)];
    verifyAddress.value = entry?.address ?? "";
    verifySalt.value = entry?.salt ?? "";
  };

  recordButton.addEventListener("click", () => {
    const address = addressInput.value.trim();
    const salt = saltInput.value.trim();
    try {
      chain.push({ address, salt, record: recordAfter(demo, chain.at(-1)?.record, address, salt) });
    } catch (error) {
      setStatus(status, error.message);
      return;
    }
    saltInput.value = randomHex(32); // The service makes a new salt for each record.
    showChain();
    showReceipt();
    update();
  });
  resetButton.addEventListener("click", () => {
    chain.length = 0;
    showChain();
    showReceipt();
    update();
  });
  saltButton.addEventListener("click", () => {
    saltInput.value = randomHex(32);
    update();
  });
  verifySelect.addEventListener("change", () => {
    showReceipt();
    update();
  });
  for (const el of [addressInput, saltInput, verifyAddress, verifySalt]) el.addEventListener("input", update);

  // The demo shop key gives a Testnet address for the user.
  addressInput.value = demo.customer_addresses(1, 1).addresses[0];
  saltInput.value = randomHex(32);
  for (const el of [addressInput, saltInput, saltButton, recordButton]) el.disabled = false;
  showChain();
  update();
} catch (error) {
  setStatus(status, `the browser cannot load the WASM module (${error.message}).`);
}
