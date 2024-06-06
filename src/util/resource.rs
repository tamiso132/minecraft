use std::{ffi::CString, mem::MaybeUninit, ptr, sync::Arc};

use ash::{
    util,
    vk::{
        self, BufferUsageFlags, DebugUtilsObjectNameInfoEXT, DescriptorType, ImageLayout,
        MemoryPropertyFlags,
    },
};
use vk_mem::Alloc;

use crate::vulkan::VulkanContext;

use super::{create_layout, init, loader::DebugLoaderEXT};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Binding {
    Uniform,
    Storage,
    Texture,
    CombinedImage,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Vertex = vk::BufferUsageFlags::VERTEX_BUFFER.as_raw(),
    Uniform = vk::BufferUsageFlags::UNIFORM_BUFFER.as_raw(),
    Storage = vk::BufferUsageFlags::STORAGE_BUFFER.as_raw(),
    Index = vk::BufferUsageFlags::INDEX_BUFFER.as_raw(),
}

impl Into<vk::BufferUsageFlags> for BufferType {
    fn into(self) -> vk::BufferUsageFlags {
        vk::BufferUsageFlags::from_raw(self as u32)
    }
}

pub struct Range {
    offset: u32,
    size: u32,
    data: *mut u8,
}
pub struct AllocatedImage {
    pub alloc: Option<vk_mem::Allocation>,
    pub image: vk::Image,
    pub view: vk::ImageView,

    pub format: vk::Format,
    pub layout: vk::ImageLayout,
    pub descriptor_type: vk::DescriptorType,
}

impl Default for AllocatedImage {
    fn default() -> Self {
        Self {
            alloc: Default::default(),
            image: Default::default(),
            view: Default::default(),
            descriptor_type: vk::DescriptorType::SAMPLER,
            format: vk::Format::R8G8B8A8_SRGB,
            layout: vk::ImageLayout::UNDEFINED,
        }
    }
}

pub struct AllocatedBuffer {
    pub index: u32,
    pub buffer: vk::Buffer,
    pub alloc: vk_mem::Allocation,
    pub buffer_type: BufferType,
    pub descriptor_type: vk::DescriptorType,
    pub size: u64,
}

pub struct Resource {
    device: Arc<ash::Device>,
    instance: Arc<ash::Instance>,
    allocator: Arc<vk_mem::Allocator>,

    pub layout: vk::DescriptorSetLayout,
    pub set: vk::DescriptorSet,

    debug_loader: DebugLoaderEXT,

    buffer_counter: u32,
    texture_counter: u32,

    storage_counter: u32,
}

impl Resource {
    const MAX_BINDINGS: u32 = 1024;

    pub unsafe fn new(
        instance: Arc<ash::Instance>,
        device: Arc<ash::Device>,
        physical: vk::PhysicalDevice,
        allocator: Arc<vk_mem::Allocator>,
        debug_loader_ext: DebugLoaderEXT,
    ) -> Self {
        let pool_sizes = vec![
            init::descriptor_pool_size(
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                Self::MAX_BINDINGS,
            ),
            init::descriptor_pool_size(vk::DescriptorType::STORAGE_IMAGE, Self::MAX_BINDINGS),
            init::descriptor_pool_size(vk::DescriptorType::STORAGE_BUFFER, Self::MAX_BINDINGS),
        ];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(3)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND_EXT);

        let descriptor_pool = device
            .create_descriptor_pool(&descriptor_pool_info, None)
            .unwrap();

        let layout = create_layout(
            device.clone(),
            0,
            vec![
                DescriptorType::COMBINED_IMAGE_SAMPLER,
                DescriptorType::STORAGE_IMAGE,
                DescriptorType::STORAGE_BUFFER,
            ],
            &debug_loader_ext,
            CString::new("global").unwrap(),
        );

