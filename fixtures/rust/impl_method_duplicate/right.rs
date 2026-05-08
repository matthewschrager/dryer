impl ReceiptReport {
    pub fn draw(&self, items: &[Receipt]) -> Vec<Entry> {
        let shown = items.iter().filter(|item| item.shown).collect::<Vec<_>>();
        let ordered = shown.iter().map(|item| item.to_entry()).collect::<Vec<_>>();
        let kept = ordered.into_iter().filter(|entry| entry.amount > 0).collect::<Vec<_>>();
        kept
    }
}
