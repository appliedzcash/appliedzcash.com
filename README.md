# appliedzcash.com

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Pages](https://github.com/appliedzcash/appliedzcash.com/actions/workflows/pages.yml/badge.svg)](https://github.com/appliedzcash/appliedzcash.com/actions/workflows/pages.yml)

Patterns for Zcash developers: build apps from shielded notes, memos and keys.

This repository contains the source of [appliedzcash.com](https://appliedzcash.com). The site has two sections:

- **Basics:** concepts, payment links, address per customer, and watch-only keys.
- **Patterns:** bound notes, polite actions, wallet-native orders, canonical action pairing, and threshold keys.

Each pattern is possible with the consensus rules of today. Each page has a live demo in the browser.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Maintainers](#maintainers)
- [Thanks](#thanks)
- [Contributing](#contributing)
- [License](#license)

## Background

An app on Zcash can put its data and rules on shielded notes. Anyone can then verify the result from chain data. The patterns for this work are on the site, one page for each pattern.

Each pattern page has six sections: problem, mechanism, cost, spec, demo, and toy example.

The site is plain HTML, one stylesheet (`styles/main.css`), and vanilla JavaScript modules. The demos are one Rust crate in `demo/`, with one module for each demo. CI builds the crate to WASM.

```
index.html            home page
basics/<name>/        basics pages
patterns/<name>/      pattern pages
styles/main.css       the stylesheet
scripts/              site modules and demo loaders
demo/                 Rust crate for the demos
tools/                CI check and the source of the Open Graph image
```

## Install

You must have:

- Rust, with the `wasm32-unknown-unknown` target;
- `wasm-bindgen-cli`, at the `wasm-bindgen` version in `demo/Cargo.lock`;
- Python 3.

```bash
git clone https://github.com/appliedzcash/appliedzcash.com.git
cd appliedzcash.com
rustup target add wasm32-unknown-unknown
cd demo
cargo install wasm-bindgen-cli --locked --version "$(cargo pkgid wasm-bindgen | sed 's/.*[@#]//')"
```

## Usage

Build the demos to WASM, then start a local server at the repository root:

```bash
cd demo
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir ../wasm target/wasm32-unknown-unknown/release/demo.wasm
cd ..
python3 -m http.server 8080
```

Open http://localhost:8080.

Do the CI checks on your computer:

```bash
python3 tools/check_chrome.py
cd demo && cargo test --locked
```

The `check_chrome.py` script compares the header, the footer and the shared head links on all pages.

The Pages workflow (`.github/workflows/pages.yml`) builds the site for each pull request. After a merge to `main`, the workflow sends the site to GitHub Pages. The workflow copies a fixed list of top-level folders. Thus, you must add each new top-level folder to the `cp` line in the workflow.

## Maintainers

[@aphelionz](https://github.com/aphelionz) (Mark Henderson).

## Thanks

- [Daniel Gorgonha](https://github.com/danielgorgonha), contributor.
- [Julian (craftsoldier)](https://github.com/craftsoldier), the source of the bound notes pattern.

## Contributing

Pull requests are accepted. Read [CONTRIBUTING.md](CONTRIBUTING.md) first. It contains the page template and the writing rules for all site text.

Ask questions in the [Signal group for builders](https://signal.group/#CjQKIGlpdEugkIjUIm3WGa_6V2Xhi6zEvJGB9UADaU3zNKGCEhCSrRufVMaAncHsqeeBZj_U).

## License

- Code: [0BSD](LICENSE) © 2026 the Applied Zcash contributors.
- Page text and images: [CC0 1.0](LICENSE-CC0).
- The WASM build contains crates under their own licenses. CI writes their notices to `wasm/notices.html`.
