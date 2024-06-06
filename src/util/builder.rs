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

use crate::vulkan::{TKQueue, VulkanContext};

use super::{
    init,
    loader::{DebugLoaderEXT, ShaderLoaderEXT},
    resource::AllocatedImage,
};

// everything that is not a builder, will be moved later from here

// specific implementation

struct DeviceHelper {}
#[derive(Debug)]
pub struct DeviceBuilder<'a> {
    features: vk::PhysicalDeviceFeatures,
    features_11: vk::PhysicalDeviceVulkan11Features<'a>,
    features_12: vk::PhysicalDeviceVulkan12Features<'a>,
    features_13: vk::PhysicalDeviceVulkan13Features<'a>,
    extensions: Vec<CString>,
    physical: vk::PhysicalDevice,

    shader_object_ext: Option<vk::PhysicalDeviceShaderObjectFeaturesEXT<'a>>,

    transfer_queue: TKQueue,
    graphic_queue: TKQueue,
}

impl<'a> DeviceBuilder<'a> {
    pub fn new() -> Self {
        let features = vk::PhysicalDeviceFeatures::default();
        let features_11 = vk::PhysicalDeviceVulkan11Features::default();
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
            shader_object_ext: None,
            features,
            features_11,
            features_12,
            features_13,
            extensions,
            physical,
            transfer_queue,
            graphic_queue,
        }
    }

    pub fn select_physical_device(mut self, instance: &ash::Instance) -> Self {
        let mut has_queues_required: bool = false;

        unsafe {
            let physical_devices = instance
                .clone()
                .enumerate_physical_devices()
                .expect("no vulkan supported gpu");

            for physical in physical_devices {
                let graphic = TKQueue::find_queue(instance.clone(), physical, QueueFlags::GRAPHICS);
                let transfer = TKQueue::find_transfer_only(instance.clone(), physical);

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

    #[rustfmt::skip]
    pub fn ext_bindless_descriptors(mut self) -> Self {
        self.features_12 = self.features_12.
        buffer_device_address(true)
        .runtime_descriptor_array(true)
        .descriptor_binding_partially_bound(true)
        .descriptor_binding_sampled_image_update_after_bind(true)
        .descriptor_binding_storage_image_update_after_bind(true)
        .descriptor_binding_sampled_image_update_after_bind(true)
        .descriptor_binding_uniform_buffer_update_after_bind(true)
        .descriptor_binding_sampled_image_update_after_bind(true)
        .descriptor_binding_storage_buffer_update_after_bind(true)
        .shader_sampled_image_array_non_uniform_indexing(true)
        .shader_storage_buffer_array_non_uniform_indexing(true)
        .shader_uniform_buffer_array_non_uniform_indexing(true);

        self.extensions.push(CString::new("VK_KHR_buffer_device_address").unwrap());
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

    pub fn ext_shader_object(mut self) -> Self {
        // self.extensions
        //     .push(CString::new("VK_EXT_shader_object").unwrap());
        // self.shader_object_ext =
        //     Some(vk::PhysicalDeviceShaderObjectFeaturesEXT::default().shader_object(true));

        self
    }

    pub fn build(
        mut self,
        instance: &ash::Instance,
    ) -> (ash::Device, vk::PhysicalDevice, TKQueue, TKQueue) {
        for ext in &self.extensions {
            println!("{:?}\n", ext.as_c_str());
        }
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
            .push_next(&mut self.features_11)
            .push_next(&mut self.features_12)
            .push_next(&mut self.features_13);

        unsafe {
            let device = instance
                .create_device(self.physical, &info, None)
                .expect("failed created a logical device");

            // for ext in self
            //     .instance
            //     .enumerate_device_extension_properties(self.physical)
            //     .unwrap()
            // {
            //     println!("{:?}\n", ext.extension_name_as_c_str());
            // }

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
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                )
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
                        descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
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
            self.vulkan_context.max_swapchain_images = self.min_image_count;

            self.vulkan_context.depth_image = AllocatedImage {
                alloc: Some(depth.1),
                image: depth.0,
                view: depth_view,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
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

pub struct ComputePipelineBuilder {
    compute_shader: vk::ShaderModule,
}

impl ComputePipelineBuilder {
    pub fn new(compute_shader: vk::ShaderModule) -> Self {
        Self { compute_shader }
    }

    pub fn build(
        &self,
        context: &VulkanContext,
        pipeline_layout: vk::PipelineLayout,
    ) -> vk::Pipeline {
        let name = CString::new("main").unwrap();

        let compute_pipeline_info = vec![vk::ComputePipelineCreateInfo::default()
            .layout(pipeline_layout)
            .stage(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::COMPUTE)
                    .module(self.compute_shader)
                    .name(&name),
            )];

        unsafe {
            context
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &compute_pipeline_info, None)
                .unwrap()[0]
        }
    }
}