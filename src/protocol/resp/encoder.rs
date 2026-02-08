pub fn simple_string(msg: &str) -> String {
    format!("+{}\r\n", msg)
}

pub fn bulk_string(msg: &str) -> String {
    format!("${}\r\n{}\r\n", msg.len(), msg)
}

pub fn null_bulk() -> String {
    "$-1\r\n".into()
}
