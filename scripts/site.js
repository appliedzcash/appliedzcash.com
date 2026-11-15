// Site chrome. The "Copy address" button in the footer:
for (const button of document.querySelectorAll("button.copy")) {
  button.addEventListener("click", async () => {
    await navigator.clipboard.writeText(button.dataset.copy);
    const label = button.textContent;
    button.textContent = "Copied";
    setTimeout(() => (button.textContent = label), 1500);
  });
}

// Heading anchors: a "#" link on each section heading that has an id.
for (const heading of document.querySelectorAll("main h2[id], main h3[id]")) {
  if (heading.closest(".demo")) continue;
  const anchor = document.createElement("a");
  anchor.className = "anchor";
  anchor.href = `#${heading.id}`;
  anchor.textContent = "#";
  anchor.setAttribute("aria-label", "Link to this section");
  heading.append(anchor);
}
