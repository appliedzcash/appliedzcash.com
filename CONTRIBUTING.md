# Contributing

Anyone can propose a pattern or a change. Use a pull request (PR).

## PR flow

1. Fork the repository.
2. Make a branch for your change.
3. Add or change the pages. Use the template below for a new pattern.
4. Open a PR against `main`.
5. The Pages workflow builds the site for each PR. Make sure that the build is successful.
6. The maintainer reviews the PR and merges it.

## Pattern page template

Put each pattern in `patterns/<name>/index.html`. Each page has these sections, in this order:

1. **Problem:** the task that an app must do.
2. **Mechanism:** how an app does the task with the shielded protocol.
3. **Cost:** the fee, the scan time and the size of the pattern.
4. **Spec:** the normative text, or a link to the ZIP.
5. **Demo:** a live demo in the browser.
6. **Toy example:** a small app that uses the pattern, for example a timestamp registry or a counter.

## Writing rules

- Write in ASD-STE100 Simplified Technical English.
- Use short sentences. Use one idea in each sentence.
- Use the active voice when a person or software does the action.
- Use an action verb only when its subject is a person or software: a wallet, a node, an app, a server, a builder, a demo.
- For a thing (a note, a key, a memo, an address, a ZIP), use "is", "has", "contains" or the passive voice. Write "The format is in ZIP 321".
- Use one word for one concept.
- Show math as Unicode or in code blocks.

## Demos

Each demo is one module in the Rust crate in `demo/`. CI builds the crate to WASM and puts the output in `wasm/`. The browser loads the output with `scripts/wasm.js`.

Build the demo on your computer:

```bash
cd demo
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir ../wasm target/wasm32-unknown-unknown/release/demo.wasm
```

Use the `wasm-bindgen-cli` version that is in `demo/Cargo.lock`. Then start a local server at the repository root, for example `python3 -m http.server 8080`.

## Stack

Use plain HTML, the one stylesheet in `styles/main.css` and vanilla JavaScript modules. Do not add frameworks, npm packages, external fonts or analytics.

## Licenses

- Code (the `demo/` crate, `scripts/`, `styles/`, the workflows): 0BSD, in `LICENSE`.
- Page text and images: CC0 1.0, in `LICENSE-CC0`.
- A spec page has the license of its ZIP draft. The license name is in the draft header.
- The WASM build contains crates under their own licenses. CI writes their notices to `wasm/notices.html`.

You agree that your contribution has these licenses.
