use std::{
    ffi::{c_void, CString},
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
    sync::Arc,
};

use ash::vk::{self, BufferUsageFlags, DebugUtilsObjectNameInfoEXT, DescriptorType, ImageLayout, ImageUsageFlags, MemoryPropertyFlags};
use vk_mem::Alloc;

use crate::vulkan::{util, VulkanContext};

use super::{init, loader::DebugLoaderEXT, TKQueue};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    CombinedImage,
    StorageImage,
    StorageBuffer,
    UniformBuffer,
    UNDEFINED,
}

impl Binding {
    /// update it if you increase amount of Bindings, dont count undefined
    const fn variants() -> usize {
        4
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Vertex = vk::BufferUsageFlags::VERTEX_BUFFER.as_raw(),
    Uniform = vk::BufferUsageFlags::UNIFORM_BUFFER.as_raw(),
    Storage = vk::BufferUsageFlags::STORAGE_BUFFER.as_raw(),
    Index = vk::BufferUsageFlags::INDEX_BUFFER.as_raw(),
}

#[repr(u32)]
#[derive(PartialEq, Eq)]
pub enum Memory {
    BestFit = 0,
    Local = vk::MemoryPropertyFlags::DEVICE_LOCAL.as_raw(),
    Host = vk::MemoryPropertyFlags::HOST_VISIBLE.as_raw(),
}

impl Into<vk::BufferUsageFlags> for BufferType {
    fn into(self) -> vk::BufferUsageFlags {
        vk::BufferUsageFlags::from_raw(self as u32)
    }
}

#[derive(Debug)]
pub struct AllocatedImage {
    /// Descriptor Binding Number
    pub binding: Binding,
    /// the index in the binding elements.
    pub index: u16,

    pub alloc: Option<vk_mem::Allocation>,
    pub image: vk::Image,
    pub view: vk::ImageView,
    /// Some might not have a sampler
    pub sampler: vk::Sampler,

    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub layout: vk::ImageLayout,
    pub memory: vk::MemoryPropertyFlags,
    pub usage: vk::ImageUsageFlags,
    pub descriptor_type: vk::DescriptorType,
}

impl AllocatedImage {
    // TODO, make this automatic, cause rn, there is high prone for error, when I change image struct

    pub fn set(&mut self, image: AllocatedImage) {
        self.binding = image.binding;
        self.index = image.index;
        self.alloc = image.alloc;
        self.image = image.image;
        self.sampler = image.sampler;
        self.extent = image.extent;
        self.format = image.format;
        self.layout = image.layout;
        self.memory = image.memory;
        self.usage = image.usage;
        self.descriptor_type = image.descriptor_type;
    }
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
            extent: Default::default(),
            sampler: vk::Sampler::null(),
            memory: vk::MemoryPropertyFlags::empty(),
            usage: vk::ImageUsageFlags::empty(),
            binding: Binding::UNDEFINED,
            index: 0,
        }
    }
}

pub struct AllocatedBuffer {
    /// the index in the binding elements.
    pub index: u16,
    /// the binding index
    pub binding: Binding,
    pub buffer: vk::Buffer,
    pub alloc: vk_mem::Allocation,
    pub buffer_type: BufferType,
    pub memory: vk::MemoryPropertyFlags,
    pub usage: vk::BufferUsageFlags,
    pub descriptor_type: vk::DescriptorType,
    /// size in bytes
    pub size: u64,
}

pub struct Resource {
    device: Arc<ash::Device>,
    instance: Arc<ash::Instance>,
    allocator: Arc<vk_mem::Allocator>,

    pub layout: vk::DescriptorSetLayout,
    pub set: vk::DescriptorSet,

    graphic_queue: TKQueue,
    cmd: vk::CommandBuffer,
    pool: vk::CommandPool,

