/// Versions for chrome.
pub mod versions;

/// Builder types.
pub mod configs;
/// Custom static profiles.
pub mod profiles;
/// GPU spoofs.
pub mod spoof_gpu;
#[cfg(feature = "headers")]
/// Spoof HTTP headers.
pub mod spoof_headers;
/// Spoof mouse-movement.
pub mod spoof_mouse_movement;
/// Referer headers.
pub mod spoof_refererer;
/// User agent.
pub mod spoof_user_agent;
/// Spoof viewport.
pub mod spoof_viewport;
/// WebGL spoofs.
pub mod spoof_webgl;
/// Generic spoofs.
pub mod spoofs;

#[cfg(feature = "headers")]
pub use spoof_headers::emulate_headers;
pub use spoof_refererer::spoof_referrer;

use configs::{AgentOs, Tier};
use profiles::{
    gpu::{select_random_gpu_profile, GpuProfile},
    gpu_limits::{build_gpu_request_adapter_script_from_limits, GpuLimits},
};
use rand::Rng;
use spoof_gpu::{
    build_gpu_spoof_script_wgsl, FP_JS, FP_JS_GPU_LINUX, FP_JS_GPU_MAC, FP_JS_GPU_WINDOWS,
    FP_JS_LINUX, FP_JS_MAC, FP_JS_WINDOWS,
};
use spoofs::{
    resolve_dpr, spoof_history_length_script, spoof_media_codecs_script, spoof_media_labels_script,
    spoof_screen_script_rng, spoof_touch_screen, DISABLE_DIALOGS, SPOOF_NOTIFICATIONS,
    SPOOF_PERMISSIONS_QUERY,
};

#[cfg(feature = "headers")]
pub use http;
pub use url;

pub use versions::{
    BASE_CHROME_VERSION, CHROME_NOT_A_BRAND_VERSION, CHROME_VERSIONS_BY_MAJOR, CHROME_VERSION_FULL,
    LATEST_CHROME_FULL_VERSION_FULL,
};

const P_EDG: usize = 0; // "edg/"
const P_OPR: usize = 1; // "opr/"
const P_CHR: usize = 2; // "chrome/"
const P_AND: usize = 3; // "android"

lazy_static::lazy_static! {
    pub static ref MOBILE_PATTERNS: [&'static str; 38] = [
        // Apple
        "iphone", "ipad", "ipod",
        // Android
        "android",
        // Generic mobile
        "mobi", "mobile", "touch",
        // Specific Android browsers/devices
        "silk", "nexus", "pixel", "huawei", "honor", "xiaomi", "miui", "redmi",
        "oneplus", "samsung", "galaxy", "lenovo", "oppo", "vivo", "realme",
        // Mobile browsers
        "opera mini", "opera mobi", "ucbrowser", "ucweb", "baidubrowser", "qqbrowser",
        "dolfin", "crmo", "fennec", "iemobile", "webos", "blackberry", "bb10",
        "playbook", "palm", "nokia"
    ];

    /// Common mobile indicators for user-agent detection.
    pub static ref MOBILE_MATCHER: aho_corasick::AhoCorasick = aho_corasick::AhoCorasickBuilder::new()
        .ascii_case_insensitive(true)
        .build(MOBILE_PATTERNS.as_ref())
        .expect("failed to compile AhoCorasick patterns");


    pub static ref ALLOWED_UA_DATA: aho_corasick::AhoCorasick = aho_corasick::AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .match_kind(aho_corasick::MatchKind::LeftmostFirst)
            .build(&["edg/", "opr/", "chrome/", "android"])
            .expect("valid device patterns");

}

#[inline]
fn parse_major_after(s: &str, end_token: usize) -> Option<u32> {
    if end_token >= s.len() {
        return None;
    }
    let bytes = s.as_bytes();
    let mut i = end_token;
    let mut n: u32 = 0;
    let mut saw = false;
    while i < bytes.len() {
        let b = bytes[i];
        if (b'0'..=b'9').contains(&b) {
            saw = true;
            n = n.saturating_mul(10) + (b - b'0') as u32;
            i += 1;
        } else {
            break;
        }
    }
    saw.then_some(n)
}

