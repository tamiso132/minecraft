use std::{
    borrow::Cow,
    ffi::{self, CStr, CString},
};

use ash::{
    ext::debug_utils,
    khr::swapchain,
    vk::{self, ApplicationInfo, MemoryPropertyFlags, Queue, QueueFlags},
    Entry,
};
use vk_mem::{Alloc, AllocationCreateInfo, AllocatorCreateInfo};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::init;
// everything that is not a builder, will be moved later from here
pub struct TKQueue {
    queue: vk::Queue,
    family: u32,
}

impl TKQueue {
    pub fn find_queue(
        instance: ash::Instance,
        physical: vk::PhysicalDevice,
        queue_flag: QueueFlags,
    ) -> Option<Self> {
        unsafe {
            let queues = instance.get_physical_device_queue_family_properties(physical);
            let mut queue: Option<TKQueue> = None;
            let _ = queues.iter().enumerate().map(|(index, queue_info)| {
                if queue_info.queue_flags.contains(queue_flag) {
                    let tk_queue = TKQueue {
                        queue: vk::Queue::null(),
                        family: index as u32,
                    };
                    queue = Some(tk_queue);
                }
            });
            queue
        }
    }
    pub fn find_transfer_only(
        instance: ash::Instance,
        physical: vk::PhysicalDevice,
    ) -> Option<Self> {
        let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
        let mut transfer_queue: Option<TKQueue> = None;
        let _ = queues.iter().enumerate().map(|(index, queue_info)| {
            if queue_info.queue_flags != QueueFlags::GRAPHICS
                && queue_info.queue_flags == QueueFlags::TRANSFER
            {
                transfer_queue = Some(TKQueue {
                    queue: vk::Queue::null(),
                    family: index as u32,
                });
            }
        });
        transfer_queue
    }
}

// specific implementation

struct DeviceHelper {}

pub struct DeviceBuilder<'a> {
    features: vk::PhysicalDeviceFeatures,
    features_13: vk::PhysicalDeviceVulkan13Features<'a>,
    extensions: Vec<CString>,
    physical: vk::PhysicalDevice,
    instance: ash::Instance,

    transfer_queue: TKQueue,
    graphic_queue: TKQueue,
}

impl<'a> DeviceBuilder<'a> {
    pub fn new(instance: ash::Instance) -> Self {
        let features = vk::PhysicalDeviceFeatures::default();
        let features_13 = vk::PhysicalDeviceVulkan13Features::default();
        let extensions = Vec::new();
        let physical = vk::PhysicalDevice::null();

        let transfer_queue = TKQueue {
            queue: Queue::default(),
            family: 0,
        };

        let graphic_queue = TKQueue {
            queue: Queue::default(),
            family: 0,
        };

        Self {
            features,
            features_13,
            extensions,
            physical,
            transfer_queue,
            graphic_queue,
            instance,
        }
    }

    pub fn select_physical_device(mut self) -> Self {
        let has_queues_required: bool = false;

        unsafe {
            let physical_devices = self
                .instance
                .clone()
                .enumerate_physical_devices()
                .expect("no vulkan supported gpu");

            for physical in physical_devices {
                let graphic =
                    TKQueue::find_queue(self.instance.clone(), physical, QueueFlags::GRAPHICS);
                let transfer = TKQueue::find_transfer_only(self.instance.clone(), physical);

                if graphic.is_some() && transfer.is_some() {
                    self.transfer_queue = transfer.unwrap();
                    self.graphic_queue = graphic.unwrap();
                    break;
                }
            }

            if !has_queues_required {
                panic!("None of the Vulkan supported gpus have the required queues");
            }
        }
        self
    }

    pub fn ext_image_cube_array(mut self) -> Self {
        self.features.image_cube_array = 1;
        self
    }

    pub fn ext_sampler_anisotropy(mut self) -> Self {
        self.features.sampler_anisotropy = 1;
        self
    }

    pub fn ext_dynamic_rendering(mut self) -> Self {
        self.features_13.dynamic_rendering = 1;
        self.extensions
            .push(CString::new("VK_KHR_dynamic_rendering").unwrap());
        self
    }

    pub fn build(mut self) -> (ash::Device, vk::PhysicalDevice) {
        let raw_ext: Vec<*const i8> = self.extensions.iter().map(|raw| raw.as_ptr()).collect();

        let info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&raw_ext)
            .enabled_features(&self.features)
            .push_next(&mut self.features_13);

        unsafe {
            let device = self
                .instance
                .create_device(self.physical, &info, None)
                .expect("failed created a logical device");

            (device, self.physical)
        }
    }
}

pub struct InstanceBuilder<'a> {
    app_name: CString,
    entry: ash::Entry,
    application_info: ApplicationInfo<'a>,
    extensions: Vec<CString>,
    layers: Vec<CString>,
    debug_util_info: Option<vk::DebugUtilsMessengerCreateInfoEXT<'a>>,

    debug: bool,
}

impl<'a> InstanceBuilder<'a> {
    const ENGINE_NAME: &'static str = "TamisoEngine";

