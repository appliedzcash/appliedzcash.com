// Bound notes demo: a toy timestamp registry. The chain is a list of documents.
// Each change recalculates every stamp, because each stamp binds the cmx of the one before it.

import { loadDemo } from "./wasm.js";

const GENESIS_PREV = "00".repeat(32);

const $ = (id) => document.getElementById(id);
const docInput = $("doc-input");
const poolSelect = $("pool-select");
const nextValues = $("next-values");
const stampButton = $("stamp-button");
const resetButton = $("reset-button");
const chainList = $("chain-list");
const chainEmpty = $("chain-empty");
const verifySelect = $("verify-select");
const verifyInput = $("verify-input");
const verifyValues = $("verify-values");
const verifyResult = $("verify-result");
const status = $("demo-status");

const documents = [];

function showValues(list, rows) {
  list.replaceChildren(
    ...rows.flatMap(([name, value]) => {
      const dt = document.createElement("dt");
      dt.textContent = name;
      const dd = document.createElement("dd");
      const code = document.createElement("code");
      code.textContent = value;
      dd.append(code);
      return [dt, dd];
    }),
  );
}

function run(demo) {
  const ironwood = poolSelect.value === "ironwood";

  // Each stamp's prev is the cmx before it. Its rho is the nullifier before it.
  const stampAfter = (stamps, text) => {
    const last = stamps.at(-1);
    return demo.stamp(text, last?.cmx ?? GENESIS_PREV, last?.nf ?? demo.genesis_rho(), ironwood);
  };
  const chain = [];
  for (const text of documents) {
    chain.push(stampAfter(chain, text));
  }

  const next = stampAfter(chain, docInput.value);
  showValues(nextValues, [
    ["fields (document hash)", next.doc_hash],
    ["prev", chain.at(-1)?.cmx ?? GENESIS_PREV],
    ["ρ", chain.at(-1)?.nf ?? demo.genesis_rho()],
    ["rseed", next.rseed],
    ["cmx", next.cmx],
  ]);

  chainList.replaceChildren(
    ...chain.map((stamp, i) => {
      const li = document.createElement("li");
      const doc = document.createElement("p");
      doc.textContent = documents[i];
      const values = document.createElement("dl");
      values.className = "values";
      showValues(values, [["cmx", stamp.cmx]]);
      li.append(doc, values);
      return li;
    }),
  );
  chainEmpty.hidden = chain.length > 0;

  const selected = verifySelect.value;
  verifySelect.replaceChildren(
    ...chain.map((_, i) => new Option(`Stamp ${i + 1}`, String(i))),
  );
  verifySelect.value = selected && Number(selected) < chain.length ? selected : String(chain.length - 1);
  const enabled = chain.length > 0;
  verifySelect.disabled = verifyInput.disabled = resetButton.disabled = !enabled;

  if (!enabled) {
    verifyValues.replaceChildren();
    verifyResult.textContent = "";
    return;
  }
  const i = Number(verifySelect.value);
  const claimed = stampAfter(chain.slice(0, i), verifyInput.value);
  showValues(verifyValues, [
    ["cmx on the chain", chain[i].cmx],
    ["cmx from this document", claimed.cmx],
  ]);
  const match = claimed.cmx === chain[i].cmx;
  verifyResult.className = `result ${match ? "match" : "mismatch"}`;
  verifyResult.textContent = match
    ? "The two values are equal. The stamp binds this document."
    : "The two values are different. The stamp binds a different document.";
}

try {
  const demo = await loadDemo();
  const update = () => {
    try {
      run(demo);
      status.textContent = "The WASM module is loaded.";
    } catch (error) {
      status.textContent = `Error: ${error.message}`;
    }
  };

  stampButton.addEventListener("click", () => {
    documents.push(docInput.value);
    verifySelect.value = "";
    verifyInput.value = docInput.value;
    update();
  });
  resetButton.addEventListener("click", () => {
    documents.length = 0;
    verifyInput.value = "";
    update();
  });
  for (const el of [docInput, poolSelect, verifySelect, verifyInput]) {
    el.addEventListener(el.tagName === "SELECT" ? "change" : "input", update);
  }

  docInput.disabled = poolSelect.disabled = stampButton.disabled = false;
  update();
} catch (error) {
  status.textContent = `Error: the WASM module failed to load (${error.message}).`;
}
