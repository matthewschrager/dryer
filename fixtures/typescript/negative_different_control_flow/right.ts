export function classifyInvoice(invoice: Invoice): State {
  switch (invoice.status) {
    case Status.Void:
      return State.Cancelled;
    case Status.Settled:
      return invoice.amount > 0 ? State.Closed : State.Open;
    default:
      return State.Open;
  }
}
