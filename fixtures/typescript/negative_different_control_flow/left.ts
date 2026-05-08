export function classifyOrder(order: Order): State {
  if (order.cancelled) {
    return State.Cancelled;
  }
  if (order.paid && order.total > 0) {
    return State.Closed;
  }
  return State.Open;
}
