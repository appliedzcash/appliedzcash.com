// Shared demo UI: the status pill and the value rows.

// Sets the status pill. With no error, the pill shows "Ready".
export function setStatus(el, error) {
  el.dataset.state = error ? "error" : "ready";
  el.textContent = error ? `Error: ${error}` : "Ready";
}

const HEX = /^[0-9a-f]{16,}$/;
const shown = new WeakMap(); // list -> Map(name -> last value), to mark changed groups

// A hex value shows in groups of 8 characters, under a fingerprint bar: four colors from its
// first bytes, so equal values look equal at a glance. Groups that changed since the last
// render flash once. The spans add no characters, so a copy gives the clean value.
function hexValue(value, last) {
  const bar = document.createElement("span");
  bar.className = "fingerprint";
  bar.setAttribute("aria-hidden", "true");
  for (let i = 0; i < 4; i += 1) {
    const segment = document.createElement("span");
    segment.style.setProperty("--h", Math.round(parseInt(value.slice(i * 4, i * 4 + 4), 16) / 182));
    bar.append(segment);
  }
  const code = document.createElement("code");
  code.className = "hex";
  for (let i = 0; i < value.length; i += 8) {
    const group = document.createElement("span");
    group.textContent = value.slice(i, i + 8);
    if (last !== undefined && last.slice(i, i + 8) !== group.textContent) group.className = "changed";
    code.append(group);
  }
  return [bar, code];
}

// Fills a <dl class="values"> with [name, value] rows. A value is a string or a DOM node.
export function showValues(list, rows) {
  const before = shown.get(list) ?? new Map();
  const after = new Map();
  list.replaceChildren(
    ...rows.flatMap(([name, value]) => {
      const dt = document.createElement("dt");
      dt.textContent = name;
      const dd = document.createElement("dd");
      if (value instanceof Node) {
        dd.append(value);
      } else if (HEX.test(value)) {
        dd.append(...hexValue(value, before.get(name)));
        after.set(name, value);
      } else {
        const code = document.createElement("code");
        code.textContent = value;
        dd.append(code);
      }
      return [dt, dd];
    }),
  );
  shown.set(list, after);
}
