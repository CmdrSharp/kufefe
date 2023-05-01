use chrono::Utc;
use std::env;

pub trait Expire {
    /// Generates expiry timestamp
    fn generate_expiry(&self) -> i64 {
        let minutes: i64 = env::var("EXPIRE_MINUTES")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<i64>()
            .unwrap_or(60);

        Utc::now()
            .checked_add_signed(chrono::Duration::minutes(minutes))
            .unwrap()
            .timestamp()
    }
}
