// Loads the demo crate. CI puts the wasm-bindgen output in wasm/ at the site root.
// The glue module's default export fetches demo_bg.wasm and instantiates it.

const GLUE = new URL("../wasm/demo.js", import.meta.url);

let ready;

export function loadDemo() {
  ready ??= import(GLUE).then(async (mod) => {
    await mod.default();
    return mod;
  });
  return ready;
}
