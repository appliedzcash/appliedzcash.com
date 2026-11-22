# appliedzcash.com

Mark bought appliedzcash.com on Nov 14, 2026. The site is a community reference for building apps on Zcash: near-programmability with today's shielded protocol, and the patterns that make it work.

## Theme

Zcash has no smart contracts. Apps can still bind data and rules to shielded notes and let anyone verify them from chain data. The site teaches these patterns, one page each, with a live demo per pattern.

### Bound notes copy (Mark, Nov 17)

Lay explanation, at the top of the bound notes page. The home hero stays general: it covers all patterns (Mark, Nov 20).

> Zcash keeps money in sealed notes. Each note has a seal that only its contents can make.
>
> A bound note puts an app's data into that seal. If someone changes the data, the seal stops matching. Everyone can see the change.
>
> Each new note also includes the seal of the note before it. To make the new note, the app must use up the old one. Zcash lets a note be used one time only.
>
> So the app's record is a single chain. Each link is sealed, and each link can come only once. Anyone with the data can check the whole chain.

Slide line: "Each record is sealed into Zcash, and Zcash allows only one next record."

Technical framing: `rseed` = H(domain ‖ kind ‖ fields ‖ predecessor `cmx`), and ZIP 212 derives ψ and rcm from it. The hash binds the memo's data to the commitment. The binding domain keeps each app's notes separate. The chain, plus spending the predecessor, gives one history that consensus enforces. Signed memos give authenticity; the single history comes from bound notes alone.

Limits the page states: about 505 data bytes per memo, one more action per extra chunk. Verification is recomputation from the revealed fields. A reveal exposes that note, so guessable data needs a secret salt. The app enforces `cmx` uniqueness (consensus enforces nullifiers). Builders need `add_output_with_rseed` (craftsoldier/orchard); any standard wallet decrypts the notes.

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
| Repo | A public repo in a new `appliedzcash` GitHub org. Anyone opens a PR; Mark merges. Copilot review is by request only (Nov 20): request it on a PR that changes demo code (Rust, WASM) or a protocol claim. Copilot reads the writing rules from `.github/copilot-instructions.md` (it reads the first 4,000 characters only). |
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
/patterns/log-in-with-zcash/      Log in with Zcash: a one-time code in a memo (ZcashMe)
/patterns/verified-address/       Verified address: a login record in a bound note (experimental)
/specs/<name>/                    Cleared ZIP drafts, one per pattern
/wishlist/signature-login/        Wishlist: designs that need new software. Signature login is page 1. Later: ZIP 68, ZIP 231, ZSAs
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
- Actors (Mark, Nov 20): the subject of an action verb is a person or deployed software (a wallet, a node, an app, a server, a builder, the demo). A thing (a note, a key, a memo, a ZIP, the chain) has a state verb or the passive. The rule is in `CONTRIBUTING.md`.

## Now

