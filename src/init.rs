use ash::vk::{self, ImageTiling, MemoryPropertyFlags};
use vk_mem::AllocationCreateFlags;

pub fn image_subresource_info() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(0)
        .base_array_layer(0)
        .layer_count(1)
}

pub fn image_components_rgba() -> vk::ComponentMapping {
    vk::ComponentMapping {
        r: vk::ComponentSwizzle::R,
        g: vk::ComponentSwizzle::G,
        b: vk::ComponentSwizzle::B,
        a: vk::ComponentSwizzle::A,
    }
}

pub fn image_info(
    extent: vk::Extent2D,
    pixel_size: u32,
    memory_type: MemoryPropertyFlags,
    format: vk::Format,
    image_usage: vk::ImageUsageFlags,
) -> (vk::ImageCreateInfo<'static>, vk_mem::AllocationCreateInfo) {
    let size_allocate = pixel_size * extent.height * extent.width;

    let alloc_info = vk_mem::AllocationCreateInfo {
        flags: AllocationCreateFlags::empty(),
        usage: vk_mem::MemoryUsage::Unknown,
        required_flags: memory_type,
        preferred_flags: MemoryPropertyFlags::empty(),
        memory_type_bits: 0,
        user_data: 0,
        priority: 0.0,
    };

    let image_info = vk::ImageCreateInfo::default()
        .extent(extent.clone().into())
        .array_layers(1)
        .mip_levels(1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .format(format.clone())
        .image_type(vk::ImageType::TYPE_2D)
        .tiling(ImageTiling::OPTIMAL)
        .samples(vk::SampleCountFlags::TYPE_1)
        .usage(image_usage);

    (image_info, alloc_info)
}

pub fn image_view_info(image:vk::Image){

}