        let a = device
            .allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&[layout]),
            )
            .unwrap();

        println!("DescriptorSet Len: {}\n", a.len());
        let set = a[0];
        debug_loader_ext
            .set_debug_util_object_name_ext(
                DebugUtilsObjectNameInfoEXT::default()
                    .object_handle(set)
                    .object_name(&CString::new("global").unwrap()),
            )
            .unwrap();

        Self {
            device: device.clone(),
            instance: instance.clone(),
            allocator,
            debug_loader: debug_loader_ext,
            buffer_counter: 0,
            texture_counter: 0,
            layout,
            set,
            storage_counter: 0,
        }
    }

    pub fn create_buffer(
        &mut self,
        alloc_size: u64,
        buffer_type: BufferType,
        queue_family: u32,
        object_name: String,
    ) -> AllocatedBuffer {
        let queue_family = [queue_family];

        let buffer_usage_flag: vk::BufferUsageFlags = buffer_type.into();

        let buffer_info = vk::BufferCreateInfo::default()
            .size(alloc_size)
            .usage(buffer_usage_flag | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family);

        let mut alloc_info = vk_mem::AllocationCreateInfo::default();

        let memory_property =
            if buffer_type == BufferType::Storage || buffer_type == BufferType::Uniform {
                MemoryPropertyFlags::DEVICE_LOCAL
            } else {
                MemoryPropertyFlags::HOST_VISIBLE
            };

        let (descriptor_type, binding) = if buffer_type == BufferType::Storage {
            (vk::DescriptorType::STORAGE_BUFFER, Binding::Storage)
        } else {
            (vk::DescriptorType::UNIFORM_BUFFER, Binding::Uniform)
        };

        alloc_info.required_flags = memory_property;

        unsafe {
            let buffer = self
                .allocator
                .create_buffer(&buffer_info, &alloc_info)
                .expect("failed to create buffer");

            let cstring = CString::new(object_name).expect("failed");
            let debug_info = vk::DebugUtilsObjectNameInfoEXT::default()
                .object_handle(buffer.0)
                .object_name(&cstring);

            self.debug_loader
                .set_debug_util_object_name_ext(debug_info)
                .unwrap();

            let buffer_info_descriptor = vk::DescriptorBufferInfo::default()
                .buffer(buffer.0)
                .offset(0)
                .range(vk::WHOLE_SIZE);

            let desc = [buffer_info_descriptor];

            let write = vk::WriteDescriptorSet::default()
                .descriptor_type(descriptor_type)
                .dst_set(self.set)
                .descriptor_count(1)
                .buffer_info(&desc)
                .dst_array_element(self.buffer_counter)
                .dst_binding(1);

            self.device.update_descriptor_sets(&[write], &vec![]);
            self.buffer_counter += 1;

            AllocatedBuffer {
                buffer: buffer.0,
                alloc: buffer.1,
                buffer_type,
                size: buffer_info.size,
                index: self.buffer_counter - 1,
                descriptor_type,
            }
        }
    }

    pub fn get_layout_vec(&self) -> Vec<vk::DescriptorSetLayout> {
        vec![self.layout]
    }

    pub fn bind_image(&mut self, image: &AllocatedImage) {
        let descriptor_image_info = vec![vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(image.view)
            .sampler(vk::Sampler::null())];

        let descriptor_write = vk::WriteDescriptorSet::default()
            .descriptor_type(image.descriptor_type)
            .descriptor_count(1)
            .dst_binding(1)
            .dst_set(self.set)
            .dst_array_element(self.storage_counter)
            .image_info(&descriptor_image_info);

        // TODO , not hardcode storage image
        self.storage_counter += 1;

        unsafe {
            self.device
                .update_descriptor_sets(&vec![descriptor_write], &vec![])
        };
    }

    pub fn create_storage_image(
        &mut self,
        extent: vk::Extent2D,
        pixel_size: u32,
        memory_type: MemoryPropertyFlags,
        format: vk::Format,
        image_usage: vk::ImageUsageFlags,
        name: CString,
    ) -> AllocatedImage {
        let (image_info, alloc_info) =
            init::image_info(extent, pixel_size, memory_type, format, image_usage);

        unsafe {
            let image = self
                .allocator
                .create_image(&image_info, &alloc_info)
                .unwrap();

            let image_view_info =
                init::image_view_info(image.0, format, vk::ImageAspectFlags::COLOR);

            let view = self
                .device
                .create_image_view(&image_view_info, None)
                .unwrap();

            let alloc_image = AllocatedImage {
                alloc: Some(image.1),
                image: image.0,
                view,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                format,
                layout: ImageLayout::UNDEFINED,
            };

            self.debug_loader
                .set_debug_util_object_name_ext(
                    vk::DebugUtilsObjectNameInfoEXT::default()
                        .object_handle(image.0)
                        .object_name(&name),
                )
                .unwrap();
            self.debug_loader
                .set_debug_util_object_name_ext(
                    vk::DebugUtilsObjectNameInfoEXT::default()
                        .object_handle(image.0)
                        .object_name(&name),
                )
                .unwrap();
            // TODO, automatically transfer it to general layout
            self.bind_image(&alloc_image);

            alloc_image
        }
    }
}
