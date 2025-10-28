/// Tier of stealth to use.
#[derive(PartialEq, Debug, Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tier {
    /// Basic spoofing.
    Basic,
    /// Basic spoofing with console.
    BasicWithConsole,
    /// Basic spoofing without webgl.
    BasicNoWebgl,
    /// Basic spoofing without webgl only gpu.
    BasicNoWebglWithGPU,
    /// Basic spoofing without webgl only gpu and console.
    BasicNoWebglWithGPUcWithConsole,
    /// Basic without unified worker.
    BasicNoWorker,
    /// Hide only the main differences.
    HideOnly,
    /// Hide only the main differences with the console enabled.
    HideOnlyWithConsole,
    /// Hide only window.chrome.
    HideOnlyChrome,
    /// Low spoofing.
    Low,
    /// Low spoofing with plugins.
    LowWithPlugins,
    #[default]
    /// Low spoofing with navigator.
    LowWithNavigator,
    /// Mid spoofing.
    Mid,
    /// Full spoofing.
    Full,
    /// Basic spoofing without extra emulation.
    BasicNoExtra,
    /// Basic spoofing without webgl only gpu and no extra.
    BasicNoWebglWithGPUNoExtra,
    /// Extra only.
    Extra,
    /// No spoofing
    None,
}

impl Tier {
    /// Stealth mode enabled.
    pub fn stealth(&self) -> bool {
        match &self {
            Tier::None => false,
            _ => true,
        }
    }
}

/// The user agent type of profiles.
#[derive(PartialEq, Clone, Copy, Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AgentOs {
    /// Linux.
    Linux,
    /// Mac.
    Mac,
    /// IPhone.
    IPhone,
    /// Ipad.
    IPad,
    /// Windows.
    Windows,
    /// Android.
    Android,
    /// Chrome OS
    ChromeOS,
    #[default]
    /// Unknown.
    Unknown,
}

impl AgentOs {
    /// Agent Operating system to string
    pub fn agent_os_string(&self) -> &'static str {
        match &self {
            AgentOs::Android => "Android",
            AgentOs::IPhone | AgentOs::IPad => "iOS",
            AgentOs::Mac => "macOS",
            AgentOs::Windows => "Windows",
            AgentOs::Linux => "Linux",
            AgentOs::ChromeOS => "Chrome OS",
            AgentOs::Unknown => "Unknown",
        }
    }
}
