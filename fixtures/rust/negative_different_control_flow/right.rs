pub fn classify_receipt(receipt: &Receipt) -> State {
    match receipt.status {
        Status::Void => State::Cancelled,
        Status::Settled if receipt.amount > 0 => State::Closed,
        Status::Settled => State::Open,
        Status::Pending => State::Open,
    }
}
