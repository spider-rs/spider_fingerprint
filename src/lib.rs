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

use crate::spoofs::{
    PATCH_SPEECH_SYNTHESIS, PLUGIN_AND_MIMETYPE_SPOOF, PLUGIN_AND_MIMETYPE_SPOOF_CHROME,
};

/// The kind of browser.
#[derive(PartialEq, Eq)]
pub enum BrowserKind {
    /// Chrome
    Chrome,
    /// Brave
    Brave,
    /// Firefox
    Firefox,
    /// Safari
    Safari,
    /// Edge
    Edge,
    /// Opera
    Opera,
    /// Other
    Other,
}

impl BrowserKind {
    /// Is the browser chromium based.
    fn is_chromium(&self) -> bool {
        match &self {
            BrowserKind::Chrome | BrowserKind::Opera | BrowserKind::Brave | BrowserKind::Edge => {
                true
            }
            _ => false,
        }
    }
}
const P_EDG: usize = 0; // "edg/"
const P_OPR: usize = 1; // "opr/"
const P_CHR: usize = 2; // "chrome/"
const P_AND: usize = 3; // "android"

lazy_static::lazy_static! {
    /// Common mobile device patterns.
    pub(crate) static ref MOBILE_PATTERNS: [&'static str; 38] = [
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
    pub(crate) static ref MOBILE_MATCHER: aho_corasick::AhoCorasick = aho_corasick::AhoCorasickBuilder::new()
        .ascii_case_insensitive(true)
        .build(MOBILE_PATTERNS.as_ref())
        .expect("failed to compile AhoCorasick patterns");


    /// Allowed ua data for chromium based browsers.
    pub(crate) static ref ALLOWED_UA_DATA: aho_corasick::AhoCorasick = aho_corasick::AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .match_kind(aho_corasick::MatchKind::LeftmostFirst)
            .build(&["edg/", "opr/", "chrome/", "android"])
            .expect("valid device patterns");

    pub(crate) static ref BROWSER_MATCH: aho_corasick::AhoCorasick = aho_corasick::AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .build(&[
                "edg/", "edgios", "edge/",        // Edge
                "opr/", "opera", "opios",         // Opera
                "firefox", "fxios",               // Firefox
                "chrome/", "crios", "chromium",   // Chrome
                "safari",                         // Safari
                "brave",                          // Brave (incase future changes add.)
            ])
            .expect("valid device patterns");


        // Detect Chrome/CriOS first (we only classify Chrome-family UAs).
        static ref CHROME_AC: aho_corasick::AhoCorasick = aho_corasick::AhoCorasickBuilder::new()
                .ascii_case_insensitive(true)
                .build(["Chrome", "CriOS"]) .expect("valid device patterns");

        /// OS patterns. Order doesn’t matter; we store priorities separately.
        static ref OS_PATTERNS:[&'static str; 12] =[
            // iOS family first (iPad/iPhone contain "Mac OS X" too, so give them better priority)
            "iPhone", "iPad", "iOS",
            // Android
            "Android",
            // Windows
            "Windows NT", "Windows", "Win64",
            // Mac
            "Macintosh", "Mac OS X", "Mac",
            // Linux
            "Linux",
            // ChromeOS if you later add an enum variant:
            "CrOS",
        ];

        static ref OS_AC: aho_corasick::AhoCorasick = aho_corasick::AhoCorasickBuilder::new()
                .ascii_case_insensitive(true)
                .build(*OS_PATTERNS)
                .expect("valid device patterns");

        /// Map each pattern index -> (AgentOs, priority). Lower priority wins on ties.
        static ref OS_MAP: [ (AgentOs, u8); 12 ] = [
            (AgentOs::IPhone,  0),
            (AgentOs::IPad,    0),
            (AgentOs::IPhone,  1),
            (AgentOs::Android, 0),
            (AgentOs::Windows, 2),
            (AgentOs::Windows, 3),
            (AgentOs::Windows, 4),
            (AgentOs::Mac,     5),
            (AgentOs::Mac,     6),
            (AgentOs::Mac,     7),
            (AgentOs::Linux,   9),
            (AgentOs::Linux,   8), // CrOS → Linux fallback
        ];

        static ref FF_PATTERNS: [&'static str; 6] = [
            "iPad", "iPhone", "iPod", "Android", "Mobile", "Tablet",
        ];

        static ref FF_AC: aho_corasick::AhoCorasick = aho_corasick::AhoCorasickBuilder::new()
                .ascii_case_insensitive(true)
                .build(*FF_PATTERNS)
                .expect("valid device patterns");
}

#[inline]
fn scan_flags(ua: &str) -> (bool, bool, bool, bool, bool, bool) {
    // (ipad, iphone, ipod, android, mobile, tablet)
    let (mut ipad, mut iphone, mut ipod, mut android, mut mobile, mut tablet) =
        (false, false, false, false, false, false);
    for m in FF_AC.find_iter(ua) {
        match m.pattern().as_u32() {
            0 => ipad = true,
            1 => iphone = true,
            2 => ipod = true,
            3 => android = true,
            4 => mobile = true,
            5 => tablet = true,
            _ => {}
        }
    }
    (ipad, iphone, ipod, android, mobile, tablet)
}

/// Return "?1" (mobile) or "?0" (not mobile).
pub fn detect_is_mobile(ua: &str) -> &'static str {
    let (ipad, iphone, ipod, android, mobile, tablet) = scan_flags(ua);

    // Tablet devices are considered "mobile = true" per your C++ mapping.
    if ipad || iphone || ipod || android || mobile || tablet {
        "?1"
    } else {
        "?0"
    }
}

