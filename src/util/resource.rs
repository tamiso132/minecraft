use std::sync::Arc;

use ash::vk;

use crate::vulkan::VulkanContext;

use super::init;

enum Binding {
    Uniform,
    Storage,
    Texture,
    CombinedImage,
}

enum BufferType {
    Vertex = vk::BufferUsageFlags::VERTEX_BUFFER.as_raw() as isize,
    Uniform = vk::BufferUsageFlags::UNIFORM_BUFFER.as_raw() as isize,
    Storage = vk::BufferUsageFlags::STORAGE_BUFFER.as_raw() as isize,
    Index = vk::BufferUsageFlags::INDEX_BUFFER.as_raw() as isize,
}

struct Resource {
    device: Arc<ash::Device>,
    instance: Arc<ash::Instance>,

    layout: vk::DescriptorSetLayout,
    set: vk::DescriptorSet,
}

impl Resource {
    pub unsafe fn new(
        instance: Arc<ash::Instance>,
        device: Arc<ash::Device>,
        physical: vk::PhysicalDevice,
    ) -> Self {
        let limits = unsafe { instance.get_physical_device_properties(physical).limits };

        let pool_sizes = vec![
            init::descriptor_pool_size(
                vk::DescriptorType::SAMPLED_IMAGE,
                limits.max_descriptor_set_sampled_images,
            ),
            init::descriptor_pool_size(
                vk::DescriptorType::STORAGE_BUFFER,
                limits.max_descriptor_set_storage_buffers,
            ),
            init::descriptor_pool_size(
                vk::DescriptorType::UNIFORM_BUFFER,
                limits.max_descriptor_set_uniform_buffers,
            ),
            init::descriptor_pool_size(vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 200),
        ];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND_EXT);

        let descriptor_pool = device
            .create_descriptor_pool(&descriptor_pool_info, None)
            .unwrap();

        let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = vec![];
        let mut set_layout_binding_flags = vec![];
        for i in 0..pool_sizes.len() {
            bindings.push(
                init::descriptor_set_layout_binding(
                    i as u32,
                    pool_sizes[i].ty,
                    pool_sizes[i].descriptor_count,
                    vk::ShaderStageFlags::ALL,
                )
                .clone(),
            );

            set_layout_binding_flags.push(
                vk::DescriptorBindingFlags::PARTIALLY_BOUND
                    | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            );
        }

        let mut set_layout_binding_f = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default()
            .binding_flags(&set_layout_binding_flags);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .push_next(&mut set_layout_binding_f);

        let set_layout = device
            .create_descriptor_set_layout(&layout_info, None)
            .unwrap();

        let descriptor_set = device
            .allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&[set_layout]),
            )
            .unwrap()[0];
        Self {
            device: device.clone(),
            instance: instance.clone(),
            layout: set_layout,
            set: descriptor_set,
        }
    }

    pub fn create_buffer() {}
}
