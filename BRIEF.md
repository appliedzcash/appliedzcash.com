# appliedzcash.com

Mark bought appliedzcash.com on Nov 14, 2026. The site is a community reference for building apps on Zcash: near-programmability with today's shielded protocol, and the patterns that make it work.

## Theme

Zcash has no smart contracts. Apps can still bind data and rules to shielded notes and let anyone verify them from chain data. The site teaches these patterns, one page each, with a live demo per pattern.

## Plan (decided with Mark, Nov 14)

| Decision | Choice |
|---|---|
| Audience | App builders first. Wallet teams and grant reviewers read the same pages. |
| Brand | Neutral and community. Shielded Labs gets author credit per page. |
| Shape | A pattern catalog. Each pattern page: problem, mechanism, cost, spec, demo, toy example. A "basics" section comes before the patterns: one page each, use, mechanism, limits and a live demo (Mark, Nov 15). |
| Examples | Fresh toy examples written for the site (a timestamp registry, a counter). Zodiac and ZcashNames stay out of the examples at launch. |
| Demos | One in-browser demo per pattern. |
| Specs | Mark reviews a ZIP draft, then the site hosts the cleared text. When a `zcash/zips` PR exists, the page links there. |
| Stack | Plain HTML, one CSS file, vanilla JS modules, like sras.io. Math as Unicode and code blocks. |
| Demo build | A Rust crate in `demo/`. GitHub Actions builds the WASM and places it beside the HTML. |
| Repo | A public repo in a new `appliedzcash` GitHub org. Anyone opens a PR; Mark merges. |
| Hosting | GitHub Pages from that repo. DNS changes wait for Mark. |

### Site map

```
/                                 Home: what near-programmability is, the basics list, the pattern list
/basics/concepts/                 Concepts: notes, pools, actions, memos, keys, addresses
/basics/payment-links/            Payment links: ZIP 321 URI and QR code
/basics/address-per-customer/     Address per customer: diversified addresses
/basics/watch-only-keys/          Watch-only keys: UFVK and UIVK for servers
/patterns/bound-notes/            Bound notes
/patterns/polite-actions/         Polite actions
/patterns/wallet-native-orders/   Wallet-native orders
/patterns/action-pairing/         Canonical action pairing
/patterns/threshold-keys/         Threshold keys: FROST n-of-m app roles
/specs/<name>/                    Cleared ZIP drafts, one per pattern
/wishlist/                        Later: primitives that would unlock more (ZIP 68, ZIP 231, ZSAs)
```

### Launch pages and demos

| Page | Source | Demo | Gate |
|---|---|---|---|
| Home | This brief | None | None |
| Bound notes | `zips/draft-bound-notes.md` | Derive `rseed` from kind, fields and predecessor; recompute `cmx` live (WASM, orchard) | Mark reviews the ZIP. Its ZcashNames credit waits on Julian's check. |
| Polite actions | ADR 5, generalized | Action-count calculator: scan cost per wallet per day (plain JS) | ADR 5 numbers restated with toy apps |
| Wallet-native orders | `zips/draft-shieldedlabs-wallet-native-orders.md` | Build a ZIP 321 URI with a memo order and show its QR code (WASM) | Mark reviews the ZIP |
| Canonical action pairing | `zips/draft-shieldedlabs-canonical-action-pairing.md` | Shuffle a bundle's actions by keyed sort and show the pairs (WASM) | Mark reviews the ZIP. Zodiac's `set_canonical_pairing` aligns with the ZIP first. |
| Threshold keys | Art market build: Art Title Notes (FROST 2-of-2 title key whose royalty side is a 2-of-3 group) and SRA on Bound Notes (one RedPallas group key, a public tweak per object so signers stay offline at registration) | Run a 2-of-3 RedPallas keygen and signing round in the browser; nest it inside a 2-of-2; tweak the group key per object (WASM, reddsa) | Mark clears the art market material (Sebastián too for SRA). The nested construction awaits a cryptographer's review, so the page labels it experimental. FROST on Ironwood is unaudited. |

### Repo layout

