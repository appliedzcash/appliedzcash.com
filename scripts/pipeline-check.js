import { loadDemo } from "./wasm.js";

const input = document.getElementById("hash-input");
const output = document.querySelector("#hash-output code");
const status = document.getElementById("hash-status");

try {
  const demo = await loadDemo();
  const update = () => {
    output.textContent = demo.hash(input.value);
  };
  input.addEventListener("input", update);
  input.disabled = false;
  status.textContent = "The WASM module is loaded.";
  update();
} catch (error) {
  status.textContent = `Error: the WASM module failed to load (${error.message}).`;
}
