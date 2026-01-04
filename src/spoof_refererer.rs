use rand::Rng;

use crate::referrers_domains_index::{DOMAINS_LEN, DOMAINS_LENS, DOMAINS_OFFSETS};
use crate::referrers_hq_index::{HQ_LEN, HQ_LENS, HQ_OFFSETS};

static HQ_BLOB: &[u8] = include_bytes!("../assets/hq_urls_blob.bin");
static DOMAINS_BLOB: &[u8] = include_bytes!("../assets/domains_blob.bin");

#[inline]
/// Hq at.
fn hq_at(i: usize) -> &'static str {
    let off = HQ_OFFSETS[i] as usize;
    let len = HQ_LENS[i] as usize;
    let slice = &HQ_BLOB[off..off + len];
    unsafe { std::str::from_utf8_unchecked(slice) }
}

#[inline]
/// Domain url at.
fn domain_url_at(i: usize) -> &'static str {
    let off = DOMAINS_OFFSETS[i] as usize;
    let len = DOMAINS_LENS[i] as usize;
    let slice = &DOMAINS_BLOB[off..off + len];
    unsafe { std::str::from_utf8_unchecked(slice) }
}

/// hq_pct typical 5..20
#[inline]
pub fn spoof_referrer_weighted_rng<R: Rng + ?Sized>(rng: &mut R, hq_pct: u8) -> &'static str {
    let roll: u8 = rng.random_range(0..100);
    let hq_pct = hq_pct.min(100);

    if roll < hq_pct && HQ_LEN > 0 {
        return hq_at(rng.random_range(0..HQ_LEN));
    }

    if DOMAINS_LEN > 0 {
        return domain_url_at(rng.random_range(0..DOMAINS_LEN));
    }

    // last resort
    "https://google.com/"
}

/// Default: 10% HQ
pub fn spoof_referrer() -> &'static str {
    spoof_referrer_rng(&mut rand::rng())
}

/// Default: 10% HQ, 90% 1M
pub fn spoof_referrer_rng<R: Rng + ?Sized>(rng: &mut R) -> &'static str {
    spoof_referrer_weighted_rng(rng, 10)
}

/// Takes a URL and returns a convincing Google referer URL using the domain name or IP. Not used in latest chrome versions.
///
/// Handles:
/// - Domain names with or without subdomains
/// - IP addresses (removes periods)
///
/// # Examples
/// ```
/// use spider_fingerprint::spoof_refererer::spoof_referrer_google;
/// use url::Url;
///
/// let url = Url::parse("https://www.example.com/test").unwrap();
/// assert_eq!(spoof_referrer_google(&url), Some("https://www.google.com/search?q=example".to_string()));
///
/// let url = Url::parse("http://192.168.1.1/").unwrap();
/// assert_eq!(spoof_referrer_google(&url), Some("https://www.google.com/search?q=19216811".to_string()));
/// ```
pub fn spoof_referrer_google(parsed: &url::Url) -> Option<String> {
    let host = parsed.host_str()?;

    // Strip www. if present
    let stripped = host.strip_prefix("www.").unwrap_or(host);

    // Handle IPv4
    if stripped.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Some(format!(
            "https://www.google.com/search?q={}",
            stripped.replace('.', "")
        ));
    }

    // Handle IPv6: remove colons and brackets
    if stripped.contains(':') {
        let cleaned = stripped.replace(['[', ']', ':'], "");
        if !cleaned.is_empty() {
            return Some(format!("https://www.google.com/search?q={}", cleaned));
        } else {
            return None;
        }
    }

    // Handle domain names
    let labels: Vec<&str> = stripped.split('.').collect();
    if labels.len() >= 2 {
        Some(format!("https://www.google.com/search?q={}", labels[0]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_standard_domain() {
        let url = Url::parse("https://www.example.com/test").unwrap();
        let result = spoof_referrer_google(&url);
        assert_eq!(
            result,
            Some("https://www.google.com/search?q=example".to_string())
        );
    }

    #[test]
    fn test_domain_without_www() {
        let url = Url::parse("https://example.com").unwrap();
        let result = spoof_referrer_google(&url);
        assert_eq!(
            result,
            Some("https://www.google.com/search?q=example".to_string())
        );
    }

    #[test]
    fn test_subdomain() {
        let url = Url::parse("https://blog.shop.site.org").unwrap();
        let result = spoof_referrer_google(&url);
        assert_eq!(
            result,
            Some("https://www.google.com/search?q=blog".to_string())
        );
    }

    #[test]
    fn test_ip_address() {
        let url = Url::parse("http://192.168.1.1/").unwrap();
        let result = spoof_referrer_google(&url);
        assert_eq!(
            result,
            Some("https://www.google.com/search?q=19216811".to_string())
        );
    }

    #[test]
    fn test_ipv6_address() {
        let url = Url::parse("http://[::1]/").unwrap();
        let result = spoof_referrer_google(&url);
        assert_eq!(
            result,
            Some("https://www.google.com/search?q=1".to_string())
        );
    }

    #[test]
    fn test_localhost() {
        let url = Url::parse("http://localhost").unwrap();
        let result = spoof_referrer_google(&url);
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_url() {
        let url = Url::parse("http:///invalid").unwrap();
        let result = spoof_referrer_google(&url);
        assert_eq!(result, None);
    }

    #[test]
    fn test_spoof_referrer_returns_nonempty() {
        let s = spoof_referrer();
        assert!(!s.is_empty());
    }
}