    pub fn new() -> Self {
        unsafe {
            let app_name = CString::new("").unwrap();
            let entry = ash::Entry::load().unwrap();

            let application_info = ApplicationInfo::default();
            let extensions = vec![];
            let layers = vec![];
            let debug_util_info = None;

            Self {
                app_name,
                entry,
                extensions,
                layers,
                debug_util_info,
                application_info,
                debug: false,
            }
        }
    }

    pub fn set_app_name(mut self, name: &str) -> Self {
        self.app_name = CString::new(name).unwrap();
        self.application_info.p_application_name = self.app_name.as_ptr();
        self
    }

    pub fn set_required_version(mut self, major: u32, minor: u32, patches: u32) -> Self {
        self.application_info.api_version = vk::make_api_version(0, major, minor, patches);
        self
    }

    pub fn enable_debug(mut self) -> Self {
        self.extensions
            .push(CString::new("VK_EXT_debug_utils").unwrap());
        self.layers
            .push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());

        self.debug_util_info = Some(
            vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback)),
        );
        self
    }

    pub fn build(
        mut self,
    ) -> (
        ash::Instance,
        Entry,
        ash::vk::DebugUtilsMessengerEXT,
        debug_utils::Instance,
    ) {
        let engine_name = CString::new(InstanceBuilder::ENGINE_NAME).unwrap();

        let raw_extensions: Vec<*const i8> =
            self.extensions.iter().map(|ext| ext.as_ptr()).collect();
        let raw_layers: Vec<*const i8> = self.layers.iter().map(|layer| layer.as_ptr()).collect();

        self.application_info.p_engine_name = engine_name.as_ptr();

        let mut instance_info = vk::InstanceCreateInfo::default();
        instance_info = instance_info
            .application_info(&self.application_info)
            .enabled_extension_names(&raw_extensions)
            .enabled_layer_names(&raw_layers);

        unsafe {
            let instance = self.entry.create_instance(&instance_info, None).unwrap();

            let debug_loader = debug_utils::Instance::new(&self.entry, &instance);
            let debug_call_back = debug_loader
                .create_debug_utils_messenger(&self.debug_util_info.unwrap(), None)
                .unwrap();
            (instance, self.entry, debug_call_back, debug_loader)
        }
    }
}

struct AllocatedImage {
    alloc: vk_mem::Allocation,
    image: vk::Image,
    view: vk::ImageView,
}

pub struct SwapchainBuilder {
    surface: vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
    physical: vk::PhysicalDevice,
    device: ash::Device,
    instance: ash::Instance,
    allocator: vk_mem::Allocator,

    present_mode: vk::PresentModeKHR,
    present_queue: TKQueue,

    min_image_count: u32,
    sharing_mode: vk::SharingMode,
    image_extent: vk::Extent2D,
    image_format: vk::Format,
}

impl SwapchainBuilder {
    pub unsafe fn new(
        entry: Entry,
        instance: ash::Instance,
        physical: vk::PhysicalDevice,
        window: winit::window::Window,
    ) -> SwapchainBuilder {
        let surface = ash_window::create_surface(
            &entry,
            &instance,
            window.display_handle().unwrap().as_raw(),
            window.window_handle().unwrap().as_raw(),
            None,
        )
        .expect("surface failed");

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        let surface_capabilities = surface_loader
            .get_physical_device_surface_capabilities(physical, surface)
            .unwrap();
        let min_image_count = surface_capabilities.min_image_count;
    }

    pub fn select_presentation_mode(mut self, present_format: vk::PresentModeKHR) {
        unsafe {
            let present_modes = self
                .surface_loader
                .get_physical_device_surface_present_modes(self.physical, self.surface)
                .expect("failed to get present modes!");

            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == present_format)
                .unwrap_or(vk::PresentModeKHR::FIFO);
        }
    }
    pub fn select_image_format(format: vk::Format) {}
    pub fn select_extent(extent: vk::Extent2D) {}
    pub fn select_sharing_mode(sharing_mode: vk::SharingMode) {}

    pub fn build(self) {
        unsafe {
            let swapchain_info = vk::SwapchainCreateInfoKHR::default()
                .image_extent(self.image_extent)
                .image_format(self.image_format)
                .image_sharing_mode(self.sharing_mode)
                .min_image_count(self.min_image_count)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_array_layers(1)
                .clipped(true);

            let swapchain_loader = swapchain::Device::new(&self.instance, &self.device);

            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_info, None)
                .expect("failed to create a swapchain");

            let swapchain_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
            let present_image_views: Vec<vk::ImageView> = swapchain_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::default()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(self.image_format)
                        .components(init::image_components_rgba())
                        .subresource_range(init::image_subresource_info())
                        .image(image);

                    self.device
                        .create_image_view(&create_view_info, None)
                        .unwrap()
                })
                .collect();
        }

        let (depth_info, alloc_info) = init::image_info(
            self.image_extent,
            4,
            MemoryPropertyFlags::DEVICE_LOCAL,
            vk::Format::D16_UNORM,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        );

        unsafe {
            let depth = self
                .allocator
                .create_image(&depth_info, &alloc_info)
                .unwrap();
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}
