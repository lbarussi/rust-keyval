use std::time::Instant;

use crate::db::storage::Db;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 2 {
        return "ERR usage GET key\n".into();
    }

    let mut db = db.lock().await;

    if let Some(entry) = db.get(parts[1]) {

        if let Some(exp) = entry.expire_at {
            if Instant::now() > exp {
                db.remove(parts[1]);
                return "NULL\n".into();
            }
        }

        return format!("{}\n", entry.value);
    }

    "NULL\n".into()
}
