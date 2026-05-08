export function generatedOne(rows: Row[]): Row[] {
  const visible = rows.filter((row) => row.visible);
  const sorted = visible.map((row) => row.item);
  const kept = sorted.filter((row) => row.enabled);
  return kept;
}
