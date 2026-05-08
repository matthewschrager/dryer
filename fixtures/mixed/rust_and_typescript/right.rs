pub fn build_invoice_rows(invoices: &[Invoice]) -> Vec<Line> {
    let shown = invoices.iter().filter(|invoice| invoice.shown).collect::<Vec<_>>();
    let lines = shown.iter().map(|invoice| invoice.to_line()).collect::<Vec<_>>();
    let active = lines.into_iter().filter(|line| line.amount > 0).collect::<Vec<_>>();
    active
}
