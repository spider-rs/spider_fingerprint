use super::gpu_profile::GpuProfile;

pub static GPU_PROFILES_MAC: &[GpuProfile] = &[
    // Apple M1 (MacBook Air/Pro base models)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M1, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 8,
    },
    // Apple M1 Pro (MacBook Pro 14/16-inch)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M1 Pro, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 10,
    },
    // Apple M1 Max
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M1 Max, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 10,
    },
    // Apple M1 Ultra (Mac Studio)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M1 Ultra, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 20,
    },
    // Apple M2 (MacBook Air, Mac mini)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M2, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 8,
    },
    // Apple M2 Pro (MacBook Pro, Mac mini)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M2 Pro, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 12,
    },
    // Apple M2 Max (MacBook Pro, Mac Studio)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M2 Max, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 16,
    },
    // Apple M2 Ultra (Mac Studio, Mac Pro)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M2 Ultra, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 24,
    },
    // Apple M3 (base model)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M3, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 8,
    },
    // Apple M3 Pro (11-core or 12-core)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M3 Pro, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 12,
    },
    // Apple M3 Max
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M3 Max, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 16,
    },
    // Apple M4 (base model)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M4, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 8,
    },
    // Apple M4 Pro
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M4 Pro, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 12,
    },
    // Apple M4 Max
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M4 Max, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 16,
    },
    // Apple M4 Ultra (e.g., Mac Studio / Mac Pro)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M4 Ultra, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 24,
    },
    // Apple M5 (base model)
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M5, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 10,
    },
    // Apple M5 Pro
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M5 Pro, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 12,
    },
    // Apple M5 Max
    GpuProfile {
        webgl_vendor: "Google Inc. (Apple)",
        webgl_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M5 Max, Unspecified Version)",
        webgpu_vendor: "apple",
        webgpu_architecture: "metal-3",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 16,
    },
    // ── Intel Macs (still ~15% of macOS users) ──
    GpuProfile {
        webgl_vendor: "Google Inc. (Intel Inc.)",
        webgl_renderer: "ANGLE (Intel Inc., Intel(R) Iris(TM) Plus Graphics 640, OpenGL 4.1)",
        webgpu_vendor: "intel",
        webgpu_architecture: "",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 4,
    },
    GpuProfile {
        webgl_vendor: "Google Inc. (Intel Inc.)",
        webgl_renderer: "ANGLE (Intel Inc., Intel(R) Iris(TM) Plus Graphics 655, OpenGL 4.1)",
        webgpu_vendor: "intel",
        webgpu_architecture: "",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 4,
    },
    GpuProfile {
        webgl_vendor: "Google Inc. (Intel Inc.)",
        webgl_renderer: "ANGLE (Intel Inc., Intel(R) UHD Graphics 630, OpenGL 4.1)",
        webgpu_vendor: "intel",
        webgpu_architecture: "",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 6,
    },
    GpuProfile {
        webgl_vendor: "Google Inc. (Intel Inc.)",
        webgl_renderer: "ANGLE (Intel Inc., Intel(R) UHD Graphics 617, OpenGL 4.1)",
        webgpu_vendor: "intel",
        webgpu_architecture: "",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 4,
    },
    // Intel Macs with discrete AMD GPUs (MacBook Pro 15/16-inch 2018-2020)
    GpuProfile {
        webgl_vendor: "Google Inc. (ATI Technologies Inc.)",
        webgl_renderer: "ANGLE (ATI Technologies Inc., AMD Radeon Pro 5500M, OpenGL 4.1)",
        webgpu_vendor: "amd",
        webgpu_architecture: "",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 12,
    },
    GpuProfile {
        webgl_vendor: "Google Inc. (ATI Technologies Inc.)",
        webgl_renderer: "ANGLE (ATI Technologies Inc., AMD Radeon Pro 5300M, OpenGL 4.1)",
        webgpu_vendor: "amd",
        webgpu_architecture: "",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 8,
    },
    GpuProfile {
        webgl_vendor: "Google Inc. (ATI Technologies Inc.)",
        webgl_renderer: "ANGLE (ATI Technologies Inc., AMD Radeon Pro 560X, OpenGL 4.1)",
        webgpu_vendor: "amd",
        webgpu_architecture: "",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 6,
    },
    GpuProfile {
        webgl_vendor: "Google Inc. (ATI Technologies Inc.)",
        webgl_renderer: "ANGLE (ATI Technologies Inc., AMD Radeon Pro Vega 20, OpenGL 4.1)",
        webgpu_vendor: "amd",
        webgpu_architecture: "",
        canvas_format: "bgra8unorm",
        hardware_concurrency: 8,
    },
];
