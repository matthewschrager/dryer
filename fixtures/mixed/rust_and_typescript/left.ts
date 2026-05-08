export function buildOrderRows(orders: Order[]): Row[] {
  const visible = orders.filter((order) => order.visible);
  const rows = visible.map((order) => ({ id: order.id, total: order.total }));
  const kept = rows.filter((row) => row.total > 0);
  return kept;
}
