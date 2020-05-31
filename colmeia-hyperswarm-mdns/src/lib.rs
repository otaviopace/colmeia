static HYPERSWARM_DOMAIN: &'static str = ".hyperswarm.local";

pub mod locator;
pub mod socket;

pub use locator::Locator;

/// {hash}.hyperswarm.local
fn domain_name(hash: &str) -> String {
    let mut domain = String::with_capacity(hash.len() + HYPERSWARM_DOMAIN.len());
    domain.push_str(hash);
    domain.push_str(HYPERSWARM_DOMAIN);
    domain
}

fn bytes_to_hash(discovery_key: &[u8]) -> String {
    hex::encode(discovery_key).chars().take(40).collect()
}

pub fn hash_to_domain(hash: &[u8]) -> String {
    let hash_as_str = bytes_to_hash(hash);
    domain_name(&hash_as_str)
}
