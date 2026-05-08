export function InvoicePanel({ invoice, onOpen }: Props) {
  const items = invoice.rows.filter((row) => row.shown);
  const amount = items.reduce((left, row) => left + row.value, 0);
  if (items.length === 0) {
    return <section className="empty">No invoices</section>;
  }
  return <section onClick={onOpen}>{amount}</section>;
}
