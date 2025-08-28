pub use super::gpu_android::GPU_PROFILES_ANDROID;
pub use super::gpu_iphone::GPU_PROFILES_IPHONE;
pub use super::gpu_linux::GPU_PROFILES_LINUX;
pub use super::gpu_mac::GPU_PROFILES_MAC;
pub use super::gpu_profile::GpuProfile;
pub use super::gpu_windows::GPU_PROFILES_WINDOWS;

use crate::AgentOs;
use rand::prelude::IndexedRandom;

/// Fallback GPU profile used when no valid match is found.
pub static FALLBACK_GPU_PROFILE: GpuProfile = GpuProfile {
    webgl_vendor: "Google Inc.",
    webgl_renderer: "ANGLE (Unknown, Generic Renderer, OpenGL)",
    webgpu_vendor: "Google Inc. (NVIDIA)",
    webgpu_architecture: "",
    canvas_format: "rgba8unorm",
    hardware_concurrency: 10,
};

/// Select a random GPU profile.
pub fn select_random_gpu_profile(os: crate::AgentOs) -> &'static GpuProfile {
    match os {
        AgentOs::Mac => GPU_PROFILES_MAC
            .choose(&mut rand::rng())
            .unwrap_or(&FALLBACK_GPU_PROFILE),
        AgentOs::IPhone => GPU_PROFILES_IPHONE
            .choose(&mut rand::rng())
            .unwrap_or(&FALLBACK_GPU_PROFILE),
        AgentOs::Windows => GPU_PROFILES_WINDOWS
            .choose(&mut rand::rng())
            .unwrap_or(&FALLBACK_GPU_PROFILE),
        AgentOs::Linux | AgentOs::Unknown => GPU_PROFILES_LINUX
            .choose(&mut rand::rng())
            .unwrap_or(&FALLBACK_GPU_PROFILE),
        AgentOs::Android => GPU_PROFILES_ANDROID
            .choose(&mut rand::rng())
            .unwrap_or(&FALLBACK_GPU_PROFILE),
    }
}