#[derive(Debug, Clone)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    Bulk(Option<Vec<u8>>),
    Array(Vec<RespValue>),
}

impl RespValue {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            RespValue::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RespValue::Error(s) => format!("-{}\r\n", s).into_bytes(),
            RespValue::Integer(n) => format!(":{}\r\n", n).into_bytes(),
            RespValue::Bulk(None) => b"$-1\r\n".to_vec(),
            RespValue::Bulk(Some(b)) => {
                let mut out = Vec::new();
                out.extend_from_slice(format!("${}\r\n", b.len()).as_bytes());
                out.extend_from_slice(b);
                out.extend_from_slice(b"\r\n");
                out
            }
            RespValue::Array(items) => {
                let mut out = Vec::new();
                out.extend_from_slice(format!("*{}\r\n", items.len()).as_bytes());
                for it in items {
                    out.extend_from_slice(&it.to_bytes());
                }
                out
            }
        }
    }
}
