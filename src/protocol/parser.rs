use crate::commands;
use crate::db::storage::Db;

pub async fn process(input: String, db: &Db) -> String {
    let parts: Vec<String> = input
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    process_parts(parts, db).await
}

pub async fn process_parts(parts: Vec<String>, db: &Db) -> String {
    if parts.is_empty() {
        return "ERR empty command\n".into();
    }

    match parts[0].to_uppercase().as_str() {
        "PING" => "PONG\n".into(),
        "SET" => commands::set::execute(parts, db).await,
        "GET" => commands::get::execute(parts, db).await,
        "KEYS" => commands::keys::execute(parts, db).await,
        "DEL" => commands::del::execute(parts, db).await,
        "EXISTS" => commands::exists::execute(parts, db).await,
        "FLUSHALL" => commands::flushall::execute(parts, db).await,
        "INCR" => commands::incr::execute(parts, db).await,
        "EXPIRE" => commands::expire::execute(parts, db).await,
        _ => "ERR unknown command\n".into(),
    }
}