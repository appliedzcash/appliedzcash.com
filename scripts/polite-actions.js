// Polite actions calculator: the scan cost that an app puts on every light wallet.

// Bytes of each Orchard-protocol action in a compact block: nullifier, cmx,
// ephemeral key and the 52-byte ciphertext prefix.
const COMPACT_ACTION_BYTES = 148;

// The root note (spend and output) and the shielded fee note with its change. With a
// transparent fee, the second action is padding: a bundle has two actions or more.
const BASE_ACTIONS = 2;

const $ = (id) => document.getElementById(id);
const txInput = $("tx-input");
const usersInput = $("users-input");
const chunksInput = $("chunks-input");
const body = $("cost-body");
const result = $("cost-result");
const bars = $("cost-bars");

const count = new Intl.NumberFormat("en-US");
const decimal = new Intl.NumberFormat("en-US", { maximumSignificantDigits: 3 });

function bytes(n) {
  const units = ["bytes", "kB", "MB", "GB", "TB"];
  let i = 0;
  while (n >= 1000 && i < units.length - 1) {
    n /= 1000;
    i += 1;
  }
  return `${i === 0 ? count.format(n) : decimal.format(n)} ${units[i]}`;
}

function read(input) {
  const n = Math.floor(Number(input.value));
  return Number.isFinite(n) && n > 0 ? n : 0;
}

function update() {
  const txPerDay = read(txInput);
  const users = read(usersInput);
  const chunks = read(chunksInput);

  const designs = [
    ["One note per player", BASE_ACTIONS + users + chunks],
    ["One note per epoch", BASE_ACTIONS + chunks],
  ].map(([name, perTx]) => ({ name, perTx, perDay: perTx * txPerDay }));

  body.replaceChildren(
    ...designs.map(({ name, perTx, perDay }) => {
      const row = document.createElement("tr");
      const cells = [
        name,
        count.format(perTx),
        count.format(perDay),
        bytes(perDay * COMPACT_ACTION_BYTES),
        bytes(perDay * COMPACT_ACTION_BYTES * 365),
      ];
      for (const [i, text] of cells.entries()) {
        const cell = document.createElement(i === 0 ? "th" : "td");
        if (i === 0) cell.scope = "row";
        cell.textContent = text;
        row.append(cell);
      }
      return row;
    }),
  );

  const max = Math.max(...designs.map((d) => d.perDay), 1);
  bars.replaceChildren(
    ...designs.map(({ name, perDay }) => {
      const row = document.createElement("div");
      row.className = "bar";
      row.style.setProperty("--w", `${(perDay / max) * 100}%`);
      const label = document.createElement("span");
      label.textContent = name;
      const value = document.createElement("span");
      value.textContent = `${count.format(perDay)} actions each day`;
      row.append(label, value);
      return row;
    }),
  );

  const [perPlayer, perEpoch] = designs;
  if (perEpoch.perDay > 0 && perPlayer.perDay > perEpoch.perDay) {
    const ratio = document.createElement("strong");
    ratio.className = "ratio";
    ratio.textContent = `${decimal.format(perPlayer.perDay / perEpoch.perDay)}×`;
    result.replaceChildren(ratio, " fewer actions with one note per epoch");
  } else {
    result.textContent = "The two designs have the same number of actions.";
  }
}

for (const input of [txInput, usersInput, chunksInput]) {
  input.addEventListener("input", update);
}
update();
