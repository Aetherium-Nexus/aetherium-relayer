use std::time::Duration;

use eyre::Result;
use rusoto_core::{HttpClient, HttpConfig};

pub const AET_POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(15);

/// Create a new HTTP client with a timeout for the connection pool.
pub fn http_client_with_timeout() -> Result<HttpClient> {
    let mut config = HttpConfig::new();
    config.pool_idle_timeout(AET_POOL_IDLE_TIMEOUT);
    Ok(HttpClient::new_with_config(config)?)
}
