#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/chrome_versions.rs"));

const CHROME_STATIC_VERSION: &str = "139.0.7258.66";

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
        .unwrap_or("99.0.0.0".into());

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
        .unwrap_or(139)
    };
}
