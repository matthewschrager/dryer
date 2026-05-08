pub fn build_order_rows(orders: &[Order]) -> Vec<Row> {
    let visible = orders.iter().filter(|order| order.visible).collect::<Vec<_>>();
    let rows = visible.iter().map(|order| order.to_row()).collect::<Vec<_>>();
    let kept = rows.into_iter().filter(|row| row.total > 0).collect::<Vec<_>>();
    kept
}
