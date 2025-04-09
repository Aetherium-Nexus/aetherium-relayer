use crate::AetheriumDomainProtocol;
use std::time::Duration;

#[cfg(feature = "float")]
use crate::U256;

const ETHEREUM_DECIMALS: u8 = 18;
const COSMOS_DECIMALS: u8 = 6;
const SOLANA_DECIMALS: u8 = 9;

/// Interval for querying the prometheus metrics endpoint.
/// This should be whatever the prometheus scrape interval is
pub const METRICS_SCRAPE_INTERVAL: Duration = Duration::from_secs(60);

/// Convert a u256 scaled integer value into the corresponding f64 value.
#[cfg(feature = "float")]
pub fn u256_as_scaled_f64(value: U256, domain: AetheriumDomainProtocol) -> f64 {
    let decimals = decimals_by_protocol(domain);
    value.to_f64_lossy() / (10u64.pow(decimals as u32) as f64)
}

/// Get the decimals each protocol typically uses for its lowest denomination
/// of the native token
pub fn decimals_by_protocol(protocol: AetheriumDomainProtocol) -> u8 {
    match protocol {
        AetheriumDomainProtocol::Cosmos => COSMOS_DECIMALS,
        AetheriumDomainProtocol::Sealevel => SOLANA_DECIMALS,
        _ => ETHEREUM_DECIMALS,
    }
}
