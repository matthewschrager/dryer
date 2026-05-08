pub fn receipt_summary(rows: &[Receipt]) -> Totals {
    let closed = rows.iter().filter(|row| row.closed).collect::<Vec<_>>();
    let local = closed.iter().filter(|row| row.region == "NA").collect::<Vec<_>>();
    let amount = local.iter().map(|row| row.value).sum::<i64>();
    let held = local.iter().filter(|row| row.held).count();
    Totals { amount, held }
}
