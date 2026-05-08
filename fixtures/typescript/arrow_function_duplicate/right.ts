export const summarizeInvoices = (items: Invoice[]): Totals => {
  const closed = items.filter((item) => item.closed);
  const values = closed.map((item) => item.value);
  const size = values.length;
  const total = values.reduce((first, second) => first + second, 0);
  return { size, total };
};
