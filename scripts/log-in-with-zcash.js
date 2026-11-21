// Log in with Zcash demo: a toy login. The page writes the login memo and calculates the
// code with HMAC-SHA256 (Web Crypto). The demo crate makes the ZIP 321 URI and its QR code.
// The memo format and the code are those of zcashme/zcash-verification-service.

import { loadDemo } from "./wasm.js";
import { memoMeter, setStatus, showValues } from "./values.js";

const LOGIN_AMOUNT = "0.002"; // ZEC

const $ = (id) => document.getElementById(id);
const sessionInput = $("session-input");
const addressInput = $("address-input");
const keyInput = $("key-input");
const sessionButton = $("session-button");
const keyButton = $("key-button");
const codeInput = $("code-input");
const linkValues = $("link-values");
const qr = $("link-qr");
const replyValues = $("reply-values");
const result = $("code-result");
const status = $("demo-status");

const random = (bytes) => crypto.getRandomValues(new Uint8Array(bytes));
const randomHex = (bytes) => Array.from(random(bytes), (b) => b.toString(16).padStart(2, "0")).join("");
// 16 random bytes give 16 decimal digits, one digit from each byte below 250.
const randomSession = () => {
  let digits = "";
  while (digits.length < 16) for (const b of random(16)) if (b < 250 && digits.length < 16) digits += b % 10;
  return digits;
};

// code = HMAC-SHA256(key, session ‖ address)[0..4] mod 10^6, as 6 decimal digits.
async function loginCode(keyHex, session, address) {
  const keyBytes = Uint8Array.from(keyHex.match(/../g), (byte) => parseInt(byte, 16));
  const key = await crypto.subtle.importKey("raw", keyBytes, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  const mac = await crypto.subtle.sign("HMAC", key, new TextEncoder().encode(session + address));
  return String(new DataView(mac).getUint32(0) % 1_000_000).padStart(6, "0");
}

try {
  const demo = await loadDemo();
  // The demo shop key gives both addresses: index 0 is the worker, index 1 is the user.
  const [serviceAddress, userAddress] = demo.customer_addresses(0, 2).addresses;
  let code = "";
  let run = 0;

  const check = () => {
    const typed = codeInput.value.trim();
    if (!code || !typed) {
      result.className = "result";
      result.textContent = "";
      return;
    }
    const match = typed === code;
    result.className = `result ${match ? "match" : "mismatch"}`;
    result.textContent = match
      ? "The two codes are equal. The user is logged in as the return address."
      : "The two codes are different. The app refuses the login.";
  };

  const fail = (message) => {
    code = "";
    linkValues.replaceChildren();
    qr.replaceChildren();
    replyValues.replaceChildren();
    check();
    setStatus(status, message);
  };

  const update = async () => {
    const turn = ++run;
    const session = sessionInput.value.trim();
    const address = addressInput.value.trim();
    const key = keyInput.value.trim().toLowerCase();
    if (!/^[0-9]{16}$/.test(session)) return fail("The session ID must be 16 decimal digits.");
    if (!/^[!-+\--z|~]+$/.test(address)) return fail("Type a return address. Use printable ASCII with no space, comma or brace.");
    if (!/^[0-9a-f]{64}$/.test(key)) return fail("The code key must be 64 hexadecimal characters.");
    try {
      const memo = `DO NOT MODIFY:{zvs/${session},${address}}`;
      const link = demo.payment_link(serviceAddress, LOGIN_AMOUNT, memo, "", "");
      const next = await loginCode(key, session, address);
      if (turn !== run) return; // A newer input is in work.
      code = next;
      showValues(linkValues, [
        ["Service address", serviceAddress],
        ["Login memo", memo],
        ["Memo length", memoMeter(memo)],
        ["ZIP 321 URI", link.uri],
      ]);
      qr.innerHTML = link.qr_svg; // SVG from the demo crate: the QR code of the URI.
      showValues(replyValues, [
        ["Reply memo", `(ZFA OTP)${code}`],
        ["Code", code],
      ]);
      check();
      setStatus(status);
    } catch (error) {
      fail(error.message);
    }
  };

  sessionButton.addEventListener("click", () => {
    sessionInput.value = randomSession();
    update();
  });
  keyButton.addEventListener("click", () => {
    keyInput.value = randomHex(32);
    update();
  });
  for (const input of [sessionInput, addressInput, keyInput]) input.addEventListener("input", update);
  codeInput.addEventListener("input", check);

  sessionInput.value = randomSession();
  addressInput.value = userAddress;
  keyInput.value = randomHex(32);
  for (const el of [sessionInput, addressInput, keyInput, sessionButton, keyButton, codeInput]) el.disabled = false;
  update();
} catch (error) {
  setStatus(status, `the browser cannot load the WASM module (${error.message}).`);
}