```
index.html
patterns/<name>/index.html
specs/<name>/index.html
styles/main.css
scripts/            site JS modules, demo loaders
demo/               Rust crate, one module per demo, built to WASM in CI
.github/workflows/  build demo, deploy Pages
CONTRIBUTING.md     how to propose a pattern
```

### Layers (each ends working)

1. **L0 skeleton:** home page, CSS, the Pages workflow with a stub WASM demo. It serves at the org's `github.io` URL.
2. **L1 bound notes:** the page, the cleared spec and the demo.
3. **L2 polite actions:** the page and the calculator.
4. **L3 wallet-native orders:** the page, the spec and the demo.
5. **L4 action pairing:** the page, the spec and the demo.
6. **L5 threshold keys:** the page and the demo.
7. **L6 launch:** Mark points appliedzcash.com at Pages, then announces.
8. **Basics:** four basics pages with demos, and the home page split into basics and patterns. One PR.

## Source material

| Topic | Where |
|---|---|
| Bound notes: data hashed into a zero-value note's `rseed`, so ZIP 212 derives ψ and rcm and standard wallets still decrypt the note | `zips/draft-bound-notes.md` in the Zodiac repo; X article draft (artifact "Bound Notes Article", https://claude.ai/artifact/MQDEGNCLnDxy7p7vVDf552); artifact "SRA on Bound Notes" (https://claude.ai/artifact/5kzFSdMMoV7qNiSZ7K5bdT) |
| Other ZIP drafts (Nov 3): wallet-native orders, canonical action pairing, decision markets | `zips/` in the Zodiac repo |
| Zodiac Markets: futarchy decision markets on Zcash testnet, verifiable checks V1 to V7, Lean-proved market math | `~/Projects/Zcash-Futarchy` (`README.md`, `DESIGN.md`, `BRIEF.md`, `docs/adr/`). The local checkout lags `origin/main`: ADR 4 and ADR 5 exist on `origin/main` only. |
| ZcashNames: the origin of the binding pattern (Julian, GitHub craftsoldier) | `/Users/mark/wiki/wiki/zcashnames.md`, `zns-verify.md` |
| Polite actions: every wallet trial-decrypts every action, so apps keep their action count low (ADR 5: one note per epoch) | `docs/adr/0005-one-note-per-epoch.md` |
| Builder support: `add_output_with_rseed` in orchard, now built from a fork, upstream PR pending | Zodiac `BRIEF.md`, craftsoldier/orchard#3 and #4 |
| FROST n-of-m: nested groups, tweaked per-object keys, member checks | `~/Projects/zcash-art-market` (local only, `DESIGN.md`); artifacts "Art Title Notes" (https://claude.ai/artifact/HtGCaeKbc12RrPr67qN1Ns) and "SRA on Bound Notes" (above); Zodiac DESIGN.md L5 (FROST 2-of-3 over PCZT, one group key per role) |
| ZIP 68: relative lock-time, Draft since 2016, transparent inputs only | `~/Projects/Zcash/zips/zips/zip-0068.rst` |
| Market and app landscape | `/Users/mark/wiki/wiki/zcash-app-opportunity-2026.md`, `zodiac-markets.md`, `openzcash.md` |
| Stack model | `~/Projects/Zcash/sras.io` |

## Constraints

- The Zodiac repo (ShieldedLabs/zodiac) is **private**. Mark clears each piece before it goes public. Record each clearance under Done.
- The Zodiac mainnet plan awaits team review. Keep it off the site until Mark clears it.
- The ZcashNames mention waits on Julian's check.
- The art market build and the SRA proposal are private. Mark clears them, and Sebastián co-owns SRA. The pilot estate's name stays off the site.
- Keep the round 6 testers' names private.
- Mark creates the GitHub org and changes DNS.
- Style: no em-dashes, no explanation by negation, short active sentences.
- Site copy (pages, `CONTRIBUTING.md`, demo UI text) is 100% ASD-STE100: dead-simple technical writing (Mark, Nov 14).

## Now

- [ ] **Start here: Mark reviews `draft-bound-notes.md` for public release.** Also fix its Motivation claim "a note commitment appears once" (consensus enforces uniqueness for nullifiers only; the site page was corrected in PR #2). Ask Julian about the ZcashNames credit. Then add `/specs/bound-notes/` and link it from the page's Spec section. That closes L1.
- [ ] Merge the L5 PR (threshold keys page and nested FROST demo, Experimental label). Mark cleared the art market's tweak and nesting recipe for publication on Nov 15. The page names neither the art market, the estate, Zodiac nor SRA.
- [ ] Mark turns on "Enforce HTTPS" for Pages (repo settings).
- [ ] Mark submits https://appliedzcash.com/sitemap.xml in Google Search Console.
- [ ] Mark picks licenses: code (MIT/Apache, like Zodiac) and page text (for example CC BY 4.0).

## Soon

- [ ] Mark reviews the wallet-native orders ZIP. Then add `/specs/wallet-native-orders/` (closes L3).
- [ ] Align Zodiac's `set_canonical_pairing` with its ZIP (Zodiac BRIEF, Soon). Mark reviews the ZIP. Then add `/specs/action-pairing/` (closes L4).
- [ ] A cryptographer reviews the nested FROST construction. Until then the L5 page keeps its Experimental label.

## Later

- [ ] L6: Mark announces the site.
- [ ] Vale in CI for STE: sentence length, passive voice, and a vocabulary from the ASD-STE100 approved-word list (free from ASD on request).
- [ ] A favicon and an og:image, so shared links show a picture.
- [ ] `/wishlist/`: ZIP 68, ZIP 231 memo bundles, ZSAs, and what each would unlock.
- [ ] Case studies: Zodiac Markets after Mark clears it; ZcashNames after Julian agrees.
- [ ] Link each spec page to its `zcash/zips` PR once one exists.
- [ ] Invite co-maintainers to the org (for example craftsoldier).

## Done

- [x] Nov 14: plan the site with Mark (this brief). Mark approved it as written.
- [x] Nov 14: Mark created the `appliedzcash` org. The empty public repo is https://github.com/appliedzcash/appliedzcash.com, set as `origin`. First commit `84538ee` pushed to `main`.
- [x] Nov 14: L0 skeleton. Home page, five pattern stubs, `styles/main.css` (light and dark), `scripts/wasm.js` loader, `demo/` crate (BLAKE2b-256 pipeline check, built to WASM), `pages.yml` workflow, `CONTRIBUTING.md` stub. Licenses still wait for Mark.
- [x] Nov 14: L0 merged (appliedzcash/appliedzcash.com#1) and deployed. appliedzcash.com already points at Pages, so the site is live. Mark is fine with that.
- [x] Nov 14: L1 page and demo merged (appliedzcash/appliedzcash.com#2). The page restates the mechanism and withholds the spec text; it names neither Zodiac nor ZcashNames. The demo is a timestamp registry on `orchard` 0.16 (both lead bytes) and replaces the home pipeline check. Copilot review fixed: cmx wording, fixed stamps, pool lock.
- [x] Nov 14: L2 merged (appliedzcash/appliedzcash.com#3) and live: polite-actions page and scan-cost calculator for a toy scoreboard. ADR 5's general facts appear without Zodiac's name. Copilot review fixed: memo-cost scope, the padded second action, row headers.
- [x] Nov 14: L3 merged (appliedzcash/appliedzcash.com#4) and live: wallet-native orders page in 100% STE and the toy-poll demo.
- [x] Nov 15: donation address (Mark's Noir wallet, Mainnet UA) and its QR code in every footer (appliedzcash/appliedzcash.com#5).
- [x] Nov 15: L4 merged (appliedzcash/appliedzcash.com#6) and live: canonical action pairing page in 100% STE and the rebuild demo (ZIP retry rule included). Copilot review fixed: retry rule, live result, CSS brace, mobile fields.
- [x] Nov 15: SEO merged (appliedzcash/appliedzcash.com#7): keyword-led titles and descriptions, canonical URLs, Open Graph and JSON-LD, `sitemap.xml` and `robots.txt`. Copilot's "demo for each pattern" comment declined, because the L5 demo comes next.
- [x] Nov 15: Basics built on branch `basics`: concepts, payment links, address per customer and watch-only keys, each with a WASM demo on Testnet keys. The demo shop seed is 32 bytes of 0x07. New deps `zip32` and `zip321`. `scripts/values.js` and `qr_svg` in the demo crate are now shared.
