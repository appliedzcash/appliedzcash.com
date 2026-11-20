// Concepts demo: decode a Zcash address, and build a unified address from its receivers.

import { loadDemo } from "./wasm.js";
import { setStatus, showValues } from "./values.js";

const $ = (id) => document.getElementById(id);
const input = $("address-input");
const values = $("address-values");
const status = $("demo-status");
const examples = document.querySelectorAll("[data-example]");
const builder = $("builder");
const switches = $("receiver-switches");
const bar = $("ua-bar");
const partList = $("ua-parts");
const jumbleValues = $("jumble-values");
const builtValues = $("built-values");

const el = (tag, className, text) => {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
};

// A color square for one receiver type. CSS gives the color from data-kind.
const chip = (kind) => {
  const square = el("span", "chip");
  square.dataset.kind = kind;
  square.setAttribute("aria-hidden", "true");
  return square;
};

try {
  const demo = await loadDemo();
  let source = ""; // the unified address that the boxes show
  const off = new Set(); // the items of `source` that are off, by index

  // One box for each receiver of the full address. The last row of a construction is the padding.
  const showSwitches = (full) => {
    switches.replaceChildren(
      switches.querySelector("legend"),
      ...full.names.slice(0, -1).map((name, i) => {
        const label = el("label");
        const box = el("input");
        box.type = "checkbox";
        box.checked = true;
        box.addEventListener("change", () => {
          if (box.checked) off.delete(i);
          else off.add(i);
          update();
        });
        label.append(box, chip(full.kinds[i]), name);
        return label;
      }),
    );
  };

  // The bytes before F4Jumble: a bar to scale, then one row for each item.
  const showBytes = (construction) => {
    const built = {
      names: construction.names,
      heads: construction.heads,
      data: construction.data,
      kinds: construction.kinds,
    };
    const sizes = built.names.map((_, i) => (built.heads[i].length + built.data[i].length) / 2);
    bar.replaceChildren(
      ...built.kinds.map((kind, i) => {
        const segment = el("span");
        segment.dataset.kind = kind;
        segment.style.flexGrow = sizes[i];
        return segment;
      }),
    );
    partList.replaceChildren(
      ...built.names.map((name, i) => {
        const item = el("li");
        item.dataset.kind = built.kinds[i];
        const title = el("span", "ua-name");
        title.append(chip(built.kinds[i]), name, el("span", "ua-size", `${sizes[i]} bytes`));
        const hex = el("code", "hex");
        if (built.heads[i]) hex.append(el("span", "head", built.heads[i]));
        for (let at = 0; at < built.data[i].length; at += 8) hex.append(el("span", "", built.data[i].slice(at, at + 8)));
        item.append(title, hex);
        return item;
      }),
    );
  };

  const summary = (parts, receivers) => [
    ["Type", parts.kind],
    ["Network", parts.network],
    ["Memo", parts.memo ? "Yes. A payment to this address can have a memo." : "No. A payment to this address has no memo."],
    ...(receivers ? parts.names.map((name, i) => [name, parts.values[i]]) : []),
  ];

  const update = () => {
    const text = input.value.trim();
    try {
      const parts = demo.inspect(text);
      builder.hidden = !parts.unified;
      if (!parts.unified) {
        source = "";
        showValues(values, summary(parts, true));
        setStatus(status);
        return;
      }
      if (text !== source) {
        source = text;
        off.clear();
        showSwitches(demo.construct(text, new Uint32Array()));
      }
    } catch (error) {
      source = "";
      builder.hidden = true;
      values.replaceChildren();
      setStatus(status, error.message);
      return;
    }
    try {
      const built = demo.construct(text, Uint32Array.from(off));
      showBytes(built);
      showValues(jumbleValues, [["All bytes", built.jumbled]]);
      showValues(builtValues, [
        ["Prefix", built.prefix],
        ["Address", built.address],
      ]);
      showValues(values, summary(demo.inspect(built.address), false));
      setStatus(status);
    } catch (error) {
      for (const list of [bar, partList, jumbleValues, builtValues, values]) list.replaceChildren();
      setStatus(status, error.message);
    }
  };

  input.addEventListener("input", update);
  for (const button of examples) {
    button.addEventListener("click", () => {
      input.value = button.dataset.example;
      update();
    });
  }
  for (const control of [input, ...examples]) control.disabled = false;
  update();
} catch (error) {
  setStatus(status, `the browser cannot load the WASM module (${error.message}).`);
}
