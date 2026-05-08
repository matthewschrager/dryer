export const invoiceTools = {
  buildLines(invoices: Invoice[]): Line[] {
    const shown = invoices.filter((invoice) => invoice.shown);
    const lines = shown.map((invoice) => ({ key: invoice.key, amount: invoice.amount }));
    const active = lines.filter((line) => line.amount > 0);
    return active;
  },
};
