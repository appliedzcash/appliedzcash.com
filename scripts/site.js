// Site chrome.

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

// "On this page" rail for long pages (10 headings or more). CSS shows it on wide screens only.
const headings = [...document.querySelectorAll("main h2[id], main h3[id]")].filter((h) => !h.closest(".demo"));
if (headings.length >= 10) {
  const nav = document.createElement("nav");
  nav.className = "toc";
  nav.setAttribute("aria-label", "On this page");
  const title = document.createElement("p");
  title.textContent = "On this page";
  const list = document.createElement("ul");
  const links = new Map();
  for (const heading of headings) {
    const item = document.createElement("li");
    if (heading.tagName === "H3") item.className = "sub";
    const link = document.createElement("a");
    link.href = `#${heading.id}`;
    link.textContent = [...heading.childNodes]
      .filter((node) => !node.classList?.contains("anchor"))
      .map((node) => node.textContent)
      .join("")
      .trim();
    item.append(link);
    list.append(item);
    links.set(heading, link);
  }
  nav.append(title, list);
  document.querySelector("main").append(nav);

  // The current section: the last heading above the top third of the screen.
  const observer = new IntersectionObserver(
    () => {
      const current = headings.findLast((h) => h.getBoundingClientRect().top < window.innerHeight / 3) ?? headings[0];
      for (const [heading, link] of links) {
        if (heading === current) link.setAttribute("aria-current", "true");
        else link.removeAttribute("aria-current");
      }
    },
    { rootMargin: "0px 0px -66% 0px" },
  );
  for (const heading of headings) observer.observe(heading);
}
