// Site chrome: the "Copy address" button in the footer.
for (const button of document.querySelectorAll("button.copy")) {
  button.addEventListener("click", async () => {
    await navigator.clipboard.writeText(button.dataset.copy);
    const label = button.textContent;
    button.textContent = "Copied";
    setTimeout(() => (button.textContent = label), 1500);
  });
}