/// Return the form factor: "Mobile" | "Tablet" | "Desktop".
pub fn detect_form_factor(ua: &str) -> &'static str {
    let (ipad, iphone, ipod, android, mobile, tablet) = scan_flags(ua);

    // Priority:
    // 1) iPad => Tablet (even if "Mobile" token appears)
    if ipad {
        return "Tablet";
    }
    // 2) iPhone/iPod => Mobile
    if iphone || ipod {
        return "Mobile";
    }
    // 3) Android: "Mobile" token => Mobile phone, otherwise Tablet
    if android {
        return if mobile { "Mobile" } else { "Tablet" };
    }
    // 4) Explicit "Tablet" token
    if tablet {
        return "Tablet";
    }
    // 5) Generic "Mobile" token
    if mobile {
        return "Mobile";
    }
    "Desktop"
}

/// Detect the browser type.
pub fn detect_browser(ua: &str) -> &'static str {
    let mut edge = false;
    let mut opera = false;
    let mut firefox = false;
    let mut chrome = false;
    let mut safari = false;
    let mut brave = false;

    for m in BROWSER_MATCH.find_iter(ua) {
        match m.pattern().as_u32() {
            0..=2 => edge = true,    // edg..., edge/
            3..=5 => opera = true,   // opr/opera/opios
            6 | 7 => firefox = true, // firefox/fxios
            8..=10 => chrome = true, // chrome/, crios, chromium
            11 => safari = true,     // safari
            12 => brave = true,      // brave
            _ => (),
        }
    }

    if brave && chrome && !edge && !opera {
        "brave"
    } else if chrome && !edge && !opera {
        "chrome"
    } else if safari && !chrome && !edge && !opera && !firefox {
        "safari"
    } else if edge {
        "edge"
    } else if firefox {
        "firefox"
    } else if opera {
        "opera"
    } else {
        "unknown"
    }
}

/// Detect the browser type to BrowserKind.
pub fn detect_browser_kind(ua: &str) -> BrowserKind {
    let s = detect_browser(ua);

    match s {
        "chrome" => BrowserKind::Chrome,
        "brave" => BrowserKind::Brave,
        "safari" => BrowserKind::Safari,
        "edge" => BrowserKind::Edge,
        "firefox" => BrowserKind::Firefox,
        "opera" => BrowserKind::Opera,
        "unknown" => BrowserKind::Other,
        _ => BrowserKind::Other,
    }
}

