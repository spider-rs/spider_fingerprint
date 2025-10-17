use crate::versions::BASE_VERSION;
use crate::{mobile_model_from_user_agent, BASE_CHROME_VERSION, CHROME_VERSIONS_BY_MAJOR};
use rand::prelude::IndexedRandom;
use rand::{rng, Rng};

/// Represents a full Chrome version (major.minor.build.patch), as seen in `chrome-for-testing`.
///
/// Used for fingerprinting, spoofing, and matching known-good Chrome versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChromeVersion {
    /// Major version component (e.g., `136` in `136.0.7103.114`).
    pub major: u32,
    /// Minor version component (usually `0` for Chrome public releases).
    pub minor: u32,
    /// Build version component (e.g., `7103` in `136.0.7103.114`).
    pub build: u32,
    /// Patch version component (e.g., `114` in `136.0.7103.114`).
    pub patch: u32,
}

impl ChromeVersion {
    /// Constructs a new `ChromeVersion`.
    ///
    /// # Example
    /// ```
    /// use spider_fingerprint::spoof_user_agent::ChromeVersion;
    ///
    /// let v = ChromeVersion::new(136, 0, 7103, 114);
    /// assert_eq!(v.major, 136);
    /// ```
    pub fn new(major: u32, minor: u32, build: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            build,
            patch,
        }
    }

    pub fn from_str(version: &str) -> Self {
        let parts: Vec<u32> = version.split('.').map(|s| s.parse().unwrap_or(0)).collect();
        Self {
            major: *parts.get(0).unwrap_or(&0),
            minor: *parts.get(1).unwrap_or(&0),
            build: *parts.get(2).unwrap_or(&0),
            patch: *parts.get(3).unwrap_or(&0),
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.major, self.minor, self.build, self.patch
        )
    }

    /// Spoof with optional decrements for each digit
    pub fn spoofed(&self, dec_major: u32, dec_minor: u32, dec_build: u32, dec_patch: u32) -> Self {
        Self {
            major: self.major.saturating_sub(dec_major),
            minor: self.minor.saturating_sub(dec_minor),
            build: self.build.saturating_sub(dec_build),
            patch: self.patch.saturating_sub(dec_patch),
        }
    }
}

/// Random range between latest version.
pub fn random_spoofed_version_base(latest: &str, rng: &mut impl Rng) -> String {
    let latest_ver = ChromeVersion::from_str(latest);

    let dec_major = rng.random_range(0..=2); // spoof up to 2 versions back
    let dec_minor = rng.random_range(0..=latest_ver.minor);
    let dec_build = rng.random_range(0..=latest_ver.build);
    let dec_patch = rng.random_range(0..=latest_ver.patch);

    latest_ver
        .spoofed(dec_major, dec_minor, dec_build, dec_patch)
        .to_string()
}

/// Random spoofed version.
pub fn random_spoofed_version(latest: &str) -> String {
    let mut rng = rng();
    random_spoofed_version_base(latest, &mut rng)
}

/// Random spoofed version.
pub fn random_spoofed_version_rng(latest: &str, rng: &mut impl Rng) -> String {
    random_spoofed_version_base(latest, rng)
}

/// Generate a real spoof for chrome full version.
pub fn smart_spoof_chrome_full_version(ua_major: &str, // e.g. "136"
) -> String {
    let mut rng = rng();

    // Try the latest full version from "latest" key in PHF
    let latest_versions = CHROME_VERSIONS_BY_MAJOR
        .get("latest")
        .and_then(|arr| arr.first())
        .map(|s| *s)
        .unwrap_or(get_default_version()); // Fallback default (shouldn't hit if PHF is built)

    // 75% chance: if ua_major is also the latest, just use the true latest version
    let ua_major = ua_major.split('.').next().unwrap_or(ua_major);
    let same_major = latest_versions.starts_with(ua_major);

    if same_major && rng.random_bool(0.75) {
        return crate::versions::random_version_based_on_default_version_base(&mut rng);
    }

    // Otherwise, pick a random known-good version in the given major
    if let Some(versions) = CHROME_VERSIONS_BY_MAJOR.get(ua_major) {
        if !versions.is_empty() {
            if let Some(v) = versions.choose(&mut rng) {
                return v.to_string();
            }
        }
    }

    if same_major {
        latest_versions.to_string()
    } else {
        random_spoofed_version_rng(ua_major, &mut rng)
    }
}

