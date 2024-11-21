use std::ffi::CStr;

// Vulkan related
use ash::vk;

// OpenGL related
// use glutin::{
//     context::{NotCurrentContext, PossiblyCurrentContext},
//     display::GetGlDisplay,
//     prelude::*,
//     surface::{Surface, WindowSurface},
// };
// use raw_window_handle::HasRawWindowHandle;
// use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to perform Vulkan operation: {0}")]
    VulkanOperationFailed(String),
    #[error("Vulkan is not supported on this platform")]
    VulkanNotSupported,
    #[error("Failed to create OpenGL context")]
    OpenGLContextCreationFailed,
    #[error("Failed to query GPU info")]
    OpenGLQueryFailed,
}

impl Error {
    pub fn is_vulkan_not_supported(&self) -> bool {
        matches!(self, Error::VulkanNotSupported)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GPUKind {
    Integrated,
    Discrete,
    Virtual,
    CPU,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct GPU {
    pub kind: GPUKind,
    pub name: String,
    pub vendor: String,
    pub driver_version: String,
    /// 0 is means unknown or not available
    pub vram: u64,
    // pub max_resolution: Resolution,
    // pub current_resolution: Resolution,
    pub clock_speed: Option<u32>,
    pub temperature: Option<u32>,
}

pub fn retrieve_gpu_info_via_vk() -> Result<Vec<GPU>, Error> {
    let entry = unsafe { ash::Entry::load() }.map_err(|_| Error::VulkanNotSupported)?;
    let app_name = c"GPUInfoApp";
    let app_info = vk::ApplicationInfo::default()
        .application_name(app_name)
        .application_version(0)
        .engine_name(app_name)
        .engine_version(0)
        .api_version(vk::API_VERSION_1_0);

    let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);
    let instance = unsafe { entry.create_instance(&create_info, None) }
        .map_err(|e| Error::VulkanOperationFailed(e.to_string()))?;

    let physical_devices = unsafe { instance.enumerate_physical_devices() }
        .map_err(|e| Error::VulkanOperationFailed(e.to_string()))?;

    if physical_devices.is_empty() {
        return Err(Error::VulkanOperationFailed(
            "No Vulkan-compatible GPUs found.".to_string(),
        ));
    }

    let mut gpus = Vec::new();

    for device in physical_devices {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(device) };

        // Extract GPU properties
        let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_str()
            .unwrap_or("Unknown")
            .to_string();

        let vendor_id = properties.vendor_id;
        let vendor_name = match vendor_id {
            0x8086 => "Intel",
            0x10DE => "NVIDIA",
            0x1002 => "AMD",
            _ => "Unknown",
        }
        .to_string();

        let driver_version = format!(
            "{}.{}.{}",
            (properties.driver_version >> 22) & 0x3FF,
            (properties.driver_version >> 12) & 0x3FF,
            properties.driver_version & 0xFFF
        );

        let device_type = match properties.device_type {
            vk::PhysicalDeviceType::INTEGRATED_GPU => GPUKind::Integrated,
            vk::PhysicalDeviceType::DISCRETE_GPU => GPUKind::Discrete,
            vk::PhysicalDeviceType::VIRTUAL_GPU => GPUKind::Virtual,
            vk::PhysicalDeviceType::CPU => GPUKind::CPU,
            _ => GPUKind::Unknown,
        };

        let vram_size = memory_properties
            .memory_heaps
            .iter()
            .take(memory_properties.memory_heap_count as usize)
            .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|heap| heap.size)
            .sum::<u64>();

        // Populate GPU struct
        let gpu = GPU {
            kind: device_type,
            name: device_name,
            vendor: vendor_name,
            driver_version,
            vram: vram_size / (1024 * 1024), // Convert to MB
            clock_speed: None,               // Vulkan does not provide clock speed
            temperature: None,               // Vulkan does not provide temperature natively
        };

        gpus.push(gpu);
    }

    Ok(gpus)
}

// pub fn retrieve_gpu_info_via_gl() -> Result<Vec<GPU>, Error> {
//     // Create a headless context
//     let event_loop = winit::event_loop::EventLoop::new();
//     let window_builder = winit::window::WindowAttributes::default()
//         .with_visible(false)
//         .with_inner_size(winit::dpi::LogicalSize::new(1, 1));

//     let template = glutin::config::ConfigTemplateBuilder::new()
//         .with_transparency(true)
//         .with_float_pixels(true)
//         .build();
//     let config = glutin::display::

//     let context = unsafe {
//         gl_config
//             .display()
//             .create_context(&gl_config, &window.raw_window_handle())
//             .map_err(|_| Error::OpenGLContextCreationFailed)?
//     };

//     let surface = Surface::new(&gl_config.display(), window.inner_size().into(), &window);

//     let context = context
//         .make_current(&surface)
//         .map_err(|_| Error::OpenGLContextCreationFailed)?;

//     // Load OpenGL functions
//     gl::load_with(|s| context.get_proc_address(s));

//     // Query GPU information
//     let vendor = unsafe {
//         let data = gl::GetString(gl::VENDOR);
//         std::ffi::CStr::from_ptr(data as *const i8)
//             .to_string_lossy()
//             .into_owned()
//     };

//     let renderer = unsafe {
//         let data = gl::GetString(gl::RENDERER);
//         std::ffi::CStr::from_ptr(data as *const i8)
//             .to_string_lossy()
//             .into_owned()
//     };

//     let version = unsafe {
//         let data = gl::GetString(gl::VERSION);
//         std::ffi::CStr::from_ptr(data as *const i8)
//             .to_string_lossy()
//             .into_owned()
//     };

//     // Try to determine GPU kind (this is a rough estimate)
//     let kind = if renderer.to_lowercase().contains("intel") {
//         GPUKind::Integrated
//     } else if renderer.to_lowercase().contains("nvidia") || renderer.to_lowercase().contains("amd")
//     {
//         GPUKind::Discrete
//     } else {
//         GPUKind::Unknown
//     };

//     // Try to get VRAM info (this is not standardized and may not work on all GPUs)
//     let vram = unsafe {
//         let mut vram_size = 0;
//         gl::GetIntegerv(
//             gl::GPU_MEMORY_INFO_TOTAL_AVAILABLE_MEMORY_NVX,
//             &mut vram_size,
//         );
//         if vram_size > 0 {
//             (vram_size as u64) * 1024 // Convert KB to bytes
//         } else {
//             0
//         }
//     };

//     let gpu = GPU {
//         kind,
//         name: renderer,
//         vendor,
//         driver_version: version,
//         vram,
//         clock_speed: None,
//         temperature: None,
//     };

//     Ok(vec![gpu])
// }

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_retrieve_gpu_info_via_vk() {
        let result = retrieve_gpu_info_via_vk();
        eprintln!("{:#?}", result);
        assert!(match result {
            Ok(gpus) => !gpus.is_empty(),
            Err(e) => e.is_vulkan_not_supported(),
        });
    }
}
