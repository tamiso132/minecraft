use std::sync::Arc;

use ash::vk::{self, QueueFlags};
use vk_mem::Allocator;

use crate::util::loader::{DebugLoaderEXT, ShaderLoaderEXT};

pub struct VulkanContext {
    pub entry: ash::Entry,
    pub instance: Arc<ash::Instance>,
    pub device: Arc<ash::Device>,
    pub physical: vk::PhysicalDevice,
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

    pub debug_loader_ext: DebugLoaderEXT,
    pub shader_loader_ext: ShaderLoaderEXT,
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
            let allocator = Allocator::new(vk_mem::AllocatorCreateInfo::new(
                &instance, &device, physical,
            ))
            .expect("failed to create vma allocator");

            let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

            let window_extent = vk::Extent2D {
                width: window.inner_size().width,
                height: window.inner_size().height,
            };

            let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

            let instance = Arc::new(instance);
            let device = Arc::new(device);
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

                debug_loader_ext: DebugLoaderEXT::new(instance.clone(), device.clone()),
                shader_loader_ext: ShaderLoaderEXT::new(instance.clone(), device.clone()),
            }
        }
    }
}

pub struct AllocatedImage {
    pub alloc: Option<vk_mem::Allocation>,
    pub image: vk::Image,
    pub view: vk::ImageView,
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
