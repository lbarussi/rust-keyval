use crate::db::storage::Db;

pub async fn execute(parts: Vec<&str>, db: &Db) -> String {
    if parts.len() < 3 {
        return "Err usage set key value/n".into();
    }

    let mut db = db.lock().await;
    db.insert(parts[1].into(), parts[2..].join(" "));

    return "Ok/n".into();
}