use crate::db::storage::Db;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 2 {
        return "ERR usage INCR key\n".into();
    }

    let mut db = db.lock().await;

    let value = db.entry(parts[1].into()).or_insert("0".into());

    match value.parse::<i64>() {
        Ok(num) => {
            let new = num + 1;
            *value = new.to_string();
            format!("{}\n", new)
        }
        Err(_) => "ERR value is not integer\n".into(),
    }
}
