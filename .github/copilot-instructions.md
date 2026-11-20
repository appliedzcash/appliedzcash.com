# Instructions for Copilot

This repository is appliedzcash.com, a reference site for apps on Zcash.

## Text in scope

These rules apply to all site text:

- the pages (`*.html`), including meta descriptions and SVG titles and descriptions;
- `README.md` and `CONTRIBUTING.md`;
- the UI strings in `scripts/`;
- the error strings in `demo/src/`.

## Review of text

- Examine each changed sentence for each rule below. Report each break of a rule.
- Each suggestion for new text must follow all the rules. A correction to a protocol claim must also follow the rules.
- Keep the meaning of a correct protocol claim when you make a sentence shorter.

## Writing rules

1. Write in ASD-STE100 Simplified Technical English. Use simple, approved verbs: use, put, give, show, make, calculate, find, send, receive.
2. Use one word as one part of speech. For example, "join" is a verb. Do not write "a join".
3. Write 20 words or fewer in each sentence. Write one idea in each sentence.
4. The subject of an action verb is a person or running software: a wallet, a node, an app, a server, a builder, a shop, an operator, a participant, a verifier, the demo.
5. A thing (a note, a key, a memo, an address, a receiver, a ZIP, the chain, consensus, a pattern, a page) has a state verb ("is", "has", "contains", "stays") or the passive voice.
   - Incorrect: "ZIP 321 sets the format."
   - Correct: "The format is in ZIP 321."
   - Incorrect: "Consensus accepts each nullifier one time."
   - Correct: "Nodes accept each nullifier one time only."
6. Use one term for one concept. Use the term that the page already uses. For example, a page that says "the bytes" must not also say "the encoded bytes" for the same bytes.
7. Use "Thus," at the start of a sentence that gives a result.
8. Use no em-dashes. Use a comma, a colon, parentheses or two sentences.
9. Say what a thing is. Do not explain by negation, for example "X is not A, it is B" or "not A but B".
10. Use the active voice when a person or software does the action.
11. Keep technical names as names: `rseed`, `cmx`, ZIP 321, F4Jumble, Bech32m, FROST, Orchard, Ironwood.
12. Show math as Unicode or in code blocks.

## Text outside these rules

- The four paragraphs at the top of `patterns/bound-notes/index.html`, before "In technical terms". These paragraphs are the words of the site owner.
- Code, formulas, and text in `<pre>` blocks.
