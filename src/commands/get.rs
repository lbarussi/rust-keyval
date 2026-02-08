use std::time::Instant;
use crate::db::storage::Db;
use crate::protocol::resp::encoder::RespValue;

pub async fn execute(parts: Vec<String>, db: &Db) -> RespValue {
    if parts.len() < 2 {
        return RespValue::Error("ERR usage GET key".into());
    }

    let key = &parts[1];
    let mut db = db.lock().await;

    if let Some(entry) = db.get(key) {
        if let Some(exp) = entry.expire_at {
            if Instant::now() > exp {
                db.remove(key);
                return RespValue::Bulk(None);
            }
        }
        return RespValue::Bulk(Some(entry.value.clone().into_bytes()));
    }

    RespValue::Bulk(None)
}
