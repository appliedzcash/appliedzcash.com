// Fills a <dl class="values"> with [name, value] rows. Each value shows as code.

export function showValues(list, rows) {
  list.replaceChildren(
    ...rows.flatMap(([name, value]) => {
      const dt = document.createElement("dt");
      dt.textContent = name;
      const dd = document.createElement("dd");
      const code = document.createElement("code");
      code.textContent = value;
      dd.append(code);
      return [dt, dd];
    }),
  );
}