    debug_loader: DebugLoaderEXT,
    counter: [u16; Binding::variants()],
}
// Genral TODO,  Seperate Descriptor create functions with creating resources not used with descriptor
// Option 1, split things into 2 functions, one for creating and another for binding
// can return a type that is able to bind, if binded, it returns another type.
// This will make it better in knowing what kind of resources stuff are
impl Resource {
    const MAX_BINDINGS: u32 = 1024;
    // Combined, Storage Image, Storage Buffer
    pub unsafe fn new(
        instance: Arc<ash::Instance>,
        device: Arc<ash::Device>,
        graphic_queue: TKQueue,
        allocator: Arc<vk_mem::Allocator>,
        debug_loader_ext: DebugLoaderEXT,
    ) -> Self {
        let pool_sizes = vec![
            init::descriptor_pool_size(vk::DescriptorType::COMBINED_IMAGE_SAMPLER, Self::MAX_BINDINGS),
            init::descriptor_pool_size(vk::DescriptorType::STORAGE_IMAGE, Self::MAX_BINDINGS),
            init::descriptor_pool_size(vk::DescriptorType::STORAGE_BUFFER, Self::MAX_BINDINGS),
            init::descriptor_pool_size(vk::DescriptorType::UNIFORM_BUFFER, Self::MAX_BINDINGS),
        ];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(3)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND_EXT);

        let descriptor_pool = device.create_descriptor_pool(&descriptor_pool_info, None).unwrap();

