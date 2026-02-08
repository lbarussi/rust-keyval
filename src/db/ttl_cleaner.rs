use std::time::Instant;
use tokio::time::{sleep, Duration};

use super::storage::Db;

pub async fn start_cleaner(db: Db) {

    tokio::spawn(async move {

        loop {

            {
                let mut db = db.lock().await;
                let now = Instant::now();

                db.retain(|_, v| {
                    if let Some(exp) = v.expire_at {
                        exp > now
                    } else {
                        true
                    }
                });
            }

            sleep(Duration::from_secs(5)).await;
        }

    });

}
