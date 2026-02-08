use crate::db::storage::Db;
use crate::protocol::resp::encoder::RespValue;

pub async fn execute(parts: Vec<String>, db: &Db) -> RespValue {
    if parts.len() < 2 {
        return RespValue::Error("ERR usage DEL key".into());
    }

    let key = &parts[1];
    let mut db = db.lock().await;

    let deleted = if db.remove(key).is_some() { 1 } else { 0 };
    RespValue::Integer(deleted)
}
