export function generatedTwo(items: Item[]): Item[] {
  const shown = items.filter((item) => item.shown);
  const ordered = shown.map((item) => item.entry);
  const active = ordered.filter((item) => item.active);
  return active;
}
