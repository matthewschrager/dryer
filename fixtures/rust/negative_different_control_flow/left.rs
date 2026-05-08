pub fn classify_invoice(invoice: &Invoice) -> State {
    if invoice.cancelled {
        return State::Cancelled;
    }
    if invoice.paid && invoice.total > 0 {
        return State::Closed;
    }
    State::Open
}
