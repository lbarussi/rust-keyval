use crate::db::storage::Db;
use crate::commands;
use crate::protocol::resp::encoder::RespValue;

pub async fn process_parts(parts: Vec<String>, db: &Db) -> RespValue {
    if parts.is_empty() {
        return RespValue::Error("ERR empty command".into());
    }

    match parts[0].to_uppercase().as_str() {
        "PING" => RespValue::SimpleString("PONG".into()),
        "SET" => commands::set::execute(parts, db).await,
        "GET" => commands::get::execute(parts, db).await,
        "INCR" => commands::incr::execute(parts, db).await,
        "DEL" => commands::del::execute(parts, db).await,
        "EXISTS" => commands::exists::execute(parts, db).await,
        "EXPIRE" => commands::expire::execute(parts, db).await,
        "KEYS" => commands::keys::execute(parts, db).await,
        "FLUSHALL" => commands::flushall::execute(parts, db).await,
        _ => RespValue::Error("ERR unknown command".into()),
    }
}

pub async fn process(input: String, db: &Db) -> String {
    let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    let resp = process_parts(parts, db).await;
    String::from_utf8_lossy(&resp.to_bytes()).to_string()
}
