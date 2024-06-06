use std::sync::Arc;

use ash::vk::{self, QueueFlags};
use vk_mem::{Alloc, Allocator};

use crate::util::{
    create_fence, create_semphore,
    loader::{DebugLoaderEXT, ShaderLoaderEXT},
    resource::{self, AllocatedImage, Resource},
};

pub trait PushConstant {
    fn size(&self) -> u64;
    fn stage_flag(&self) -> vk::ShaderStageFlags;
    fn push_constant_range(&self) -> vk::PushConstantRange;
}

#[repr(C)]
pub struct SkyBoxPushConstant {
    data1: glm::Vec4,
    data2: glm::Vec4,
    data3: glm::Vec4,
    data4: glm::Vec4,
}

impl SkyBoxPushConstant {
    pub fn new() -> Self {
        Self {
            data1: glm::vec4(0.5, 0.5, 0.5, 0.5),
            data2: glm::vec4(1.0, 0.5, 0.5, 0.5),
            data3: glm::vec4(1.0, 1.0, 1.0, 1.0),
            data4: glm::vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl PushConstant for SkyBoxPushConstant {
    fn size(&self) -> u64 {
        std::mem::size_of::<SkyBoxPushConstant>() as u64
    }

    fn stage_flag(&self) -> vk::ShaderStageFlags {
        vk::ShaderStageFlags::COMPUTE
    }

    fn push_constant_range(&self) -> vk::PushConstantRange {
        vk::PushConstantRange::default()
            .size(self.size() as u32)
            .offset(0)
            .stage_flags(self.stage_flag())
    }
}

struct Application {
    context: VulkanContext,
}

pub struct VulkanContext {
    pub entry: ash::Entry,
    pub instance: Arc<ash::Instance>,
    pub device: Arc<ash::Device>,
    pub physical: vk::PhysicalDevice,
    /// Don't forget to clean this one up
    pub allocator: Arc<vk_mem::Allocator>,

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

    pub debug_loader_ext: DebugLoaderEXT,

    pub pipeline_layout: vk::PipelineLayout,
    pub resources: Resource,

    pub max_swapchain_images: u32,
    pub swapchain_index: u32,

    pub queue_is_done_fen: vk::Fence,

    pub image_aquired_semp: vk::Semaphore,
    pub render_is_done: vk::Semaphore,
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
            let mut allocator_info = vk_mem::AllocatorCreateInfo::new(&instance, &device, physical);
            allocator_info.flags |= vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;

            let allocator =
                Arc::new(Allocator::new(allocator_info).expect("failed to create vma allocator"));

            let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

            let window_extent = vk::Extent2D {
                width: window.inner_size().width,
                height: window.inner_size().height,
            };

            let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

            let instance = Arc::new(instance);
            let device = Arc::new(device);

            let debug_loader_ext = DebugLoaderEXT::new(instance.clone(), device.clone());

            let resources = resource::Resource::new(
                instance.clone(),
                device.clone(),
                physical,
                allocator.clone(),
                debug_loader_ext.clone(),
            );
            Self {
                entry,
                instance: instance.clone(),
                allocator,
                window,
                device: device.clone(),
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

                debug_loader_ext,
                pipeline_layout: vk::PipelineLayout::null(),

                resources,
                swapchain_index: 0,
                max_swapchain_images: 0,
                queue_is_done_fen: create_fence(&device),
                image_aquired_semp: create_semphore(&device),
                render_is_done: create_semphore(&device),
            }
        }
    }
}
#[derive(Debug)]
pub struct TKQueue {
    pub queue: vk::Queue,
    pub family: u32,
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
