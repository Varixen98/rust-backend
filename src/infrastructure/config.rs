use std::env;
use time::Duration;


pub struct AppConfig {
    pub jwt_secret: String,
    pub access_token_ttl: String,
    pub refresh_token_ttl: String
}

impl AppConfig {
    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set!");

        let refresh_jwt_ttl_days: i64 = env::var("REFRESH_TOKEN_TTL_DAYS")
            .unwrap_or("30".to_string())
            .parse()
            .expect("REFRESH_TOKEN_TTL_DAYS must be set!");

        let access_token_ttl_secs: i64 = env::var("ACCESS_TOKEN_TTL_SECS")
            .unwrap_or("30".to_string())
            .parse()
            .expect("ACCESS_TOKEN_TTL_SECS must be set!");
    }
}


