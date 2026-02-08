use std::time::{Duration, Instant};

use crate::db::storage::Db;
use crate::protocol::resp::encoder::RespValue;

pub async fn execute(parts: Vec<String>, db: &Db) -> RespValue {
    if parts.len() < 3 {
        return RespValue::Error("ERR usage EXPIRE key seconds".into());
    }

    let key = &parts[1];
    let seconds: u64 = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => return RespValue::Error("ERR invalid seconds".into()),
    };

    let mut db = db.lock().await;

    if let Some(entry) = db.get_mut(key) {
        entry.expire_at = Some(Instant::now() + Duration::from_secs(seconds));
        RespValue::Integer(1)
    } else {
        RespValue::Integer(0)
    }
}
