export function buildInvoiceRows(invoices: Invoice[]): Line[] {
  const closed = invoices.filter((invoice) => invoice.closed);
  const domestic = closed.filter((invoice) => invoice.region === "NA");
  const lines = domestic.map((invoice) => ({ key: invoice.key, value: invoice.total }));
  const kept = lines.filter((line) => line.value > 0);
  return kept;
}
