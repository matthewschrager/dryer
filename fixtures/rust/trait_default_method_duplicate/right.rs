pub trait ReceiptFormatter {
    fn format_receipt(&self, receipt: &Receipt) -> Vec<Item> {
        let rows = receipt.rows.iter().filter(|row| row.shown).collect::<Vec<_>>();
        let items = rows.iter().map(|row| row.to_item()).collect::<Vec<_>>();
        let kept = items.into_iter().filter(|item| item.active).collect::<Vec<_>>();
        kept
    }
}
