use std::{
    borrow::Cow,
    ffi::{self, CStr, CString},
    ptr::null,
    sync::Arc,
};

use ash::{
    ext::debug_utils,
    khr::swapchain,
    vk::{self, ApplicationInfo, ColorSpaceKHR, MemoryPropertyFlags, Queue, QueueFlags},
    Entry,
};
use vk_mem::{Alloc, AllocationCreateInfo, Allocator, AllocatorCreateInfo};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::init;
// everything that is not a builder, will be moved later from here

pub struct TKQueue {
    queue: vk::Queue,
    family: u32,
}

impl Default for TKQueue {
    fn default() -> Self {
        Self {
            queue: Default::default(),
            family: Default::default(),
        }
    }
}
impl TKQueue {
    pub fn get_family(&self) -> u32 {
        self.family
    }
    pub fn get_queue(&self) -> vk::Queue {
        self.queue
    }
}

pub struct AllocatedImage {
    alloc: Option<vk_mem::Allocation>,
    image: vk::Image,
    view: vk::ImageView,
}

impl Default for AllocatedImage {
    fn default() -> Self {
        Self {
            alloc: Default::default(),
            image: Default::default(),
            view: Default::default(),
        }
    }
}

pub struct VulkanContext {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub physical: vk::PhysicalDevice,
    pub device: ash::Device,
    pub allocator: vk_mem::Allocator,

    pub window_extent: vk::Extent2D,
    pub window: winit::window::Window,

    pub surface: vk::SurfaceKHR,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<AllocatedImage>,
    pub depth_image: AllocatedImage,

    pub main_cmd: vk::CommandBuffer,
    pub main_pool: vk::CommandPool,

    pub graphic_queue: TKQueue,
    pub transfer: TKQueue,

    pub debug_messenger: vk::DebugUtilsMessengerEXT,

    pub swapchain_loader: ash::khr::swapchain::Device,
    pub surface_loader: ash::khr::surface::Instance,
    pub debug_loader: Option<ash::ext::debug_utils::Instance>,

    pub set_debug_util_object_name_ext: vk::PFN_vkSetDebugUtilsObjectNameEXT,
}

impl VulkanContext {
    pub fn new(
        entry: ash::Entry,
        instance: ash::Instance,
        device: ash::Device,
        physical: vk::PhysicalDevice,
        window: winit::window::Window,
    ) -> Self {
        unsafe {
            let allocator = Allocator::new(AllocatorCreateInfo::new(&instance, &device, physical))
                .expect("failed to create vma allocator");

            let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

            let window_extent = vk::Extent2D {
                width: window.inner_size().width,
                height: window.inner_size().height,
            };

            let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

            let set_debug_util_object_name_ext: vk::PFN_vkSetDebugUtilsObjectNameEXT =
                std::mem::transmute(
                    instance
                        .get_device_proc_addr(
                            device.handle(),
                            CString::new("vkSetDebugUtilsObjectNameEXT")
                                .unwrap()
                                .as_ptr(),
                        )
                        .unwrap(),
                );

            Self {
                entry,
                instance,
                allocator,
                window,
                device,
                window_extent,
                physical: Default::default(),
                surface: Default::default(),

                main_cmd: Default::default(),
                main_pool: Default::default(),

                graphic_queue: Default::default(),
                transfer: Default::default(),

                swapchain_images: Default::default(),
                depth_image: Default::default(),
                swapchain: Default::default(),
                swapchain_loader,
                surface_loader,

                debug_messenger: Default::default(),
                debug_loader: None,

                set_debug_util_object_name_ext,
            }
        }
    }
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

