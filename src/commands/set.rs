use std::time::{Duration, Instant};
use crate::db::storage::Db;
use crate::db::value::ValueEntry;
use crate::protocol::resp::encoder::RespValue;

pub async fn execute(parts: Vec<String>, db: &Db) -> RespValue {
    if parts.len() < 3 {
        return RespValue::Error("ERR usage SET key value [EX seconds]".into());
    }

    let key = parts[1].clone();
    let value = parts[2].clone();

    let mut expire = None;
    if parts.len() >= 5 && parts[3].to_uppercase() == "EX" {
        if let Ok(sec) = parts[4].parse::<u64>() {
            expire = Some(Instant::now() + Duration::from_secs(sec));
        }
    }

    let mut db = db.lock().await;
    db.insert(key, ValueEntry { value, expire_at: expire });

    RespValue::SimpleString("OK".into())
}
