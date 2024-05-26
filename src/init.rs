use std::{
    borrow::Cow,
    ffi::{self, CStr, CString},
};

use ash::{
    ext::debug_utils,
    vk::{self, ApplicationInfo, QueueFlags},
    Entry,
};
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
            queues.iter().enumerate().map(|(index, queue_info)| {
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
        queues.iter().enumerate().map(|(index, queue_info)| {
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

struct DeviceBuilder<'a> {
    features: vk::PhysicalDeviceFeatures,
    features_13: vk::PhysicalDeviceVulkan13Features<'a>,
    extensions: Vec<CString>,
    device_info: vk::DeviceCreateInfo<'a>,
    physical: vk::PhysicalDevice,
    instance: ash::Instance,

    transfer_queue: TKQueue,
    present_queue: TKQueue,
    graphic_queue: TKQueue,
}

impl<'a> DeviceBuilder<'a> {
    fn select_physical_device(&mut self, instance: ash::Instance) {
        let has_queues_required: bool = false;

        unsafe {
            let physical_devices = instance
                .clone()
                .enumerate_physical_devices()
                .expect("no vulkan supported gpu");

            for physical in physical_devices {
                let mut graphic =
                    TKQueue::find_queue(instance.clone(), physical, QueueFlags::GRAPHICS);
                let mut transfer = TKQueue::find_transfer_only(instance.clone(), physical);

                if (graphic.is_some() && transfer.is_some()) {
                    self.transfer_queue = transfer.unwrap();
                    self.graphic_queue = graphic.unwrap();
                    break;
                }
            }

            if (!has_queues_required) {
                panic!("None of the Vulkan supported gpus have the required queues");
            }
        }
    }

    fn image_cube_array(&mut self) {
        self.features.image_cube_array = 1;
    }

    fn sampler_anisotropy(&mut self) {
        self.features.sampler_anisotropy = 1;
    }

    fn dynamic_rendering(&mut self) {
        self.features_13.dynamic_rendering = 1;
        self.extensions
            .push(CString::new("VK_KHR_dynamic_rendering").unwrap());
    }

    fn build() {}
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