/// The user-agent allows navigator.userAgentData.getHighEntropyValues
pub fn ua_allows_gethighentropy(ua: &str) -> bool {
    let mut seen: u32 = 0;
    let mut endpos: [Option<usize>; 4] = [None; 4];

    for m in ALLOWED_UA_DATA.find_iter(ua) {
        let idx = m.pattern().as_usize();
        if endpos[idx].is_none() {
            endpos[idx] = Some(m.end());
            seen |= 1u32 << idx;
        }
    }

    let has = |i: usize| (seen & (1u32 << i)) != 0;
    let is_android = has(P_AND);

    if let Some(end) = endpos[P_EDG] {
        if is_android {
            return false;
        }
        return parse_major_after(ua, end).map_or(false, |v| v >= 90);
    }
    if let Some(end) = endpos[P_OPR] {
        return parse_major_after(ua, end).map_or(false, |v| {
            if is_android {
                v >= 64
            } else {
                v >= 76
            }
        });
    }
    if let Some(end) = endpos[P_CHR] {
        return parse_major_after(ua, end).map_or(false, |v| v >= 90);
    }
    false
}

/// Returns `true` if the user-agent is likely a mobile browser.
pub fn is_mobile_user_agent(user_agent: &str) -> bool {
    MOBILE_MATCHER.find(user_agent).is_some()
}

/// Does the user-agent matches a mobile device indicator.
pub fn mobile_model_from_user_agent(user_agent: &str) -> Option<&'static str> {
    MOBILE_MATCHER
        .find(&user_agent)
        .map(|m| MOBILE_PATTERNS[m.pattern()])
}

/// Get a random device hardware concurrency.
pub fn get_random_hardware_concurrency(user_agent: &str) -> usize {
    let gpu_profile = select_random_gpu_profile(get_agent_os(user_agent));
    gpu_profile.hardware_concurrency
}

/// Generate the initial stealth script to send in one command.
fn build_stealth_script_base(
    gpu_profile: &'static GpuProfile,
    tier: Tier,
    os: AgentOs,
    concurrency: bool,
) -> String {
    use crate::spoofs::{
        spoof_hardware_concurrency, unified_worker_override, worker_override, HIDE_CHROME,
        HIDE_CONSOLE, HIDE_WEBDRIVER, NAVIGATOR_SCRIPT, PLUGIN_AND_MIMETYPE_SPOOF,
    };

    let spoof_gpu = build_gpu_spoof_script_wgsl(gpu_profile.canvas_format);

    let spoof_webgl = if tier == Tier::BasicNoWorker {
        Default::default()
    } else if concurrency {
        unified_worker_override(
            gpu_profile.hardware_concurrency,
            gpu_profile.webgl_vendor,
            gpu_profile.webgl_renderer,
            tier != Tier::BasicNoWebglWithGPU,
        )
    } else {
        worker_override(gpu_profile.webgl_vendor, gpu_profile.webgl_renderer)
    };

    let spoof_concurrency = spoof_hardware_concurrency(gpu_profile.hardware_concurrency);

    let mut gpu_limit = GpuLimits::for_os(os);

    if gpu_profile.webgl_renderer
        != "ANGLE (Apple, ANGLE Metal Renderer: Apple M1, Unspecified Version)"
    {
        gpu_limit = gpu_limit.with_variation(gpu_profile.hardware_concurrency);
    }

    let spoof_gpu_adapter = build_gpu_request_adapter_script_from_limits(
        gpu_profile.webgpu_vendor,
        gpu_profile.webgpu_architecture,
        "",
        "",
        &gpu_limit,
    );

    if tier == Tier::Basic || tier == Tier::BasicNoWorker {
        format!(
            r#"{HIDE_CHROME}{HIDE_CONSOLE}{spoof_webgl}{spoof_gpu_adapter}{NAVIGATOR_SCRIPT}{PLUGIN_AND_MIMETYPE_SPOOF}"#
        )
    } else if tier == Tier::BasicWithConsole {
        format!(
            r#"{HIDE_CHROME}{spoof_webgl}{spoof_gpu_adapter}{NAVIGATOR_SCRIPT}{PLUGIN_AND_MIMETYPE_SPOOF}"#
        )
    } else if tier == Tier::BasicNoWebgl || tier == Tier::BasicNoWebglWithGPU {
        format!(
            r#"{HIDE_CHROME}{HIDE_CONSOLE}{spoof_concurrency}{NAVIGATOR_SCRIPT}{PLUGIN_AND_MIMETYPE_SPOOF}"#
        )
    } else if tier == Tier::Mid {
        format!(
            r#"{HIDE_CHROME}{HIDE_CONSOLE}{spoof_webgl}{spoof_gpu_adapter}{HIDE_WEBDRIVER}{NAVIGATOR_SCRIPT}{PLUGIN_AND_MIMETYPE_SPOOF}"#
        )
    } else if tier == Tier::Full {
        format!("{HIDE_CHROME}{HIDE_CONSOLE}{spoof_webgl}{spoof_gpu_adapter}{HIDE_WEBDRIVER}{NAVIGATOR_SCRIPT}{PLUGIN_AND_MIMETYPE_SPOOF}{spoof_gpu}")
    } else {
        Default::default()
    }
}

