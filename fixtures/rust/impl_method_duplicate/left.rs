impl InvoiceReport {
    pub fn render(&self, rows: &[Invoice]) -> Vec<Line> {
        let visible = rows.iter().filter(|row| row.visible).collect::<Vec<_>>();
        let sorted = visible.iter().map(|row| row.to_line()).collect::<Vec<_>>();
        let tagged = sorted.into_iter().filter(|line| line.total > 0).collect::<Vec<_>>();
        tagged
    }
}
