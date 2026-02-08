use std::time::Instant;

#[derive(Clone)]
pub struct ValueEntry {
    pub value: String,
    pub expire_at: Option<Instant>,
}
