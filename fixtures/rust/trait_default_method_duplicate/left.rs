pub trait InvoiceFormatter {
    fn format_invoice(&self, invoice: &Invoice) -> Vec<Cell> {
        let lines = invoice.lines.iter().filter(|line| line.visible).collect::<Vec<_>>();
        let cells = lines.iter().map(|line| line.to_cell()).collect::<Vec<_>>();
        let active = cells.into_iter().filter(|cell| cell.enabled).collect::<Vec<_>>();
        active
    }
}
