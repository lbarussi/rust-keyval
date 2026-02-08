use std::time::{Duration, Instant};

use crate::db::storage::Db;
use crate::db::value::ValueEntry;

pub async fn execute(parts: Vec<String>, db: &Db) -> String {
    // parts: ["SET", "key", "value", ...]
    if parts.len() < 3 {
        return "ERR usage SET key value [EX seconds]\n".into();
    }

    let key = parts[1].clone();
    let value = parts[2].clone(); // ✅ pode ser ""

    let mut expire = None;

    if parts.len() >= 5 && parts[3].to_uppercase() == "EX" {
        if let Ok(sec) = parts[4].parse::<u64>() {
            expire = Some(Instant::now() + Duration::from_secs(sec));
        }
    }

    let mut db = db.lock().await;
    db.insert(
        key,
        ValueEntry {
            value,
            expire_at: expire,
        },
    );

    "OK\n".into()
}