/// Generate the initial stealth script to send in one command.
pub fn build_stealth_script(tier: Tier, os: AgentOs) -> String {
    let gpu_profile = select_random_gpu_profile(os);
    build_stealth_script_base(gpu_profile, tier, os, true)
}

/// Generate the initial stealth script to send in one command without hardware concurrency.
pub fn build_stealth_script_no_concurrency(tier: Tier, os: AgentOs) -> String {
    let gpu_profile = select_random_gpu_profile(os);
    build_stealth_script_base(gpu_profile, tier, os, false)
}

/// Generate the initial stealth script to send in one command and profile.
pub fn build_stealth_script_with_profile(
    gpu_profile: &'static GpuProfile,
    tier: Tier,
    os: AgentOs,
) -> String {
    build_stealth_script_base(gpu_profile, tier, os, true)
}

/// Generate the initial stealth script to send in one command without hardware concurrency and profile.
pub fn build_stealth_script_no_concurrency_with_profile(
    gpu_profile: &'static GpuProfile,
    tier: Tier,
    os: AgentOs,
) -> String {
    build_stealth_script_base(gpu_profile, tier, os, false)
}

/// Generate the hide plugins script.
pub fn generate_hide_plugins() -> String {
    format!(
        "{}{}",
        crate::spoofs::NAVIGATOR_SCRIPT,
        crate::spoofs::PLUGIN_AND_MIMETYPE_SPOOF
    )
}

/// Simple function to wrap the eval script safely.
pub fn wrap_eval_script(source: &str) -> String {
    format!(r#"(()=>{{{}}})();"#, source)
}

/// The fingerprint type to use.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Fingerprint {
    /// Basic finterprint that includes webgl and gpu attempt spoof.
    Basic,
    /// Basic fingerprint that does not spoof the gpu. Used for real gpu based headless instances.
    /// This will bypass the most advanced anti-bots without the speed reduction of a virtual display.
    NativeGPU,
    /// None - no fingerprint and use the default browser fingerprinting. This may be a good option to use at times.
    #[default]
    None,
}

impl Fingerprint {
    /// Fingerprint should be used.
    pub fn valid(&self) -> bool {
        match &self {
            Self::Basic | Self::NativeGPU => true,
            _ => false,
        }
    }
}
/// Configuration options for browser fingerprinting and automation.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EmulationConfiguration {
    /// Enables stealth mode to help avoid detection by anti-bot mechanisms.
    pub tier: configs::Tier,
    /// If enabled, will auto-dismiss browser popups and dialogs.
    pub dismiss_dialogs: bool,
    /// The detailed fingerprint configuration for the browser session.
    pub fingerprint: Fingerprint,
    /// The agent os.
    pub agent_os: AgentOs,
    /// Is this firefox?
    pub firefox_agent: bool,
    /// Add userAgentData. Usually can be disabled when set via CDP for accuracy.
    pub user_agent_data: Option<bool>,
    /// Touch screen enabling or disabling emulation based on device?
    pub touch_screen: bool,
    /// Hardware concurrency emulation?
    pub hardware_concurrency: bool,
}

/// Get the OS being used.
pub fn get_agent_os(user_agent: &str) -> AgentOs {
    let mut agent_os = AgentOs::Unknown;

    if user_agent.contains("Chrome") {
        if user_agent.contains("Linux") {
            agent_os = AgentOs::Linux;
        } else if user_agent.contains("Mac") {
            agent_os = AgentOs::Mac;
        } else if user_agent.contains("Windows") {
            agent_os = AgentOs::Windows;
        } else if user_agent.contains("Android") {
            agent_os = AgentOs::Android;
        } else if user_agent.contains("iPhone")
            || user_agent.contains("iPad")
            || user_agent.contains("iOS")
        {
            agent_os = AgentOs::IPhone;
        }
    }

    agent_os
}

