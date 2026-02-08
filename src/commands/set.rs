use std::time::{Instant, Duration};

use crate::db::storage::Db;
use crate::db::value::ValueEntry;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 3 {
        return "ERR usage SET key value [EX seconds]\n".into();
    }

    let mut expire = None;

    if parts.len() >= 5 && parts[3].to_uppercase() == "EX" {
        if let Ok(sec) = parts[4].parse::<u64>() {
            expire = Some(Instant::now() + Duration::from_secs(sec));
        }
    }

    let mut db = db.lock().await;

    db.insert(
        parts[1].into(),
        ValueEntry {
            value: parts[2].into(),
            expire_at: expire,
        },
    );

    "OK\n".into()
}
