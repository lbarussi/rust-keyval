use std::time::Instant;

use crate::db::storage::Db;
use crate::db::value::ValueEntry;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {

    if parts.len() < 2 {
        return "ERR usage INCR key\n".into();
    }

    let mut db = db.lock().await;

    if let Some(entry) = db.get(parts[1]) {
        if let Some(exp) = entry.expire_at {
            if Instant::now() > exp {
                db.remove(parts[1]);
            }
        }
    }

    let entry: &mut ValueEntry = db.entry(parts[1].into()).or_insert(ValueEntry {
        value: "0".into(),
        expire_at: None,
    });

    match entry.value.parse::<i64>() {
        Ok(num) => {
            let new = num + 1;
            entry.value = new.to_string();
            format!("{}\n", new)
        }
        Err(_) => "ERR value is not integer\n".into(),
    }
}