/// Setup the emulation defaults.
impl EmulationConfiguration {
    /// Setup the defaults.
    pub fn setup_defaults(user_agent: &str) -> EmulationConfiguration {
        let mut firefox_agent = false;

        let agent_os = get_agent_os(user_agent);

        if agent_os == AgentOs::Unknown {
            firefox_agent = user_agent.contains("Firefox");
        }

        let mut emulation_config = Self::default();

        emulation_config.firefox_agent = firefox_agent;
        emulation_config.agent_os = agent_os;
        emulation_config.touch_screen = false; // by default spider_chrome emulates touch over CDP.
        emulation_config.hardware_concurrency = true; // should be disabled and moved to CDP to cover all frames.

        emulation_config
    }
}

/// Join the scrips pre-allocated.
fn join_scripts<I: IntoIterator<Item = impl AsRef<str>>>(parts: I) -> String {
    // Heuristically preallocate some capacity (tweak as needed for your use-case).
    let mut script = String::with_capacity(4096);
    for part in parts {
        script.push_str(part.as_ref());
    }
    script
}

/// Emulate a real chrome browser.
pub fn emulate_base(
    user_agent: &str,
    config: &EmulationConfiguration,
    viewport: &Option<&crate::spoof_viewport::Viewport>,
    evaluate_on_new_document: &Option<Box<String>>,
    gpu_profile: Option<&'static GpuProfile>,
) -> Option<String> {
    let stealth = config.tier.stealth();
    let dismiss_dialogs = config.dismiss_dialogs;
    let agent_os = if config.agent_os == AgentOs::Unknown {
        get_agent_os(user_agent)
    } else {
        config.agent_os
    };
    let spoof_script = if stealth
        && ua_allows_gethighentropy(user_agent)
        && config.user_agent_data.unwrap_or(true)
    {
        &crate::spoof_user_agent::spoof_user_agent_data_high_entropy_values(
            &crate::spoof_user_agent::build_high_entropy_data(&Some(user_agent)),
        )
    } else {
        &Default::default()
    };

    let linux = agent_os == AgentOs::Linux;

    let mut fingerprint_gpu = false;
    let fingerprint = match config.fingerprint {
        Fingerprint::Basic => true,
        Fingerprint::NativeGPU => {
            fingerprint_gpu = true;
            true
        }
        _ => false,
    };

    let fp_script = if fingerprint {
        let fp_script = if linux {
            if fingerprint_gpu {
                &*FP_JS_GPU_LINUX
            } else {
                &*FP_JS_LINUX
            }
        } else if agent_os == AgentOs::Mac {
            if fingerprint_gpu {
                &*FP_JS_GPU_MAC
            } else {
                &*FP_JS_MAC
            }
        } else if agent_os == AgentOs::Windows {
            if fingerprint_gpu {
                &*FP_JS_GPU_WINDOWS
            } else {
                &*FP_JS_WINDOWS
            }
        } else {
            &*FP_JS
        };
        fp_script
    } else {
        &Default::default()
    };

    let disable_dialogs = if dismiss_dialogs { DISABLE_DIALOGS } else { "" };
    let mut mobile_device = false;

    let screen_spoof = if let Some(viewport) = &viewport {
        mobile_device = viewport.emulating_mobile;
        let dpr = resolve_dpr(
            viewport.emulating_mobile,
            viewport.device_scale_factor,
            agent_os,
        );

        spoof_screen_script_rng(
            viewport.width,
            viewport.height,
            dpr,
            viewport.emulating_mobile,
            &mut rand::rng(),
            agent_os,
        )
    } else {
        Default::default()
    };

    let gpu_profile = gpu_profile.unwrap_or(select_random_gpu_profile(agent_os));

    let st = if config.hardware_concurrency {
        crate::build_stealth_script_with_profile(gpu_profile, config.tier, agent_os)
    } else {
        crate::build_stealth_script_no_concurrency_with_profile(gpu_profile, config.tier, agent_os)
    };

    let touch_screen_script = if config.touch_screen {
        spoof_touch_screen(mobile_device)
    } else {
        Default::default()
    };

    // Final combined script to inject
    let merged_script = if let Some(script) = evaluate_on_new_document.as_deref() {
        if fingerprint {
            let mut b = join_scripts([
                &fp_script,
                &spoof_script,
                disable_dialogs,
                &screen_spoof,
                SPOOF_NOTIFICATIONS,
                SPOOF_PERMISSIONS_QUERY,
                &spoof_media_codecs_script(),
                &touch_screen_script,
                &spoof_media_labels_script(agent_os),
                &spoof_history_length_script(rand::rng().random_range(1..=6)),
                &st,
                &wrap_eval_script(script),
            ]);

            b.push_str(&wrap_eval_script(script));

            Some(b)
        } else {
            let mut b = join_scripts([
                &spoof_script,
                disable_dialogs,
                &screen_spoof,
                SPOOF_NOTIFICATIONS,
                SPOOF_PERMISSIONS_QUERY,
                &spoof_media_codecs_script(),
                &touch_screen_script,
                &spoof_media_labels_script(agent_os),
                &spoof_history_length_script(rand::rng().random_range(1..=6)),
                &st,
                &wrap_eval_script(script),
            ]);
            b.push_str(&wrap_eval_script(script));

            Some(b)
        }
    } else if fingerprint {
        Some(join_scripts([
            &fp_script,
            &spoof_script,
            disable_dialogs,
            &screen_spoof,
            SPOOF_NOTIFICATIONS,
            SPOOF_PERMISSIONS_QUERY,
            &spoof_media_codecs_script(),
            &touch_screen_script,
            &spoof_media_labels_script(agent_os),
            &spoof_history_length_script(rand::rng().random_range(1..=6)),
            &st,
        ]))
    } else if stealth {
        Some(join_scripts([
            &spoof_script,
            disable_dialogs,
            &screen_spoof,
            SPOOF_NOTIFICATIONS,
            SPOOF_PERMISSIONS_QUERY,
            &spoof_media_codecs_script(),
            &touch_screen_script,
            &spoof_media_labels_script(agent_os),
            &spoof_history_length_script(rand::rng().random_range(1..=6)),
            &st,
        ]))
    } else {
        None
    };

    merged_script
}

