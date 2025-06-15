use super::gpu_profile::GpuProfile;

pub static GPU_PROFILES_IPHONE: &[GpuProfile] = &[
    // iPhone 13 / 13 Pro / 13 Pro Max (Apple A15 Bionic)
    GpuProfile {
        webgl_vendor: "Apple Inc.",
        webgl_renderer: "Apple A15 GPU",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-2",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 6,
    },
    // iPhone 14 / 14 Plus (Apple A15 Bionic, reused)
    GpuProfile {
        webgl_vendor: "Apple Inc.",
        webgl_renderer: "Apple A15 GPU",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-2",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 6,
    },
    // iPhone 14 Pro / Pro Max (Apple A16 Bionic)
    GpuProfile {
        webgl_vendor: "Apple Inc.",
        webgl_renderer: "Apple A16 GPU",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 6,
    },
    // iPhone 15 / 15 Plus (Apple A16 Bionic)
    GpuProfile {
        webgl_vendor: "Apple Inc.",
        webgl_renderer: "Apple A16 GPU",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 6,
    },
    // iPhone 15 Pro / Pro Max (Apple A17 Pro)
    GpuProfile {
        webgl_vendor: "Apple Inc.",
        webgl_renderer: "Apple A17 Pro GPU",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 6,
    },
];
