use std::time::{Instant, Duration};

use crate::db::storage::Db;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 3 {
        return "ERR usage EXPIRE key seconds\n".into();
    }

    let seconds = match parts[2].parse::<u64>() {
        Ok(v) => v,
        Err(_) => return "ERR invalid seconds\n".into(),
    };

    let mut db = db.lock().await;

    if let Some(entry) = db.get_mut(parts[1]) {
        entry.expire_at = Some(Instant::now() + Duration::from_secs(seconds));
        return "1\n".into();
    }

    "0\n".into()
}