#[inline]
/// Parse the major after.
pub fn parse_major_after(s: &str, end_token: usize) -> Option<u32> {
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
        return parse_major_after(ua, end).is_some_and(|v| v >= 90);
    }
    if let Some(end) = endpos[P_OPR] {
        return parse_major_after(ua, end).is_some_and(
            |v| {
                if is_android {
                    v >= 64
                } else {
                    v >= 76
                }
            },
        );
    }
    if let Some(end) = endpos[P_CHR] {
        return parse_major_after(ua, end).is_some_and(|v| v >= 90);
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
        .find(user_agent)
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
    browser: BrowserKind,
) -> String {
    use crate::spoofs::{
        spoof_hardware_concurrency, unified_worker_override, worker_override, HIDE_CHROME,
        HIDE_CONSOLE, HIDE_WEBDRIVER, NAVIGATOR_SCRIPT, REMOVE_CHROME,
    };

    // tmp used for chrome only os checking.
    let chrome = browser.is_chromium() || os != AgentOs::Unknown;

    let spoof_worker = if tier == Tier::BasicNoWorker {
        Default::default()
    } else if concurrency {
        unified_worker_override(
            gpu_profile.hardware_concurrency,
            gpu_profile.webgl_vendor,
            gpu_profile.webgl_renderer,
            !matches!(
                tier,
                |Tier::BasicNoWebglWithGPU| Tier::BasicNoWebglWithGPUNoExtra
                    | Tier::BasicNoWebglWithGPUcWithConsole
            ),
        )
    } else {
        worker_override(gpu_profile.webgl_vendor, gpu_profile.webgl_renderer)
    };

    let spoof_concurrency = spoof_hardware_concurrency(gpu_profile.hardware_concurrency);

    let gpu_limit = GpuLimits::for_os(os).with_variation(gpu_profile.hardware_concurrency);

    let spoof_gpu_adapter = build_gpu_request_adapter_script_from_limits(
        gpu_profile.webgpu_vendor,
        gpu_profile.webgpu_architecture,
        "",
        "",
        &gpu_limit,
    );

    let chrome_spoof = if chrome { HIDE_CHROME } else { REMOVE_CHROME };

    match tier {
        Tier::Basic | Tier::BasicNoWorker | Tier::BasicNoExtra => {
            format!(
                r#"{chrome_spoof}{HIDE_CONSOLE}{spoof_worker}{spoof_gpu_adapter}{NAVIGATOR_SCRIPT}"#
            )
        }
        Tier::BasicWithConsole => {
            format!(
                r#"{chrome_spoof}{spoof_worker}{spoof_concurrency}{spoof_gpu_adapter}{NAVIGATOR_SCRIPT}"#
            )
        }
        Tier::BasicNoWebgl | Tier::BasicNoWebglWithGPU | Tier::BasicNoWebglWithGPUNoExtra => {
            format!(
                r#"{chrome_spoof}{HIDE_CONSOLE}{spoof_worker}{spoof_concurrency}{NAVIGATOR_SCRIPT}"#
            )
        }
        Tier::BasicNoWebglWithGPUcWithConsole => {
            format!(r#"{chrome_spoof}{spoof_worker}{spoof_concurrency}{NAVIGATOR_SCRIPT}"#)
        }
        Tier::HideOnly => {
            format!(r#"{chrome_spoof}{HIDE_CONSOLE}{HIDE_WEBDRIVER}"#)
        }
        Tier::HideOnlyWithConsole => {
            format!(r#"{chrome_spoof}{HIDE_WEBDRIVER}"#)
        }
        Tier::HideOnlyChrome => chrome_spoof.into(),
        Tier::Low => {
            format!(
                r#"{chrome_spoof}{HIDE_CONSOLE}{spoof_worker}{spoof_concurrency}{spoof_gpu_adapter}{HIDE_WEBDRIVER}"#
            )
        }
        Tier::LowWithPlugins => {
            format!(
                r#"{chrome_spoof}{HIDE_CONSOLE}{spoof_worker}{spoof_concurrency}{spoof_gpu_adapter}{HIDE_WEBDRIVER}"#
            )
        }
        Tier::LowWithNavigator => {
            format!(
                r#"{chrome_spoof}{HIDE_CONSOLE}{spoof_worker}{spoof_concurrency}{spoof_gpu_adapter}{HIDE_WEBDRIVER}{NAVIGATOR_SCRIPT}"#
            )
        }
        Tier::Mid => {
            format!(
                r#"{chrome_spoof}{HIDE_CONSOLE}{spoof_worker}{spoof_concurrency}{spoof_gpu_adapter}{NAVIGATOR_SCRIPT}{HIDE_WEBDRIVER}"#
            )
        }
        Tier::Full => {
            let spoof_gpu = build_gpu_spoof_script_wgsl(gpu_profile.canvas_format);

            format!("{chrome_spoof}{HIDE_CONSOLE}{spoof_worker}{spoof_concurrency}{spoof_gpu_adapter}{HIDE_WEBDRIVER}{NAVIGATOR_SCRIPT}{spoof_gpu}")
        }
        _ => Default::default(),
    }
}

/// Generate the initial stealth script to send in one command.
pub fn build_stealth_script(tier: Tier, os: AgentOs) -> String {
    let gpu_profile = select_random_gpu_profile(os);
    build_stealth_script_base(gpu_profile, tier, os, true, BrowserKind::Other)
}

/// Generate the initial stealth script to send in one command without hardware concurrency.
pub fn build_stealth_script_no_concurrency(tier: Tier, os: AgentOs) -> String {
    let gpu_profile = select_random_gpu_profile(os);
    build_stealth_script_base(gpu_profile, tier, os, false, BrowserKind::Other)
}

/// Generate the initial stealth script to send in one command and profile.
pub fn build_stealth_script_with_profile(
    gpu_profile: &'static GpuProfile,
    tier: Tier,
    os: AgentOs,
) -> String {
    build_stealth_script_base(gpu_profile, tier, os, true, BrowserKind::Other)
}

/// Generate the initial stealth script to send in one command and profile.
pub fn build_stealth_script_with_profile_and_browser(
    gpu_profile: &'static GpuProfile,
    tier: Tier,
    os: AgentOs,
    browser: BrowserKind,
) -> String {
    build_stealth_script_base(gpu_profile, tier, os, true, browser)
}

/// Generate the initial stealth script to send in one command without hardware concurrency and profile.
pub fn build_stealth_script_no_concurrency_with_profile_and_browser(
    gpu_profile: &'static GpuProfile,
    tier: Tier,
    os: AgentOs,
    browser: BrowserKind,
) -> String {
    build_stealth_script_base(gpu_profile, tier, os, false, browser)
}

/// Generate the initial stealth script to send in one command without hardware concurrency and profile.
pub fn build_stealth_script_no_concurrency_with_profile(
    gpu_profile: &'static GpuProfile,
    tier: Tier,
    os: AgentOs,
) -> String {
    build_stealth_script_base(gpu_profile, tier, os, false, BrowserKind::Other)
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
        matches!(self, Self::Basic | Self::NativeGPU)
    }
}
/// Configuration options for browser fingerprinting and automation.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EmulationConfiguration {
    /// Enables stealth mode to help avoid detection by anti-bot mechanisms.
    pub tier: configs::Tier,
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
    /// If enabled, will auto-dismiss browser popups and dialogs.
    pub dismiss_dialogs: bool,
    /// Disable notification emulation.
    pub disable_notifications: bool,
    /// Disable permissions emulation.
    pub disable_permissions: bool,
    /// Disable media codecs emulation.
    pub disable_media_codecs: bool,
    /// Disable speech syntheses.
    pub disable_speech_syntheses: bool,
    /// Disable media labels.
    pub disable_media_labels: bool,
    /// Disable navigator history length
    pub disable_history_length: bool,
    /// Disable the user agent data emulation - extra guard for user_agent_data.
    pub disable_user_agent_data: bool,
    /// Disable screen emulation.
    pub disable_screen: bool,
    /// Disable touch screen emulation.
    pub disable_touch_screen: bool,
    /// Disable the plugins spoof.
    pub disable_plugins: bool,
    /// Disable the stealth emulation.
    pub disable_stealth: bool,
}

/// Fast Chrome-only OS detection using Aho-Corasick (ASCII case-insensitive).
pub fn get_agent_os(user_agent: &str) -> AgentOs {
    if !CHROME_AC.is_match(user_agent) {
        return AgentOs::Unknown;
    }
    let mut best: Option<(u8, usize, AgentOs)> = None;
    for m in OS_AC.find_iter(user_agent) {
        let (os, pri) = OS_MAP[m.pattern()];
        let cand = (pri, m.len(), os);
        best = match best {
            None => Some(cand),
            Some(cur) => {
                if cand.0 < cur.0 || (cand.0 == cur.0 && cand.1 > cur.1) {
                    Some(cand)
                } else {
                    Some(cur)
                }
            }
        };
    }
    best.map(|t| t.2).unwrap_or(AgentOs::Unknown)
}

/// Agent Operating system to string
pub fn agent_os_strings(os: AgentOs) -> &'static str {
    match os {
        AgentOs::Android => "Android",
        AgentOs::IPhone | AgentOs::IPad => "iOS",
        AgentOs::Mac => "macOS",
        AgentOs::Windows => "Windows",
        AgentOs::Linux => "Linux",
        AgentOs::ChromeOS => "Chrome OS",
        AgentOs::Unknown => "Unknown",
    }
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
        emulation_config.disable_notifications = true; // fix
        emulation_config.disable_media_codecs = true; // fix
        emulation_config.disable_plugins = true; // fix

        emulation_config
    }
}

