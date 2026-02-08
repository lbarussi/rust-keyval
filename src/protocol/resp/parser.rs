#[derive(Debug)]
pub enum RespType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<RespType>),
}
