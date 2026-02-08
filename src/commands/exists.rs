use crate::db::storage::Db;
use crate::protocol::resp::encoder::RespValue;

pub async fn execute(parts: Vec<String>, db: &Db) -> RespValue {
    if parts.len() < 2 {
        return RespValue::Error("ERR usage EXISTS key".into());
    }

    let key = &parts[1];
    let db = db.lock().await;

    RespValue::Integer(if db.contains_key(key) { 1 } else { 0 })
}
