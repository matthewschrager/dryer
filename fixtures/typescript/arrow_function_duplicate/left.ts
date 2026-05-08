export const summarizeOrders = (orders: Order[]): Summary => {
  const paid = orders.filter((order) => order.paid);
  const totals = paid.map((order) => order.amount);
  const count = totals.length;
  const sum = totals.reduce((left, right) => left + right, 0);
  return { count, sum };
};