/// Join the scrips pre-allocated.
pub fn join_scripts<I: IntoIterator<Item = impl AsRef<str>>>(parts: I) -> String {
    let mut script = String::with_capacity(4096);
    for part in parts {
        script.push_str(part.as_ref());
    }
    script
}

/// Join the scrips pre-allocated.
pub fn join_scripts_with_capacity<I: IntoIterator<Item = impl AsRef<str>>>(
    parts: I,
    capacity: usize,
) -> String {
    let mut script = String::with_capacity(capacity);
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
    let agent_os = if config.agent_os == AgentOs::Unknown {
        get_agent_os(user_agent)
    } else {
        config.agent_os
    };
    let spoof_user_agent_data = if stealth
        && ua_allows_gethighentropy(user_agent)
        && config.user_agent_data.unwrap_or(true)
    {
        &crate::spoof_user_agent::spoof_user_agent_data_high_entropy_values(
            &crate::spoof_user_agent::build_high_entropy_data(&Some(user_agent)),
        )
    } else {
        &Default::default()
    };
    let spoof_speech_syn = if stealth && agent_os != AgentOs::Unknown {
        PATCH_SPEECH_SYNTHESIS
    } else {
        Default::default()
    };
    let linux = agent_os == AgentOs::Linux;

    let no_extra =
        config.tier == Tier::BasicNoExtra || config.tier == Tier::BasicNoWebglWithGPUNoExtra;

    let (fingerprint, fingerprint_gpu) = match config.fingerprint {
        Fingerprint::Basic => (true, false),
        Fingerprint::NativeGPU => (true, true),
        _ => (false, false),
    };

    let fp_script = if fingerprint {
        if linux {
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
        }
    } else {
        &Default::default()
    };

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
    let browser_kind = detect_browser_kind(user_agent);

    let plugin_spoof = if browser_kind == BrowserKind::Chrome {
        PLUGIN_AND_MIMETYPE_SPOOF_CHROME
    } else {
        PLUGIN_AND_MIMETYPE_SPOOF
    };

    let st = if config.hardware_concurrency {
        crate::build_stealth_script_with_profile_and_browser(
            gpu_profile,
            config.tier,
            agent_os,
            browser_kind,
        )
    } else {
        crate::build_stealth_script_no_concurrency_with_profile_and_browser(
            gpu_profile,
            config.tier,
            agent_os,
            browser_kind,
        )
    };

    let touch_screen_script = if config.touch_screen {
        spoof_touch_screen(mobile_device)
    } else {
        Default::default()
    };

    let eval_script = if let Some(script) = evaluate_on_new_document.as_deref() {
        wrap_eval_script(script)
    } else {
        Default::default()
    };

    let stealth_scripts = if stealth {
        join_scripts([
            if no_extra || config.disable_speech_syntheses {
                Default::default()
            } else {
                spoof_speech_syn
            },
            if no_extra || config.disable_user_agent_data {
                Default::default()
            } else {
                spoof_user_agent_data
            },
            if no_extra || config.dismiss_dialogs {
                DISABLE_DIALOGS
            } else {
                ""
            },
            if no_extra || config.disable_screen {
                Default::default()
            } else {
                &screen_spoof
            },
            if no_extra || config.disable_notifications {
                Default::default()
            } else {
                SPOOF_NOTIFICATIONS
            },
            if no_extra || config.disable_permissions {
                Default::default()
            } else {
                SPOOF_PERMISSIONS_QUERY
            },
            if no_extra || config.disable_media_codecs {
                Default::default()
            } else {
                spoof_media_codecs_script()
            },
            if no_extra || config.disable_touch_screen {
                Default::default()
            } else {
                touch_screen_script
            },
            &if no_extra || config.disable_media_labels {
                Default::default()
            } else {
                spoof_media_labels_script(agent_os)
            },
            &if no_extra || config.disable_history_length {
                Default::default()
            } else {
                spoof_history_length_script(rand::rng().random_range(1..=6))
            },
            &if no_extra || config.disable_plugins && config.tier != Tier::LowWithPlugins {
                Default::default()
            } else {
                plugin_spoof
            },
            &if config.disable_stealth {
                Default::default()
            } else {
                st
            },
        ])
    } else {
        Default::default()
    };

    // Final combined script to inject
    if stealth || fingerprint {
        Some(join_scripts_with_capacity(
            [fp_script, &stealth_scripts, &eval_script],
            fp_script.capacity() + stealth_scripts.capacity() + eval_script.capacity(),
        ))
    } else if !eval_script.is_empty() {
        Some(eval_script)
    } else {
        None
    }
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
    use super::{
        detect_form_factor, detect_is_mobile, emulate, get_agent_os, ua_allows_gethighentropy,
        AgentOs, EmulationConfiguration,
    };

    #[test]
    fn emulation() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";
        let config = EmulationConfiguration::default();
        std::env::set_var("CHROME_VERSION_FULL", "139.0.7258.67");
        let data = emulate(ua, &config, &None, &None);
        assert!(data.is_some())
    }

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

    #[test]
    fn detects_agent_os_across_platforms() {
        let cases: &[(&str, AgentOs)] = &[
            // Windows (Chrome)
            ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
             AgentOs::Windows),

            // macOS (Chrome)
            ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
             AgentOs::Mac),

            // Linux (Chrome)
            ("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
             AgentOs::Linux),

            // Android (Chrome)
            ("Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36",
             AgentOs::Android),

            // iPhone (Chrome on iOS uses CriOS)
            ("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/124.0.0.0 Mobile/15E148 Safari/604.1",
             AgentOs::IPhone),

            // iPad (CriOS) — should still resolve to iOS
            ("Mozilla/5.0 (iPad; CPU OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/123.0.0.0 Mobile/15E148 Safari/604.1",
             AgentOs::IPad),

            // Edge (Chromium) still contains Chrome token -> Windows
            ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0",
             AgentOs::Windows),

            // Mixed case (should be matched case-insensitively) -> Linux
            ("mozilla/5.0 (x11; linux x86_64) applewebkit/537.36 (khtml, like gecko) chrome/120.0.0.0 safari/537.36",
             AgentOs::Linux),

            // Non-Chrome (Firefox) -> Unknown due to Chrome/CriOS gate
            ("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/124.0",
             AgentOs::Unknown),

            // Not a browser UA
            ("curl/8.0.1",
             AgentOs::Unknown),
        ];

        for (ua, expected) in cases {
            let got = get_agent_os(ua);
            assert_eq!(got, *expected, "UA: {}", ua);
        }
    }

    #[test]
    fn prioritizes_ios_over_mac_tokens() {
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/124.0.0.0 Mobile/15E148 Safari/604.1";
        assert_eq!(get_agent_os(ua), AgentOs::IPhone);
    }

    #[test]
    fn android_phone() {
        let ua = "Mozilla/5.0 (Linux; Android 13; Pixel 7) ... Mobile Safari/537.36";
        assert_eq!(detect_is_mobile(ua), "?1");
        assert_eq!(detect_form_factor(ua), "Mobile");
    }

    #[test]
    fn android_tablet() {
        let ua = "Mozilla/5.0 (Linux; Android 12; SM-T970) ... Safari/537.36";
        assert_eq!(detect_is_mobile(ua), "?1");
        assert_eq!(detect_form_factor(ua), "Tablet");
    }

    #[test]
    fn iphone_and_ipad() {
        let iphone = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 ...) CriOS/124.0.0.0 Mobile/15E148";
        assert_eq!(detect_is_mobile(iphone), "?1");
        assert_eq!(detect_form_factor(iphone), "Mobile");

        let ipad = "Mozilla/5.0 (iPad; CPU OS 16_6 ...) CriOS/123.0.0.0 Mobile/15E148";
        assert_eq!(detect_is_mobile(ipad), "?1");
        assert_eq!(detect_form_factor(ipad), "Tablet");
    }

    #[test]
    fn desktop_linux() {
        let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 ... Chrome/124 Safari/537.36";
        assert_eq!(detect_is_mobile(ua), "?0");
        assert_eq!(detect_form_factor(ua), "Desktop");
    }
}
