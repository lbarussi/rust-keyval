use std::time::Instant;

use crate::db::storage::Db;
use crate::db::value::ValueEntry;
use crate::protocol::resp::encoder::RespValue;

pub async fn execute(parts: Vec<String>, db: &Db) -> RespValue {
    if parts.len() < 2 {
        return RespValue::Error("ERR usage INCR key".into());
    }

    let key = parts[1].clone();
    let mut db = db.lock().await;

    if let Some(entry) = db.get(&key) {
        if let Some(exp) = entry.expire_at {
            if Instant::now() > exp {
                db.remove(&key);
            }
        }
    }

    let entry = db.entry(key).or_insert(ValueEntry {
        value: "0".into(),
        expire_at: None,
    });

    let current: i64 = match entry.value.parse() {
        Ok(n) => n,
        Err(_) => return RespValue::Error("ERR value is not an integer".into()),
    };

    let next = current + 1;
    entry.value = next.to_string();

    RespValue::Integer(next)
}
