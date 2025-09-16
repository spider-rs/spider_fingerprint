/// Tier of stealth to use.
#[derive(PartialEq, Debug, Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tier {
    #[default]
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
    /// Mid spoofing.
    Mid,
    /// Full spoofing.
    Full,
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
    /// Iphone
    IPhone,
    /// Windows.
    Windows,
    /// Android.
    Android,
    #[default]
    /// Unknown.
    Unknown,
}