        let layout = util::create_bindless_layout(
            &device,
            0,
            vec![
                DescriptorType::COMBINED_IMAGE_SAMPLER,
                DescriptorType::STORAGE_IMAGE,
                DescriptorType::STORAGE_BUFFER,
                DescriptorType::UNIFORM_BUFFER,
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

        let set = a[0];
        debug_loader_ext
            .set_debug_util_object_name_ext(
                DebugUtilsObjectNameInfoEXT::default()
                    .object_handle(set)
                    .object_name(&CString::new("global").unwrap()),
            )
            .unwrap();
        let pool = util::create_pool(&device, graphic_queue.family);
        let cmd = util::create_cmd(&device, pool);

        Self {
            device,
            instance,
            allocator,
            debug_loader: debug_loader_ext,
            layout,
            set,
            graphic_queue,
            cmd,
            pool,
            counter: [0, 0, 0, 0],
        }
    }

    // dosent matter if it has many if statements, this is not supposed to be called every frame, as resizing is possible.

    /// a buffer that isnt bind into the descriptor, For vertex or Index buffers.
    pub fn create_buffer_non_descriptor(
        &mut self,
        alloc_size: u64,
        buffer_type: BufferType,
        memory: Memory,
        queue_family: u32,
        object_name: String,
    ) -> AllocatedBuffer {
        assert!(buffer_type == BufferType::Index || buffer_type == BufferType::Vertex, "Use regular create buffer");
        let queue_family = [queue_family];
        let mut alloc_info = vk_mem::AllocationCreateInfo::default();

        (alloc_info.required_flags) = {
            if memory == Memory::BestFit {
                vk::MemoryPropertyFlags::HOST_VISIBLE
            } else {
                vk::MemoryPropertyFlags::from_raw(memory as u32)
            }
        };

        let mut buffer_usage_flag: vk::BufferUsageFlags = buffer_type.into();

        if alloc_info.required_flags == MemoryPropertyFlags::DEVICE_LOCAL {
            buffer_usage_flag |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        let buffer_info = vk::BufferCreateInfo::default()
            .size(alloc_size)
            .usage(buffer_usage_flag | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family);

        unsafe {
            let buffer = self.allocator.create_buffer(&buffer_info, &alloc_info).expect("failed to create buffer");

            let cstring = CString::new(object_name).expect("failed");
            let debug_info = vk::DebugUtilsObjectNameInfoEXT::default().object_handle(buffer.0).object_name(&cstring);

            self.debug_loader.set_debug_util_object_name_ext(debug_info).unwrap();

            AllocatedBuffer {
                buffer: buffer.0,
                alloc: buffer.1,
                buffer_type,
                size: alloc_size,
                index: 0,
                descriptor_type: vk::DescriptorType::from_raw(0),
                memory: alloc_info.required_flags,
                usage: buffer_usage_flag,
                binding: Binding::UNDEFINED,
            }
        }
    }

    pub fn create_buffer(
        &mut self,
        alloc_size: u64,
        buffer_type: BufferType,
        memory: Memory,
        queue_family: u32,
        object_name: String,
    ) -> AllocatedBuffer {
        let queue_family = [queue_family];

        let mut alloc_info = vk_mem::AllocationCreateInfo::default();

        let (descriptor_type, binding) = if buffer_type == BufferType::Storage {
            (vk::DescriptorType::STORAGE_BUFFER, Binding::StorageBuffer)
        } else {
            (vk::DescriptorType::UNIFORM_BUFFER, Binding::UniformBuffer)
        };

        (alloc_info.required_flags) = {
            if memory == Memory::BestFit {
                vk::MemoryPropertyFlags::HOST_VISIBLE
            } else {
                vk::MemoryPropertyFlags::from_raw(memory as u32)
            }
        };

        let mut buffer_usage_flag: vk::BufferUsageFlags = buffer_type.into();

        if alloc_info.required_flags == MemoryPropertyFlags::DEVICE_LOCAL {
            buffer_usage_flag |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        let buffer_info = vk::BufferCreateInfo::default()
            .size(alloc_size)
            .usage(buffer_usage_flag | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family);

        unsafe {
            let buffer = self.allocator.create_buffer(&buffer_info, &alloc_info).expect("failed to create buffer");

            let cstring = CString::new(object_name).expect("failed");
            let debug_info = vk::DebugUtilsObjectNameInfoEXT::default().object_handle(buffer.0).object_name(&cstring);

            self.debug_loader.set_debug_util_object_name_ext(debug_info).unwrap();

            let mut alloc_buffer = AllocatedBuffer {
                buffer: buffer.0,
                alloc: buffer.1,
                buffer_type,
                size: buffer_info.size,
                index: self.counter[binding as usize] - 1,
                descriptor_type,
                memory: alloc_info.required_flags,
                usage: buffer_usage_flag,
                binding,
            };

            let buffer_descriptor = init::buffer_descriptor_info(alloc_buffer.buffer);

            self.bind_to_descriptor(
                &mut alloc_buffer.index,
                alloc_buffer.descriptor_type,
                alloc_buffer.binding,
                vec![],
                buffer_descriptor,
            );

            alloc_buffer
        }
    }

    pub fn create_depth_image(&mut self, format: vk::Format, extent: vk::Extent2D) -> AllocatedImage {
        let usage = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;

        let (image_info, alloc_info) = init::image_info(extent, 4, vk::MemoryPropertyFlags::DEVICE_LOCAL, format, usage);
        unsafe {
            let depth_image = self.allocator.create_image(&image_info, &alloc_info).unwrap();

            let view_info = init::image_view_info(depth_image.0, format, vk::ImageAspectFlags::DEPTH);
            let view = self.device.create_image_view(&view_info, None).unwrap();

            AllocatedImage {
                binding: Binding::UNDEFINED,
                index: 0,
                alloc: Some(depth_image.1),
                image: depth_image.0,
                view,
                sampler: vk::Sampler::null(),
                extent,
                format,
                layout: vk::ImageLayout::UNDEFINED,
                memory: vk::MemoryPropertyFlags::DEVICE_LOCAL,
                usage,
                descriptor_type: vk::DescriptorType::from_raw(0),
            }
        }
    }

    pub fn create_texture_image(&mut self, extent: vk::Extent2D, data: &[u8]) -> AllocatedImage {
        let (staging_buffer, staging_alloc) = self.create_staging_buffer(data);

        let usage = ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED;
        let memory = vk::MemoryPropertyFlags::DEVICE_LOCAL;

        let (image_info, alloc_info) = init::image_info(extent, 4, memory, vk::Format::R8G8B8A8_UNORM, usage);

        unsafe {
            let texture_image = self.allocator.create_image(&image_info, &alloc_info).unwrap();
            let view_info = init::image_view_info(texture_image.0, image_info.format, vk::ImageAspectFlags::COLOR);

            let view = self.device.create_image_view(&view_info, None).unwrap();

            let sampler = util::create_sampler(&self.device, vk::Filter::LINEAR, vk::SamplerAddressMode::REPEAT);

            let mut image = AllocatedImage {
                alloc: Some(texture_image.1),
                image: texture_image.0,
                view,
                extent,
                format: view_info.format,
                layout: vk::ImageLayout::UNDEFINED,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                index: 0,
                sampler,
                memory,
                usage,
                binding: Binding::CombinedImage,
            };

            util::begin_cmd(&self.device, self.cmd);

            util::transition_image_transfer(&self.device, self.cmd, image.image);

            image.layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;

            util::copy_to_image_from_buffer(&self.device, self.cmd, &image, (staging_buffer, staging_alloc));

            util::transition_image_shader_only(&self.device, self.cmd, image.image);

            image.layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

            util::end_cmd_and_submit(&self.device, self.cmd, self.graphic_queue, vec![], vec![], vk::Fence::null());

            // TODO, bind to descriptor
            // TODO, fix this into returning a "promise", and they can await it when they need the texture.
            // there is also the option of queing up all the create textures.
            self.device.device_wait_idle().unwrap();

            let image_descriptor = init::image_descriptor_info(image.layout, image.view, image.sampler);

            // gonna remove this later when I refactor out imgui from using this
            self.bind_to_descriptor(&mut image.index, image.descriptor_type, image.binding, image_descriptor, vec![]);

            image
        }
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
        let (image_info, alloc_info) = init::image_info(extent, pixel_size, memory_type, format, image_usage);

        unsafe {
            let image = self.allocator.create_image(&image_info, &alloc_info).unwrap();

            let image_view_info = init::image_view_info(image.0, format, vk::ImageAspectFlags::COLOR);

            let view = self.device.create_image_view(&image_view_info, None).unwrap();
            let mut alloc_image = AllocatedImage {
                alloc: Some(image.1),
                image: image.0,
                view,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                format,
                layout: ImageLayout::UNDEFINED,
                extent,
                index: 0,
                memory: memory_type,
                usage: image_usage,
                sampler: vk::Sampler::null(),
                binding: Binding::StorageImage,
            };
            self.debug_loader
                .set_debug_util_object_name_ext(vk::DebugUtilsObjectNameInfoEXT::default().object_handle(image.0).object_name(&name))
                .unwrap();
            self.debug_loader
                .set_debug_util_object_name_ext(vk::DebugUtilsObjectNameInfoEXT::default().object_handle(image.0).object_name(&name))
                .unwrap();

            // TODO, automatically transfer it to general layout
            // self.bind_to_descriptor(&mut alloc_image, Binding::StorageImage);
            util::begin_cmd(&self.device, self.cmd);
            util::transition_image_general(&self.device, self.cmd, &mut alloc_image);
            util::end_cmd_and_submit(&self.device, self.cmd, self.graphic_queue, vec![], vec![], vk::Fence::null());
            self.device.queue_wait_idle(self.graphic_queue.get_queue()).unwrap();

            let image_descriptor = init::image_descriptor_info(alloc_image.layout, alloc_image.view, alloc_image.sampler);

            self.bind_to_descriptor(&mut alloc_image.index, alloc_image.descriptor_type, Binding::StorageImage, image_descriptor, vec![]);

            alloc_image
        }
    }

    pub fn create_staging_buffer(&self, data: &[u8]) -> (vk::Buffer, vk_mem::Allocation) {
        let buffer_info = vk::BufferCreateInfo::default()
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .size(data.len() as u64);

        unsafe {
            let mut alloc_info = vk_mem::AllocationCreateInfo::default();
            alloc_info.required_flags = vk::MemoryPropertyFlags::HOST_VISIBLE;

            let mut buffer = self.allocator.create_buffer(&buffer_info, &alloc_info).unwrap();

            let dst_ptr = self.allocator.map_memory(&mut buffer.1).unwrap();

            std::ptr::copy_nonoverlapping(data.as_ptr(), dst_ptr, data.len());

            self.allocator.unmap_memory(&mut buffer.1);

            buffer
        }
    }

    pub fn write_to_buffer_host(&mut self, buffer: &mut AllocatedBuffer, data: &[u8]) {
        unsafe {
            let dst_ptr = self.allocator.map_memory(&mut buffer.alloc).unwrap();

            ptr::copy_nonoverlapping(data.as_ptr(), dst_ptr, data.len());

            self.allocator.unmap_memory(&mut buffer.alloc);
        }
    }

    /// Only works for host visible memory
    pub fn resize_buffer(&mut self, resize_buffer: &mut AllocatedBuffer, new_size: u64) {
        let buffer_info = vk::BufferCreateInfo::default()
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(new_size)
            .usage(resize_buffer.usage);

        let mut alloc_info = vk_mem::AllocationCreateInfo::default();
        alloc_info.required_flags = resize_buffer.memory;

        unsafe {
            let new_buffer = self.allocator.create_buffer(&buffer_info, &alloc_info).unwrap();
            self.allocator.destroy_buffer(resize_buffer.buffer, &mut resize_buffer.alloc);

            /*Update  buffer */
            resize_buffer.size = new_size;
            resize_buffer.alloc = new_buffer.1;
            resize_buffer.buffer = new_buffer.0;

            let buffer_descriptor = init::buffer_descriptor_info(resize_buffer.buffer);

            self.update_descriptor_bind(
                resize_buffer.index as u32,
                resize_buffer.descriptor_type,
                resize_buffer.binding,
                vec![],
                buffer_descriptor,
            )
        }
    }

    /// Only use this for memory that has been allocated using VMA and is a descriptor_set aka not depth buffer
    pub fn resize_image(&mut self, alloc_image: &mut AllocatedImage, new_extent: vk::Extent2D) {
        assert!(alloc_image.alloc.is_some(), "Resource is not created with VMA");

        let (image_info, alloc_info) = init::image_info(new_extent, 4, alloc_image.memory, alloc_image.format, alloc_image.usage);
        unsafe {
            let image = self.allocator.create_image(&image_info, &alloc_info).unwrap();

            let image_view_info = init::image_view_info(image.0, alloc_image.format, vk::ImageAspectFlags::COLOR);

            let view = self.device.create_image_view(&image_view_info, None).unwrap();

            self.allocator.destroy_image(alloc_image.image, alloc_image.alloc.as_mut().unwrap());
            self.device.destroy_image_view(alloc_image.view, None);

            alloc_image.image = image.0;
            alloc_image.alloc = Some(image.1);
            alloc_image.view = view;
            alloc_image.extent = new_extent;
        }
    }

    pub fn resize_buffer_non_descriptor(&mut self, resize_buffer: &mut AllocatedBuffer, new_size: u64) {
        let buffer_info = vk::BufferCreateInfo::default()
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .size(new_size)
            .usage(resize_buffer.usage);

        let mut alloc_info = vk_mem::AllocationCreateInfo::default();
        alloc_info.required_flags = resize_buffer.memory;

        unsafe {
            let new_buffer = self.allocator.create_buffer(&buffer_info, &alloc_info).unwrap();
            self.allocator.destroy_buffer(resize_buffer.buffer, &mut resize_buffer.alloc);

            /*Update  buffer */
            resize_buffer.size = new_size;
            resize_buffer.alloc = new_buffer.1;
            resize_buffer.buffer = new_buffer.0;
        }
    }

    pub fn resize_image_non_descriptor(&mut self, depth_alloc: &mut AllocatedImage) {}

    pub fn get_layout_vec(&self) -> Vec<vk::DescriptorSetLayout> {
        vec![self.layout]
    }

    // binds the image to the descriptor layout so it can be accessed through the shader
    fn bind_to_descriptor(
        &mut self,
        index: &mut u16,
        descriptor_type: vk::DescriptorType,
        binding: Binding,
        image_descriptor: Vec<vk::DescriptorImageInfo>,
        buffer_descriptor: Vec<vk::DescriptorBufferInfo>,
    ) {
        let binding = binding as usize;

        let descriptor_write = vk::WriteDescriptorSet::default()
            .descriptor_type(descriptor_type)
            .dst_binding(binding as u32)
            .dst_set(self.set)
            .dst_array_element(self.counter[binding] as u32)
            .image_info(&image_descriptor)
            .buffer_info(&buffer_descriptor)
            .descriptor_count(1);

        *index = self.counter[binding];

        self.counter[binding] += 1;

        unsafe { self.device.update_descriptor_sets(&vec![descriptor_write], &vec![]) };
    }

    fn update_descriptor_bind(
        &mut self,
        index: u32,
        descriptor_type: vk::DescriptorType,
        binding: Binding,
        image_descriptor: Vec<vk::DescriptorImageInfo>,
        buffer_descriptor: Vec<vk::DescriptorBufferInfo>,
    ) {
        let binding = binding as usize;

        let descriptor_write = vk::WriteDescriptorSet::default()
            .descriptor_type(descriptor_type)
            .descriptor_count(1)
            .dst_binding(binding as u32)
            .dst_set(self.set)
            .dst_array_element(index)
            .image_info(&image_descriptor)
            .buffer_info(&buffer_descriptor);

        unsafe { self.device.update_descriptor_sets(&vec![descriptor_write], &vec![]) };
    }
}
