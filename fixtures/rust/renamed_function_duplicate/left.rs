pub fn invoice_summary(orders: &[Order]) -> Summary {
    let paid = orders.iter().filter(|order| order.paid).collect::<Vec<_>>();
    let domestic = paid.iter().filter(|order| order.country == "US").collect::<Vec<_>>();
    let total = domestic.iter().map(|order| order.amount).sum::<i64>();
    let flagged = domestic.iter().filter(|order| order.flagged).count();
    Summary { total, flagged }
}