            for (index, family) in queues.iter().enumerate() {
                if family.queue_flags.contains(queue_flag) {
                    let tk_queue = TKQueue {
                        queue: vk::Queue::null(),
                        family: index as u32,
                    };
                    queue = Some(tk_queue);
                    break;
                }
            }
            queue
        }
    }
    pub fn find_transfer_only(
        instance: ash::Instance,
        physical: vk::PhysicalDevice,
    ) -> Option<Self> {
        let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
        let mut transfer_queue: Option<TKQueue> = None;

        for (index, family) in queues.iter().enumerate() {
            if !family.queue_flags.contains(QueueFlags::GRAPHICS)
                && family.queue_flags.contains(QueueFlags::TRANSFER)
            {
                let tk_queue = TKQueue {
                    queue: vk::Queue::null(),
                    family: index as u32,
                };
                transfer_queue = Some(tk_queue);
                break;
            }
        }
        transfer_queue
    }
}

// specific implementation

struct DeviceHelper {}

pub struct DeviceBuilder<'a> {
    features: vk::PhysicalDeviceFeatures,
    features_12: vk::PhysicalDeviceVulkan12Features<'a>,
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
        let features_12 = vk::PhysicalDeviceVulkan12Features::default();
        let features_13 = vk::PhysicalDeviceVulkan13Features::default();
        let extensions = vec![CString::new("VK_KHR_swapchain").unwrap()];
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
            features_12,
            features_13,
            extensions,
            physical,
            transfer_queue,
            graphic_queue,
            instance,
        }
    }

    pub fn select_physical_device(mut self) -> Self {
        let mut has_queues_required: bool = false;

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
                    self.physical = physical;
                    has_queues_required = true;
                    break;
                }
            }

            if !has_queues_required {
                panic!("None of the Vulkan supported gpus have the required queues");
            }
        }

        self
    }

    pub fn ext_bindless_descriptors(mut self) -> Self {
        self.features_12 = self
            .features_12
            .descriptor_binding_sampled_image_update_after_bind(true)
            .shader_sampled_image_array_non_uniform_indexing(true)
            .shader_storage_buffer_array_non_uniform_indexing(true)
            .shader_uniform_buffer_array_non_uniform_indexing(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_buffer_update_after_bind(true)
            .buffer_device_address(true);

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

    pub fn build(mut self) -> (ash::Device, vk::PhysicalDevice, TKQueue, TKQueue) {
        let raw_ext: Vec<*const i8> = self.extensions.iter().map(|raw| raw.as_ptr()).collect();

        let priority = [1.0 as f32];
        let device_queue_info = [
            init::device_create_into(self.graphic_queue.family).queue_priorities(&priority),
            init::device_create_into(self.transfer_queue.family).queue_priorities(&priority),
        ];

        let info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&raw_ext)
            .enabled_features(&self.features)
            .queue_create_infos(&device_queue_info)
            .push_next(&mut self.features_13);

        unsafe {
            let device = self
                .instance
                .create_device(self.physical, &info, None)
                .expect("failed created a logical device");

            self.graphic_queue.queue =
                device.get_device_queue2(&init::device_queue_info(self.graphic_queue.family));

            self.transfer_queue.queue =
                device.get_device_queue2(&init::device_queue_info(self.graphic_queue.family));

            (
                device,
                self.physical,
                self.graphic_queue,
                self.transfer_queue,
            )
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
            let extensions = vec![CString::new("VK_KHR_surface").unwrap()];
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

    pub fn set_xlib_ext(mut self) -> Self {
        self.extensions
            .push(CString::new("VK_KHR_xlib_surface").unwrap());
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

pub struct SwapchainBuilder<'a> {
    vulkan_context: &'a mut VulkanContext,
    present_mode: vk::PresentModeKHR,
    present_queue: Option<TKQueue>,

    min_image_count: u32,
    sharing_mode: vk::SharingMode,
    image_format: vk::Format,

    transform: vk::SurfaceTransformFlagsKHR,
}

impl<'a> SwapchainBuilder<'a> {
    pub unsafe fn new(vulkan_context: &'a mut VulkanContext) -> SwapchainBuilder {
        vulkan_context.surface = ash_window::create_surface(
            &vulkan_context.entry,
            &vulkan_context.instance,
            vulkan_context.window.display_handle().unwrap().as_raw(),
            vulkan_context.window.window_handle().unwrap().as_raw(),
            None,
        )
        .expect("surface failed");

        let surface_capabilities = vulkan_context
            .surface_loader
            .get_physical_device_surface_capabilities(
                vulkan_context.physical,
                vulkan_context.surface,
            )
            .unwrap();

        let formats = vulkan_context
            .surface_loader
            .get_physical_device_surface_formats(vulkan_context.physical, vulkan_context.surface)
            .unwrap();
        for format in formats {
            println!(
                "Available formats {:?}\nWith Color space {:?}\n\n",
                format.format, format.color_space
            );
        }

        let min_image_count = surface_capabilities.min_image_count;

        Self {
            transform: surface_capabilities.current_transform,
            vulkan_context,
            present_mode: vk::PresentModeKHR::FIFO,
            present_queue: None,
            min_image_count,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            image_format: vk::Format::R8G8B8A8_SRGB,
        }
    }

    pub fn select_presentation_mode(mut self, present_format: vk::PresentModeKHR) -> Self {
        unsafe {
            let present_modes = self
                .vulkan_context
                .surface_loader
                .get_physical_device_surface_present_modes(
                    self.vulkan_context.physical,
                    self.vulkan_context.surface,
                )
                .expect("failed to get present modes!");

            self.present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == present_format)
                .unwrap_or(vk::PresentModeKHR::FIFO);
        }
        self
    }
    pub fn select_image_format(mut self, format: vk::Format) -> Self {
        self.image_format = format;
        self
    }

    pub fn select_sharing_mode(mut self, sharing_mode: vk::SharingMode) -> Self {
        self.sharing_mode = sharing_mode;
        self
    }

    pub fn build(self) {
        unsafe {
            let swapchain_info = vk::SwapchainCreateInfoKHR::default()
                .flags(vk::SwapchainCreateFlagsKHR::empty())
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_extent(self.vulkan_context.window_extent)
                .image_format(self.image_format)
                .image_sharing_mode(self.sharing_mode)
                .min_image_count(self.min_image_count)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_array_layers(1)
                .surface(self.vulkan_context.surface)
                .pre_transform(self.transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .clipped(true);

            let swapchain = self
                .vulkan_context
                .swapchain_loader
                .create_swapchain(&swapchain_info, None)
                .expect("failed to create a swapchain");

            let swapchain_images = self
                .vulkan_context
                .swapchain_loader
                .get_swapchain_images(swapchain)
                .unwrap();
            let swapchain_images_alloc: Vec<AllocatedImage> = swapchain_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::default()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(self.image_format)
                        .components(init::image_components_rgba())
                        .subresource_range(init::image_subresource_info(
                            vk::ImageAspectFlags::COLOR,
                        ))
                        .image(image);

                    let view = self
                        .vulkan_context
                        .device
                        .create_image_view(&create_view_info, None)
                        .unwrap();

                    AllocatedImage {
                        alloc: None,
                        image,
                        view,
                    }
                })
                .collect();

            let (depth_info, alloc_info) = init::image_info(
                self.vulkan_context.window_extent,
                4,
                MemoryPropertyFlags::DEVICE_LOCAL,
                vk::Format::D16_UNORM,
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            );

            let depth = self
                .vulkan_context
                .allocator
                .create_image(&depth_info, &alloc_info)
                .unwrap();

            let depth_view = self
                .vulkan_context
                .device
                .create_image_view(
                    &init::image_view_info(
                        depth.0,
                        vk::Format::D16_UNORM,
                        vk::ImageAspectFlags::DEPTH,
                    ),
                    None,
                )
                .unwrap();
            self.vulkan_context.swapchain_images = swapchain_images_alloc;
            self.vulkan_context.swapchain = swapchain;

            self.vulkan_context.depth_image = AllocatedImage {
                alloc: Some(depth.1),
                image: depth.0,
                view: depth_view,
            };
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
    if message_type == vk::DebugUtilsMessageTypeFlagsEXT::GENERAL {
        return vk::FALSE;
    }
    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}
