// Threshold keys demo: a 2-of-3 guard group nested in a 2-of-2 deed key.
// The browser draws the key seed and a fresh nonce seed for each signature.

import { loadDemo } from "./wasm.js";

const $ = (id) => document.getElementById(id);
const keysButton = $("keys-button");
const keygenValues = $("keygen-values");
const objectInput = $("object-input");
const objectValues = $("object-values");
const signers = $("signers");
const messageInput = $("message-input");
const signButton = $("sign-button");
const signValues = $("sign-values");
const result = $("sign-result");
const status = $("demo-status");

const randomHex = (bytes) =>
  Array.from(crypto.getRandomValues(new Uint8Array(bytes)), (b) => b.toString(16).padStart(2, "0")).join("");

let keySeed = randomHex(32);

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

const chosen = () => [...signers.querySelectorAll("input:checked")].map((box) => Number(box.value));

function run(demo, sign) {
  // The key values need a valid signer set, so this call always uses members 1 and 2.
  // A sign call uses the chosen members and a fresh nonce seed.
  const members = sign ? chosen() : [1, 2];
  try {
    const out = demo.threshold(keySeed, randomHex(32), objectInput.value, new Uint16Array(members), messageInput.value);
    showValues(keygenValues, [
      ["Group key A_G", out.group_key],
      ...out.member_keys.map((key, i) => [`Member ${i + 1} share`, key]),
    ]);
    showValues(objectValues, [
      ["Tweak t", out.tweak],
      ["Object group key", out.object_group_key],
      ["Owner share A_O", out.owner_key],
      ["ak", `${out.ak}${out.negated ? " (both sides negated)" : ""}`],
    ]);
    if (sign) {
      showValues(signValues, [
        ...out.commitment_hashes.map((line, i) => [i === 0 ? "Commitment hashes" : "", line]),
        ["Commitment set digest", out.set_digest],
        ["Randomizer α", out.alpha],
        ["rk = ak + [α]G", out.rk],
        ["Signature", out.signature],
      ]);
      result.className = `result ${out.verified ? "match" : "mismatch"}`;
      result.textContent = out.verified
        ? `The owner and members ${members.slice(0, -1).join(", ")} and ${members.at(-1)} made one signature. It verifies under rk.`
        : "The signature failed to verify.";
    }
    status.textContent = "The WASM module is loaded.";
  } catch (error) {
    if (sign) {
      signValues.replaceChildren();
      result.className = "result mismatch";
      result.textContent = error.message;
    } else {
      status.textContent = `Error: ${error.message}`;
    }
  }
}

function clearSignature() {
  signValues.replaceChildren();
  result.textContent = "";
}

try {
  const demo = await loadDemo();
  keysButton.addEventListener("click", () => {
    keySeed = randomHex(32);
    clearSignature();
    run(demo, false);
  });
  objectInput.addEventListener("input", () => {
    clearSignature();
    run(demo, false);
  });
  signButton.addEventListener("click", () => run(demo, true));
  for (const el of [keysButton, objectInput, signers, messageInput, signButton]) {
    el.disabled = false;
  }
  run(demo, false);
} catch (error) {
  status.textContent = `Error: the WASM module failed to load (${error.message}).`;
}
