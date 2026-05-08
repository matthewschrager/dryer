export function OrderPanel({ order, onSelect }: Props) {
  const rows = order.lines.filter((line) => line.visible);
  const total = rows.reduce((sum, line) => sum + line.amount, 0);
  if (rows.length === 0) {
    return <section className="empty">No orders</section>;
  }
  return <section onClick={onSelect}>{total}</section>;
}
