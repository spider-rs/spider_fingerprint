use rand::Rng;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/chrome_versions.rs"));

pub(crate) const BASE_VERSION: u32 = 141;
pub(crate) const CHROME_STATIC_VERSION: &str = "141.0.7390.66";

#[cfg(not(feature = "std"))]
// Fallback if build script wasn't run — define the constant with default data
#[allow(dead_code)]
pub static CHROME_VERSIONS_BY_MAJOR: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "latest" => &[CHROME_STATIC_VERSION],
};

lazy_static::lazy_static! {
    // Get the latest chrome version as the base to use.
    pub static ref LATEST_CHROME_FULL_VERSION_FULL: &'static str = CHROME_VERSIONS_BY_MAJOR
        .get("latest")
        .and_then(|arr| arr.first().copied())
        .unwrap_or(&CHROME_STATIC_VERSION);
    /// The latest Chrome not a brand version, configurable via the `CHROME_NOT_A_BRAND_VERSION` env variable.
    pub static ref CHROME_NOT_A_BRAND_VERSION: String = std::env::var("CHROME_NOT_A_BRAND_VERSION")
        .ok()
        .and_then(|v| if v.is_empty() { None } else { Some(v) })
        .unwrap_or("8.0.0.0".into());

    /// Force the chrome version, configurable via the `CHROME_VERSION_FULL` env variable.
    pub static ref CHROME_VERSION_FULL: String = std::env::var("CHROME_VERSION_FULL")
        .ok()
        .and_then(|v| if v.is_empty() { None } else { Some(v) })
        .unwrap_or("".into());

    /// The latest Chrome version major ex: 137.
    pub static ref BASE_CHROME_VERSION: u32 = {
       if CHROME_VERSION_FULL.is_empty() {
            LATEST_CHROME_FULL_VERSION_FULL
            .split('.')
        } else {
            CHROME_VERSION_FULL
            .split('.')
        }.next()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(BASE_VERSION)
    };
}

/// Random version based on the get_default_version.
pub(crate) fn random_version_based_on_default_version_base<R: Rng>(rng: &mut R) -> String {
    let full = crate::spoof_user_agent::get_default_version();
    let mut it = full.split('.');

    let major = it
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(*BASE_CHROME_VERSION);
    let minor = it.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let build = it
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(7000);
    let patch = it.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(50);

    // minor: keep tiny if the default is 0 (most Chrome), otherwise ±2 around default
    let (minor_lo, minor_hi) = if minor == 0 {
        (0, 0)
    } else {
        (minor.saturating_sub(2), minor + 2)
    };
    let random_minor = rng.random_range(minor_lo..=(minor_hi));

    // build: clamp within ±2000 (e.g., 7390 -> [5390..=7890])
    let build_lo = build.saturating_sub(2000);
    let build_hi = build + 500;
    let random_build = rng.random_range(build_lo..=(build_hi));

    // patch: only allow the *upper* cap if random_build is NOT in the same
    // thousands bucket as the real build. Otherwise keep it close to 0.
    let same_thousands_bucket = (random_build / 1000) == (build / 1000);
    let patch_upper = if same_thousands_bucket {
        core::cmp::min(30, patch)
    } else {
        core::cmp::min(150, patch.saturating_add(30))
    };
    let random_patch = rng.random_range(0..=patch_upper);

    format!(
        "{}.{}.{}.{}",
        major, random_minor, random_build, random_patch
    )
}

/// Random version based on the get_default_version.
pub fn random_version_based_on_default_version() -> String {
    let mut rng = rand::rng();
    random_version_based_on_default_version_base(&mut rng)
}

#[test]
fn test_random_version_based_on_default_version() {
    let version = random_version_based_on_default_version();
    println!("{:?}", version);
    assert!(!version.is_empty());
}