/// Emulate a real chrome browser.
pub fn emulate(
    user_agent: &str,
    config: &EmulationConfiguration,
    viewport: &Option<&crate::spoof_viewport::Viewport>,
    evaluate_on_new_document: &Option<Box<String>>,
) -> Option<String> {
    emulate_base(user_agent, config, viewport, evaluate_on_new_document, None)
}

/// Emulate a real chrome browser with a gpu profile.
pub fn emulate_with_profile(
    user_agent: &str,
    config: &EmulationConfiguration,
    viewport: &Option<&crate::spoof_viewport::Viewport>,
    evaluate_on_new_document: &Option<Box<String>>,
    gpu_profile: &'static GpuProfile,
) -> Option<String> {
    emulate_base(
        user_agent,
        config,
        viewport,
        evaluate_on_new_document,
        Some(gpu_profile),
    )
}

#[cfg(test)]
mod tests {
    use super::ua_allows_gethighentropy;

    #[test]
    fn ua_green_supported_positive() {
        // Chrome desktop ≥90
        let chrome_win = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
            AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

        // Chrome Android ≥90
        let chrome_android = "Mozilla/5.0 (Linux; Android 11; Pixel 4) \
            AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.5735.61 Mobile Safari/537.36";

        // Edge (Chromium) desktop ≥90
        let edge_win = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
            AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36 Edg/114.0.1823.55";

        // Opera desktop ≥76 (has OPR and Chrome base)
        let opera_win = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
            AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.93 Safari/537.36 OPR/76.0.4017.94";

        // Opera Android ≥64
        let opera_android = "Mozilla/5.0 (Linux; Android 10; SM-G973F) \
            AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Mobile Safari/537.36 OPR/64.0.2254.62069";

        for ua in [
            chrome_win,
            chrome_android,
            edge_win,
            opera_win,
            opera_android,
        ] {
            assert!(ua_allows_gethighentropy(ua), "expected supported: {ua}");
        }
    }

    #[test]
    fn ua_green_supported_negative() {
        // Chrome desktop 89 (below threshold)
        let chrome_89 = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
            AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36";

        // Firefox (no support)
        let firefox = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:118.0) \
            Gecko/20100101 Firefox/118.0";

        // Safari desktop (no Chrome token)
        let safari_mac = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
            AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";

        for ua in [chrome_89, firefox, safari_mac] {
            assert!(
                !ua_allows_gethighentropy(ua),
                "expected NOT supported: {ua}"
            );
        }
    }
}