- [ ] **Start here: Mark posts the site to X (L6).** Credit Daniel as "Daniel Gorgonha (DeegaLabs)": X @danielgorgonha1, Discord @danielgorgonha1 (his request, Nov 20).
- [ ] Compare the Pages build time of a PR run before and after #27 (merged Nov 20: `Swatinem/rust-cache` v2.9.3, pinned by commit, `workspaces: demo`). The first run on `main` fills the cache.
- [ ] Mark merges #47: a third design on the signature login page, the identity key signature (zafu's zid: Ed25519 for each origin, from a hash of the seed phrase, apart from the BIP 39 seed). Source: a reply on X (@patchwinner, Nov 22) that login keys must be apart from spend keys. The page credits the author as "hitch (hitchho)" (their answer by DM): zid is their work, in zafu, a wallet based on Penumbra Labs' Prax. GitHub hitchho, forum @hitch, X @patchwinner. New title: "log in with a wallet key". Copilot review requested.
- [ ] Mark sends two pages to c-node for a review: signature login (after #47; it has the protocol claims of #42 and #47) and verified address (#38, merged without the review).
- [ ] Julian checks the log in with Zcash page (#37, merged). Get Zechariah's surname or handle for the credit (the page and the home page have the first name only).
- [ ] Mark decides: move his four lay paragraphs on bound notes to strict STE, or keep his wording. #31 keeps it.
- [ ] Mark requests indexing in Search Console for each page, `/basics/smart-contracts/` first. Mark adds the site to Bing Webmaster Tools (import from Search Console; Bing results also show in ChatGPT search and DuckDuckGo).

## Soon

- [ ] Mark reviews the wallet-native orders ZIP. Then add `/specs/wallet-native-orders/` (closes L3).
- [ ] A cryptographer reviews the nested FROST construction and the owner-only `sk` model (section 4). Until then the L5 page keeps its Experimental label.
- [ ] Add Daniel's list of FROST implementations to the threshold keys page (#40, item 4, posted Nov 22): 9 entries in three groups, with his criterion (license, release or crate, a push in the last 60 days, no hackathon builds). Then close #40.
- [ ] SEO backlinks (Mark approves each): a PR to the ZecHub developer resources page (zechub.wiki/start-here/developer-resources), a PR to the Developer Tools page of zcash.readthedocs.io, a request to zecdev to list the site, and a post in the Zcash Community Forum.
- [ ] GitHub repo: set the description, the homepage URL and topics (zcash, zero-knowledge, privacy).


## Later

- [ ] Vale in CI for STE: sentence length, passive voice, and a vocabulary from the ASD-STE100 approved-word list (free from ASD on request).
- [ ] Log in with Zcash, a verifiable record (Mark's question, Nov 21): after the code check, the worker writes a bound note with the session ID and a salted hash of the address. An open idea, absent from the page.
- [ ] More wishlist pages: ZIP 68, ZIP 231 memo bundles, ZSAs, and what each would unlock. Each page uses the wishlist template in `CONTRIBUTING.md`.
- [ ] Case studies: Zodiac Markets after Mark clears it; ZcashNames after Julian agrees.
- [ ] Link each spec page to its `zcash/zips` PR once one exists.
- [ ] Invite co-maintainers to the org (for example craftsoldier).
- [ ] SEO pages: hub pages at `/basics/`, `/patterns/` and `/wishlist/` (each URL gives 404 today), then breadcrumbs in the JSON-LD. Maybe "smart contracts" in the home title.
- [ ] Mark decides: a cookieless counter (for example GoatCounter) for referral data from X and the forum. Search Console has search clicks only. It is a privacy trade-off on a Zcash site.

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
- [x] Nov 15: L5 merged (appliedzcash/appliedzcash.com#8) and live: threshold keys page in 100% STE with an Experimental label, and the nested FROST demo (2-of-3 group in a 2-of-2 key, per-object tweak, hash-then-reveal commitments; reddsa 0.5.2 on frost-core 3.0.0). Mark cleared the art market's tweak and nesting recipe; the page names neither the art market, the estate, Zodiac nor SRA. All five pattern pages are live.
- [x] Nov 15: HTTPS fixed. GitHub had issued no certificate, so https://appliedzcash.com served a *.github.io certificate. Removing and re-adding the custom domain (Pages API, Mark's OK) got a Let's Encrypt certificate for appliedzcash.com and www (expires Feb 13, 2027). Enforce HTTPS is on, and HTTP redirects to HTTPS.
- [x] Nov 15: licenses chosen by Mark (most permissive): code 0BSD (`LICENSE`), page text and images CC0 1.0 (`LICENSE-CC0`). Spec pages keep their ZIP draft's license (MIT). CI writes the WASM's third-party notices to `wasm/notices.html` with cargo-about 0.9.3. Merged in appliedzcash/appliedzcash.com#10 and live.
- [x] Nov 15: Mark submitted https://appliedzcash.com/sitemap.xml in Google Search Console.
- [x] Nov 15: Mark reviewed the bound-notes draft in a local preview and decided it stays off this site; he handles the ZIPs on the Zodiac end. ShieldedLabs/zodiac#88 trims all four drafts (bound notes 467 to 290 lines with test vectors; pairing, decision markets and wallet orders also moved to STE).
- [x] Nov 15: `set_canonical_pairing` in the pinned orchard fork (aphelionz/orchard `2e69445`) checked against the pairing ZIP rule by rule: it matches.
- [x] Nov 15: Mark merged ShieldedLabs/zodiac#88: all four ZIP drafts trimmed, three moved to STE; decision markets gained an advisory report window D (VOID without a decision) and a fee allowance F at m x ZIP 317. Open source problems are listed in the PR description.
- [x] Nov 15: design PR 1, foundations: paper and ink color tokens, an accent focus ring, a skip link, a serif display face (Source Serif 4) and a mono face (JetBrains Mono), both self-hosted under the OFL in `fonts/` (91 KB).
- [x] Nov 15: design PR 2, identity and chrome: the sealed-note mark (Mark picked option A of four), `images/mark.svg` favicon, apple-touch-icon, one og card for all pages (`images/og.png`, large summary card), theme-color, a frosted sticky header with a GitHub link, and a compact donate card with a copy button (`scripts/site.js`).
- [x] Nov 15: design PR 3, reading: formula blocks, byte rulers (bound notes, polite actions), captioned demo parameters, typed notices (warning, caution, note), ids and hover anchors on all headings, step circles on procedure lists, and Cost as a card grid. The fact strip and section numbers were left out.
- [x] Nov 15: design PR 4, demos: a demo header with a status pill (`setStatus` in `scripts/values.js`, used by all nine WASM demos), hex values in groups of 8 with a color fingerprint and a flash on changed groups, verdict cards with an icon, stamp chain cards joined by "prev = cmx", a memo meter that counts UTF-8 bytes (it counted characters before), the QR on a white card, and bars plus a large ratio on the polite-actions calculator. Deferred: a grouped hex view inside the action-pairing table.
- [x] Nov 15: design PR 5, home and navigation: hero buttons (Start with the basics, See the patterns), numbered card grids with a whole-card link and an Experimental tag on threshold keys, a prev/next pager across all nine pages in home order, and an edit-on-GitHub link. The threshold card now says "t of n" like its page.
- [x] Nov 15: design PR 6, figures: the six ASCII diagrams are inline SVG figures (vertical flow, one SVG at all widths, STE title and description, site color tokens). Mark approved the bound-notes sample first. The home page gained a small bind, send and verify figure under the hero buttons.
- [x] Nov 15: all six design PRs live (#11 to #16). Fix #17: the Pages workflow copies a fixed folder list and left out `fonts/`, so the live site showed fallback fonts from #11 until #17. A new top-level folder must be added to the `cp` line in `pages.yml`.
- [x] Nov 15: design hygiene: STE fixes in the new UI strings ("Wait" for the loading pill, "Demo", "Change this page on GitHub"), a `404.html` with absolute paths, forced-colors and print styles, and `tools/check_chrome.py` as the first CI step (same header, footer and head links on every page; the deploy copies each linked folder).
- [x] Nov 15: "On this page" rail on pages with 10 or more headings (orders, pairing), at 72rem and wider, built by `scripts/site.js`.
- [x] Nov 15: action-pairing table: the seed uses the shared grouped hex view, each nf and cmx has a color swatch, and a cmx that changed since the last build flashes (all four on a second standard build, none on a canonical one).
- [x] Nov 15: pattern glyphs on the home cards (Mark approved the mockup): a chain of notes, a short row of actions, a memo note, a joined pair, and three signers with two sealed. The seal parts use the amber seal color, and the system highlight color in forced colors.
- [x] Nov 18: the donation footer is removed (#22).
- [x] Nov 20: first outside contributor, Daniel Gorgonha (danielgorgonha). Merged: the 25-second block time since NU7 on polite actions (#23) and the ZIP 2009 fee for two actions (#24).
- [x] Nov 20: launch polish (#26), merged. The home hero is general: new lede (parts of a payment, three app functions), a figure that maps note, memo and key to record, order and shared role, and general meta descriptions. The bound notes page starts with Mark's lay copy, then "In technical terms". The home page has a Contributors section: Mark, Daniel, and Julian (craftsoldier) as the source of the bound notes pattern (Mark's call, Nov 20; ZcashNames stays unnamed). `images/og.png` has the new figure; its source is `tools/og.svg` (render with static TTF copies of the two site fonts, `PANGOCAIRO_BACKEND=fc`).
- [x] Nov 20: Mark merged #25 (Daniel Gorgonha): verifier rules for bound note chains. The claims match orchard 0.16.0: `circuit.rs:358` (zero-value spends skip the anchor check), `builder.rs:415` and the builder shuffle.
- [x] Nov 20: Mark merged #28: the Signal group invite for builders. The home page has a third hero button, "Go to the Signal group", and the footer of all 11 pages has one sentence with the link. The button text uses "Go to" because "join" is doubtful as an STE verb.
- [x] Nov 20: Mark merged #29: the three hero buttons are on one line at desktop width (`.button` side padding 16 px, was 24 px).
- [x] Nov 20: Mark merged #30: the concepts lede is one sentence, Pools has Ironwood and transparent only (Sapling and Orchard in a footnote, class `footnote`), and the Yes and No cells of the concepts key table have a green or red tint (`.capability-table .yes` and `.no`). The same tint for the watch-only keys table came after the merge, so it is in #32.
- [x] Nov 20: #31 opened: all site copy follows the actor rule (10 pages, `CONTRIBUTING.md`, UI strings in `scripts/` and `demo/src/`). The spec sentence on three pattern pages is now "The draft will be published after review".
- [x] Nov 20: #32 opened: the concepts demo is a unified address builder (Mark's call: one demo on the concepts page, no receivers page). `construct` in `demo/src/inspect.rs` leaves out the receivers that are off and returns the items, the padding, the F4Jumble bytes and the Bech32m string. A test compares the F4Jumble bytes with the Bech32m payload of the library. The first example is a ZIP 316 test vector with a transparent, a Sapling and an Orchard receiver (zcash-test-vectors `unified_address.py`). Receiver colors are `--rc-transparent`, `--rc-sapling` and `--rc-orchard`. New deps: `f4jumble`, and `bech32` for tests. A local WASM build needs `wasm-bindgen-cli` 0.2.130, which is not installed on this Mac outside a scratch folder.
- [x] Nov 20: Mark merged #32 into `ste-strict`, with Copilot fixes to five protocol claims. Three of the fixes broke STE ("you can see changes throughout the encoded bytes", "a join", a 25-word sentence). The claims stay, in STE. The writing rules moved from `CONTRIBUTING.md` to `.github/copilot-instructions.md`, so Copilot review uses them; `CONTRIBUTING.md` links there.
- [x] Nov 20: Mark squash-merged #31 into `main`. #32 and e9705fd reached `ste-strict` after that merge, so #34 restores them. #34 also puts `README.md` in Copilot's text scope.
- [x] Nov 20: #33 opened: `README.md` in the Standard Readme format (RichardLitt/standard-readme), in STE. It points to `CONTRIBUTING.md` for the writing rules.
- [x] Nov 20: Mark merged #33 and #34. The repo has "Automatically delete head branches" on, so GitHub deletes a PR's branch at merge and moves a stacked PR's base to `main`. Clean-up: all stale branches (remote and local), both old worktrees and 1.3 GB of `demo/target` are gone. After a merge, start new work from `main`.
- [x] Nov 21: Julian agreed to add the OTP login flow of ZcashMe (source: zcashme/zcash-verification-service, MIT; the OTP idea is Zechariah's). Mark chose pattern page 6, named credits, and a demo with the login link and the code. Built on branch `log-in-with-zcash`: `/patterns/log-in-with-zcash/`, `scripts/log-in-with-zcash.js` (HMAC-SHA256 with Web Crypto; URI and QR code from the existing `payment_link` WASM, so the demo crate is unchanged), a sixth home card, a Contributors credit, the pager and the sitemap. `memoMeter` moved to `scripts/values.js`. The demo matches the test vector in `src/otp.rs` (880279). Local preview: copy the changed files to `/tmp/claude-501/site` (it has the WASM of Nov 20), then start `site` from `.claude/launch.json`.
- [x] Nov 21: #37 opened: the log in with Zcash page. Mark then asked for a second pattern for a durable record. Built on branch `verified-address`: `/patterns/verified-address/` (Experimental), `demo/src/verified_address.rs` (`verified_record`: claim = BLAKE2b-256("VerifiedAddr_clm", salt ‖ address), binding domain "VerifiedAddr_v1_", kind 1) and `scripts/verified-address.js` (a toy directory and a receipt check). `bound_note` in `demo/src/bound_notes.rs` is now shared by both demos. The reasoning: a login is a fact between two parties and needs no record; a record is for deadlines, a second app and audits. A local WASM build works offline: `wasm-bindgen` 0.2.130 is in the scratch folder of session `aac9f8b0` under `/private/tmp/claude-501/`.
- [x] Nov 21: #37, Copilot review: six fixes, each checked against `src/wallet/actor.rs` of the worker. The reply memo has the 6 digits only (the `(ZFA OTP)` prefix is in the README only). The reply amount is 0.0005 ZEC and the minimum login amount is 0.002 ZEC. The worker makes one reply transaction at most for each login payment and sends the same transaction again after a failure. The page also states that the service knows each code.
- [x] Nov 21: Mark merged #37 (squash). The base of #38 moved to `main`, and `main` was merged into `verified-address` with conflicts in four files; each conflict kept the lines of both sides.
- [x] Nov 21: #38, Copilot review: seven fixes. The page has a completeness check for audits (a checkpoint height, and the nullifier of the last listed record must be absent from the chain). A token needs no online service, the address is trimmed before the hash, and the salt is necessary only for the address check. The demo clears a failed receipt check. The main checkout was on `og-cards` (other work), so this fix was made in the worktree `.claude/worktrees/verified-address`.
- [x] Nov 21: one social card for each basics page and each pattern page (branch `og-cards`). `tools/og_cards.py` makes `images/og/<page folder>.png` from the kicker and the `og:title` of each page; it makes the static fonts from `fonts/` itself. The home page keeps `images/og.png`. All 11 pages have `twitter:image:alt`. `tools/check_chrome.py` fails when the card file of a page is absent. A new page needs one run of the tool. Note: a quote post on X shows the quoted post in place of the link card.
- [x] Nov 21: Mark merged the social cards (#41). `main` was merged into `verified-address` (one conflict, in this file). The verified address page has its card, `images/og/verified-address.png`, and the card meta tags. The sandbox protects the git data of a worktree that a session made by hand, so this merge was made in a scratch clone and the worktree `.claude/worktrees/verified-address` was removed.
- [x] Nov 21: "Alternative: a signature login" section on the log in with Zcash page (#42, merged). Source: a proposal by c-node (@colludingnode) on X, in reply to the pattern. The wallet makes one RedDSA key for each app (`rk = ak + [α]G`, α from a wallet secret and the app domain) and signs an ERC-4361-style message. The section has the keys, the sequence, a comparison table (`.compare-table`, new in `main.css`), what the signature shows, and the parts of a spec. It has a Proposal notice, because a spec and wallet support are absent. No demo. Later the same day: a second design from Julian, the viewing key signature (a Schnorr signature with `ivk` under `g_d` and `pk_d`, so the address is the account). The comparison has three rows: code in a memo, viewing key signature, app key signature. Julian's other idea (the wallet decrypts a login packet from the app) is absent from the page: it needs its own KDF personalization, or an app can use the wallet to decrypt a real note.
- [x] Nov 21: Wishlist, the third section of the site (name from the Signal poll; Mark's call). It is for designs that need new software: a wallet function, a ZIP or a consensus change. Page 1 is `/wishlist/signature-login/`, moved from the #42 section; the log in with Zcash page keeps a short pointer at the same anchor (`#signature-login`) and a Next link. A wishlist page has: necessary changes (improves, wallets, ZIP, consensus, status), problem, mechanism, comparison, parts of a spec. No demo. Also changed: a Wishlist link in the header of all 13 pages, the home section, the sitemap, `wishlist` in the `cp` line of `pages.yml`, and wishlist pages in `tools/og_cards.py` and `tools/check_chrome.py`. On a phone (32rem and narrower), the header hides the GitHub link, because four links are wider than the screen; the footer has the GitHub link. The local preview keeps old pages in the browser cache: add a query string to the URL.
- [x] Nov 21: Mark merged #44: the Signal group is easier to find (an X builder missed the hero button). Each header has a Signal link with the logo (logo only below 384px; Wishlist is hidden on phones), the hero button is on its own line ("Join the builders on Signal"), and the 11 basics, pattern and wishlist pages have an Ask box before the pager. Logo: `images/signal.svg` from Simple Icons (CC0).
- [x] Nov 22: Mark merged #46, threshold keys after Daniel's review in #40 (items 1, 2, 3 and 5; Daniel approved). Step 1 cites ZIP 2005 for `nk`, `qsk`, `qk` and `rivk`. Section 4, "The owner keeps sk", has the owner-only `sk` model with Daniel's conditions and a Caution notice (different from the ZIP 2005 text). Limits has Backup and Quantum items. Sign rule: when the sign bit is 1, the members negate their shares, and the coordinator negates the public term and each verifying share (as `into_even_y` in reddsa 0.5.2). `rivk` uses the x coordinate of `ak`, so the sign has no effect on it. The local `~/Projects/Zcash/zips` copy (April) has older wording in ZIP 2005 § 4.2.3, so check ZIP claims against `zcash/zips` main.
- [x] Nov 22: SEO indexes (#48, merged). `tools/site_index.py` writes `sitemap.xml` (with `lastmod` from the last commit of each page) and `llms.txt` (an index for AI assistants: the og:title and description of each page) at deploy time. The sections and the page order come from the home page, so a new page needs no sitemap edit. `sitemap.xml` is gone from the repo; the checkout has the full history. The JSON-LD of each page has a publisher (logo: `images/apple-touch-icon.png`, sameAs the GitHub org), the CC0 license, and Zcash as `about` (sameAs Wikipedia); each article also has its social card as `image`.
- [x] Nov 22: Smart contracts on Zcash page (#49, merged, live). `/basics/smart-contracts/` is for the search "smart contracts on Zcash": the short answer, a table of contract tasks (the Ethereum form and the Zcash pattern), who checks the rules, and the Draft ZIPs (226, 227, 228, 68, 231; checked on `zcash/zips` main). It is the first basics page, with no demo. The home hero, the home basics list and the Concepts pager link to it. Copilot found four protocol errors, all fixed: a deposit address finds the customer (the payer can be someone else); the fee is in ZIP 317 and polite actions keep the scan work small; one next record needs the next note in the action in which the old note is spent, plus the block order check; ZIP 68 is relative lock-time only.
- [x] Nov 22: the Rich Results Test on the live bound notes page found one valid Article, with one optional field absent: `author`. Mark's call: the Applied Zcash organization is the author of each article (#50).
