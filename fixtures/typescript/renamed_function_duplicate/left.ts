export function buildOrderRows(orders: Order[]): Row[] {
  const paid = orders.filter((order) => order.paid);
  const local = paid.filter((order) => order.country === "US");
  const rows = local.map((order) => ({ id: order.id, total: order.amount }));
  const flagged = rows.filter((row) => row.total > 0);
  return flagged;
}