/// Represents a browser brand and its version, used for spoofing `userAgentData.fullVersionList`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BrandEntry {
    /// The name of the browser brand (e.g., "Chromium", "Not-A.Brand").
    pub brand: String,
    /// The full version string of the brand (e.g., "122.0.0.0").
    pub version: String,
}

/// Represents the high-entropy values returned by `navigator.userAgentData.getHighEntropyValues()`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HighEntropyUaData {
    /// The CPU architecture of the device (e.g., "x86", "arm").
    pub architecture: String,
    /// The device model (mostly non-empty for Android devices).
    pub model: String,
    /// The bitness property.
    pub bitness: String,
    /// The platform being used.
    pub platform: String,
    /// The OS platform version (e.g., "10.0" for Windows 10, "13" for Android 13).
    pub platform_version: String,
    /// A list of brand/version pairs representing the full user agent fingerprint.
    pub full_version_list: Vec<BrandEntry>,
    /// The ua full version.
    pub ua_full_version: String,
    /// Is this user-agent part of the mobile list?
    pub mobile: bool,
    /// A boolean indicating if the user agent’s binary is running in 32-bit mode on 64-bit Windows.
    pub wow64_ness: bool,
}

/// Get the default chrome version.
pub fn get_default_version() -> &'static str {
    if !&crate::CHROME_VERSION_FULL.is_empty() {
        crate::CHROME_VERSION_FULL.as_str()
    } else {
        &crate::LATEST_CHROME_FULL_VERSION_FULL
    }
}

/// Build the entropy data.
pub fn build_high_entropy_data(user_agent: &Option<&str>) -> HighEntropyUaData {
    let user_agent: &str = user_agent.as_deref().map_or("", |v| v);
    let full_version = user_agent
        .split_whitespace()
        .find_map(|s| s.strip_prefix("Chrome/"))
        .unwrap_or(&get_default_version());

    let mut older_brand = true;
    let mut chrome_major = 0;

    let (architecture, model, platform, platform_version, bitness): (
        &str,
        String,
        &str,
        String,
        &str,
    ) = if user_agent.contains("Android") {
        let version = user_agent
            .split(';')
            .find_map(|s| s.trim().strip_prefix("Android "))
            .unwrap_or("13");
        chrome_major = version.parse::<u32>().ok().unwrap_or_default();

        let model = user_agent
            .split(';')
            .nth(2)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let bitness = if user_agent.contains("arm64") || user_agent.contains("aarch64") {
            "64"
        } else {
            "32"
        };

        ("arm", model, "Android", version.to_string(), bitness)
    } else if user_agent.contains("Windows NT") {
        let version = user_agent
            .split("Windows NT ")
            .nth(1)
            .and_then(|s| s.split(';').next())
            .unwrap_or("10.0");
        chrome_major = version.parse::<u32>().ok().unwrap_or_default();

        let bitness = if user_agent.contains("Win64")
            || user_agent.contains("x64")
            || user_agent.contains("WOW64")
        {
            "64"
        } else {
            "32"
        };

        (
            "x86",
            "".to_string(),
            "Windows",
            version.to_string(),
            bitness,
        )
    } else if user_agent.contains("Mac OS X") {
        chrome_major = full_version
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(*BASE_CHROME_VERSION);

        // 139 plus
        let base_mac = 15.5;

        let platform_version = if chrome_major == 138 {
            "14.6.1".into()
        } else if (139..=141).contains(&chrome_major) {
            "15.5.0".into()
        } else {
            let sub_delta = chrome_major < *BASE_CHROME_VERSION;

            let delta = if sub_delta {
                ((*BASE_CHROME_VERSION - chrome_major) as f32 * 0.1).round()
            } else if chrome_major > *BASE_CHROME_VERSION {
                ((chrome_major - *BASE_CHROME_VERSION) as f32 * 0.1).round()
            } else if chrome_major > BASE_VERSION {
                let ft = chrome_major - *BASE_CHROME_VERSION;
                let ft = ft as f32;
                ft + 0.9
            } else if *BASE_CHROME_VERSION > BASE_VERSION {
                let ft = *BASE_CHROME_VERSION - BASE_VERSION;
                let ft = ft as f32;
                ft + 0.9
            } else {
                0.0
            };

            let mac_major = if sub_delta {
                base_mac - delta
            } else {
                base_mac + delta
            };

            if mac_major >= 136.0 {
                older_brand = false;
            }

            if mac_major < 15.0 {
                format!("{:.1}.1", mac_major)
            } else {
                format!("{:.1}.0", mac_major)
            }
        };

        ("arm", "".to_string(), "macOS", platform_version, "64")
    } else if user_agent.contains("Linux") {
        let platform_version = full_version
            .split('.')
            .take(3)
            .collect::<Vec<_>>()
            .join(".");

        chrome_major = platform_version.parse::<u32>().ok().unwrap_or_default();

        let bitness = if user_agent.contains("x86_64")
            || user_agent.contains("amd64")
            || user_agent.contains("arm64")
        {
            "64"
        } else {
            "32"
        };

        ("x86", "".to_string(), "Linux", platform_version, bitness)
    } else {
        ("x86", "".to_string(), "Unknown", "1.0.0".to_string(), "64")
    };

    // chrome canary order - Not, Chromium, and "Google Chrome ( use a flag for it. )
    // base canary is released 2 versions ahead of chrome.
    // canary not a brand starts at 8.0 while normal chrome "99"
    // we need to spoof this for firefox.
    let full_version_list = if chrome_major == 141 {
        vec![
            BrandEntry {
                brand: "Google Chrome".into(),
                version: full_version.into(),
            },
            BrandEntry {
                brand: "Chromium".into(),
                version: full_version.into(),
            },
            BrandEntry {
                brand: "Not?A_Brand".into(),
                version: crate::CHROME_NOT_A_BRAND_VERSION.clone(),
            },
        ]
    } else {
        vec![
            BrandEntry {
                brand: "Chromium".into(),
                version: full_version.into(),
            },
            BrandEntry {
                brand: "Google Chrome".into(),
                version: full_version.into(),
            },
            BrandEntry {
                brand: if older_brand {
                    "Not-A.Brand"
                } else if platform_version == "15.5.0" {
                    "Not;A=Brand"
                } else {
                    "Not.A/Brand"
                }
                .into(),
                version: crate::CHROME_NOT_A_BRAND_VERSION.clone(),
            },
        ]
    };

    let mobile = mobile_model_from_user_agent(user_agent);
    let mobile_device = mobile.is_some();

    HighEntropyUaData {
        architecture: architecture.to_string(),
        bitness: bitness.to_string(),
        model: if let Some(mobile_model) = mobile {
            mobile_model.to_string()
        } else {
            model
        },
        platform: platform.to_string(),
        platform_version,
        full_version_list,
        ua_full_version: smart_spoof_chrome_full_version(full_version),
        mobile: mobile_device,
        wow64_ness: false,
    }
}

