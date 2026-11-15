// Canonical action pairing demo. Each click builds the bundle again. The page compares the
// cmx values with the previous build of the same mode and the same inputs.

import { loadDemo } from "./wasm.js";

const $ = (id) => document.getElementById(id);
const keyInput = $("pkey-input");
const ctxInput = $("ctx-input");
const spendsInput = $("spends-input");
const outputsInput = $("outputs-input");
const actionsInput = $("actions-input");
const canonicalButton = $("canonical-button");
const standardButton = $("standard-button");
const title = $("build-title");
const seedValues = $("seed-values");
const body = $("pairing-body");
const result = $("pairing-result");
const status = $("demo-status");

const randomHex = (bytes) =>
  Array.from(crypto.getRandomValues(new Uint8Array(bytes)), (b) => b.toString(16).padStart(2, "0")).join("");
const short = (hex) => `${hex.slice(0, 8)}…${hex.slice(-4)}`;

// The cmx values of the last build, for each mode and set of inputs.
const last = new Map();

function show(bundle, mode) {
  title.textContent = `Bundle: ${mode} build`;
  seedValues.replaceChildren();
  const dt = document.createElement("dt");
  dt.textContent = mode === "canonical" ? "Pairing seed d" : "Random seed";
  const dd = document.createElement("dd");
  const code = document.createElement("code");
  code.textContent = bundle.seed;
  dd.append(code);
  seedValues.append(dt, dd);

  const { spend_labels, nullifiers, output_labels, cmxs } = bundle;
  body.replaceChildren(
    ...cmxs.map((cmx, a) => {
      const row = document.createElement("tr");
      const cells = [String(a), spend_labels[a], short(nullifiers[a]), output_labels[a], short(cmx)];
      for (const [i, text] of cells.entries()) {
        const cell = document.createElement(i === 0 ? "th" : "td");
        if (i === 0) cell.scope = "row";
        if (i === 2 || i === 4) {
          const code = document.createElement("code");
          code.textContent = text;
          code.title = i === 2 ? nullifiers[a] : cmx;
          cell.append(code);
        } else {
          cell.textContent = text;
        }
        if ((i === 1 && text.startsWith("dummy")) || (i === 3 && text.startsWith("dummy"))) {
          cell.className = "dummy";
        }
        row.append(cell);
      }
      return row;
    }),
  );
}

function run(demo, mode) {
  const inputs = [keyInput.value, ctxInput.value, spendsInput.value, outputsInput.value, actionsInput.value];
  try {
    const bundle = demo.pair(
      keyInput.value,
      ctxInput.value,
      Number(spendsInput.value) >>> 0,
      Number(outputsInput.value) >>> 0,
      Number(actionsInput.value) >>> 0,
      mode === "canonical",
      randomHex(32),
    );
    show(bundle, mode);
    const id = JSON.stringify([mode, ...inputs]);
    const cmxs = bundle.cmxs.join();
    const previous = last.get(id);
    last.set(id, cmxs);
    if (previous === undefined) {
      result.className = "result";
      result.textContent = "This is the first build with these inputs. Click the same button again.";
    } else if (previous === cmxs) {
      result.className = "result match";
      result.textContent = "The cmx values are the same as in the previous build.";
    } else {
      result.className = "result mismatch";
      result.textContent = "The cmx values are different from the previous build.";
    }
    status.textContent = "The WASM module is loaded.";
  } catch (error) {
    body.replaceChildren();
    seedValues.replaceChildren();
    result.textContent = "";
    status.textContent = `Error: ${error.message}`;
  }
}

try {
  const demo = await loadDemo();
  canonicalButton.addEventListener("click", () => run(demo, "canonical"));
  standardButton.addEventListener("click", () => run(demo, "standard"));
  keyInput.value = randomHex(32);
  for (const el of [keyInput, ctxInput, spendsInput, outputsInput, actionsInput, canonicalButton, standardButton]) {
    el.disabled = false;
  }
  run(demo, "canonical");
} catch (error) {
  status.textContent = `Error: the WASM module failed to load (${error.message}).`;
}
