// Bound notes demo: a toy timestamp registry. Each added stamp is fixed: the chain keeps its
// document and values. The pool locks while the chain holds stamps, so every stamp shares one version.

import { loadDemo } from "./wasm.js";
import { setStatus, showValues } from "./values.js";

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

const chain = []; // { text, stamp }

const ironwood = () => poolSelect.value === "ironwood";

// A stamp's prev is the cmx of the stamp before it. Its rho is the nullifier of that stamp.
function stampAfter(demo, last, text) {
  return demo.stamp(text, last?.cmx ?? GENESIS_PREV, last?.nf ?? demo.genesis_rho(), ironwood());
}

function showChain() {
  chainList.replaceChildren(
    ...chain.map(({ text, stamp }) => {
      const li = document.createElement("li");
      const doc = document.createElement("p");
      doc.textContent = text;
      const values = document.createElement("dl");
      values.className = "values";
      showValues(values, [["cmx", stamp.cmx]]);
      li.append(doc, values);
      return li;
    }),
  );
  chainEmpty.hidden = chain.length > 0;
  poolSelect.disabled = chain.length > 0;
  verifySelect.replaceChildren(...chain.map((_, i) => new Option(`Stamp ${i + 1}`, String(i))));
  verifySelect.value = String(chain.length - 1);
  verifySelect.disabled = verifyInput.disabled = resetButton.disabled = chain.length === 0;
}

function run(demo) {
  const tip = chain.at(-1)?.stamp;
  const next = stampAfter(demo, tip, docInput.value);
  showValues(nextValues, [
    ["fields (document hash)", next.doc_hash],
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
  const claimed = stampAfter(demo, chain[i - 1]?.stamp, verifyInput.value);
  const onChain = chain[i].stamp.cmx;
  showValues(verifyValues, [
    ["cmx on the chain", onChain],
    ["cmx from this document", claimed.cmx],
  ]);
  const match = claimed.cmx === onChain;
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
      setStatus(status);
    } catch (error) {
      setStatus(status, error.message);
    }
  };

  stampButton.addEventListener("click", () => {
    try {
      chain.push({ text: docInput.value, stamp: stampAfter(demo, chain.at(-1)?.stamp, docInput.value) });
    } catch (error) {
      setStatus(status, error.message);
      return;
    }
    verifyInput.value = docInput.value;
    showChain();
    update();
  });
  resetButton.addEventListener("click", () => {
    chain.length = 0;
    verifyInput.value = "";
    showChain();
    update();
  });
  for (const el of [docInput, poolSelect, verifySelect, verifyInput]) {
    el.addEventListener(el.tagName === "SELECT" ? "change" : "input", update);
  }

  docInput.disabled = stampButton.disabled = false;
  showChain();
  update();
} catch (error) {
  setStatus(status, `the WASM module failed to load (${error.message}).`);
}