/// Spoof navigator.userAgentData.
pub fn spoof_user_agent_data_high_entropy_values(data: &HighEntropyUaData) -> String {
    let brands = data
        .full_version_list
        .iter()
        .map(|b| {
            let major = b.version.split('.').next().unwrap_or("99");
            format!("{{brand:'{}',version:'{}'}}", b.brand, major)
        })
        .collect::<Vec<_>>()
        .join(",");
    let full_versions = data
        .full_version_list
        .iter()
        .map(|b| format!("{{brand:'{}',version:'{}'}}", b.brand, b.version))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r###"(()=>{{if(typeof NavigatorUAData==='undefined')window.NavigatorUAData=function NavigatorUAData(){{}};const p=NavigatorUAData.prototype,v=Object.create(p),d={{architecture:'{}',bitness:'{}',model:'{}',platformVersion:'{}',fullVersionList:[{}],brands:[{}],mobile:!1,platform:'{}'}};Object.defineProperties(v,{{brands:{{value:d.brands,enumerable:true}},mobile:{{value:d.mobile,enumerable:true}},platform:{{value:d.platform,enumerable:true}}}});Object.defineProperties(p,{{brands:{{get:function brands(){{return this.brands}}}},mobile:{{get:function mobile(){{return this.mobile}}}},platform:{{get:function platform(){{return this.platform}}}}}});function getHighEntropyValues(keys){{keys=Array.isArray(keys)?keys:[];var out={{}};for(var i=0;i<keys.length;i++){{var k=keys[i];if(k==='architecture'||k==='bitness'||k==='model'||k==='platformVersion'||k==='uaFullVersion'||k==='fullVersionList'){{out[k]=(k==='uaFullVersion'?'{}':d[k]);}}}}return Promise.resolve(Object.assign({{brands:d.brands,mobile:d.mobile,platform:d.platform}},out))}};Object.defineProperty(p,'getHighEntropyValues',{{value:getHighEntropyValues}});function toJSON(){{return{{brands:this.brands,mobile:this.mobile,platform:this.platform}}}}Object.defineProperty(p,'toJSON',{{value:toJSON}});const f=()=>v;Object.defineProperty(f,'toString',{{value:()=>`function get userAgentData() {{ [native code] }}`}});Object.defineProperty(Navigator.prototype,'userAgentData',{{get:f,configurable:!0}});}})();"###,
        data.architecture,
        data.bitness,
        data.model,
        data.platform_version,
        full_versions,
        brands,
        data.platform,
        data.ua_full_version
    )
}

/// Returns the browser *major* version from a UA string (allocation-free, no deps).
pub fn ua_major(ua: &str) -> Option<u16> {
    // Prioritized tokens (desktop/mobile variants first).
    const TOKENS: &[&str] = &[
        "Chrome/",
        "CriOS/",
        "Edg/",
        "EdgiOS/",
        "OPR/",
        "OPiOS/",
        "Opera/",
        "Firefox/",
        "FxiOS/",
        "Brave/",
        "Chromium/",
        "Version/", // Safari
    ];

    let bytes = ua.as_bytes();

    // Fast path: look for any known token and parse digits after it.
    for &t in TOKENS {
        if let Some(pos) = find_substr(bytes, t.as_bytes()) {
            return parse_major_digits(&bytes[pos + t.len()..]);
        }
    }

    // Fallback: skip "Mozilla/5.0 (...)" and scan for next token-like "word/<digits>"
    if let Some(end_paren) = ua.find(") ").map(|p| p + 2) {
        let b = &bytes[end_paren..];
        let mut i = 0;
        while i < b.len() {
            // find next '/'
            if let Some(slash) = find_byte(&b[i..], b'/') {
                let j = i + slash + 1;
                if let Some(v) = parse_major_digits(&b[j..]) {
                    return Some(v);
                }
                i = j;
            } else {
                break;
            }
        }
    }

    None
}

#[inline]
fn parse_major_digits(bytes: &[u8]) -> Option<u16> {
    let mut val: u32 = 0;
    let mut saw = false;
    for &ch in bytes {
        if ch.is_ascii_digit() {
            saw = true;
            val = val * 10 + u32::from(ch - b'0');
            if val > u16::MAX as u32 {
                return None;
            }
        } else {
            break; // stop at '.' or any non-digit
        }
    }
    if saw {
        Some(val as u16)
    } else {
        None
    }
}

/// Tiny, dependency-free substring search optimized for short needles.
#[inline]
fn find_substr(hay: &[u8], needle: &[u8]) -> Option<usize> {
    match needle {
        [] => Some(0),
        [first, rest @ ..] => {
            let n = needle.len();
            let mut i = 0;
            while i + n <= hay.len() {
                // quick check on first byte
                if hay[i] == *first && &hay[i + 1..i + n] == rest {
                    return Some(i);
                }
                i += 1;
            }
            None
        }
    }
}

/// Find a single byte.
#[inline]
fn find_byte(hay: &[u8], byte: u8) -> Option<usize> {
    for (i, &b) in hay.iter().enumerate() {
        if b == byte {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ua_major_examples() {
        // Chrome desktop
        let chrome = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                      AppleWebKit/537.36 (KHTML, like Gecko) \
                      Chrome/124.0.6367.118 Safari/537.36";
        assert_eq!(ua_major(chrome), Some(124));

        // Safari (uses Version/x.y before Safari/)
        let safari = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) \
                      AppleWebKit/605.1.15 (KHTML, like Gecko) \
                      Version/17.1 Safari/605.1.15";
        assert_eq!(ua_major(safari), Some(17));

        // Firefox desktop
        let firefox = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:128.0) \
                       Gecko/20100101 Firefox/128.0";
        assert_eq!(ua_major(firefox), Some(128));
    }

    #[test]
    fn build_high_entropy_data_test() {
        let data = build_high_entropy_data(&Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36"));
        assert!(data.platform == "macOS");
        assert!(data.platform_version == "15.5.0");
    }
}
