#![feature(const_raw_ptr_deref, const_mut_refs)]
#![allow(unused)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(invalid_value)]

extern crate spirv_reflect;

use arrayvec::ArrayVec;
use ash::{
  extensions::{
    self,
    ext::DebugUtils,
    khr::{self, Surface, Swapchain},
  },
  vk::{
    self,
    make_api_version,
    ApplicationInfo, // },
    ApplicationInfoBuilder,
    // vk::{
    CommandPool,
    DebugUtilsMessengerEXT,
    Device,
    Framebuffer,
    ImageView,
    Instance,
    InstanceCreateInfoBuilder,
    Queue,
    SurfaceFormatKHR,
    SwapchainKHR,
    API_VERSION_1_0,

    API_VERSION_1_3,
  },
  Entry,
};

use generational_arena::Arena;
use gpu_alloc::{Config, GpuAllocator, Request, UsageFlags};
use gpu_alloc_ash::AshMemoryDevice;
use smallvec::SmallVec;
use winit::error::OsError;
use winit::{
  dpi::{LogicalPosition, LogicalSize},
  platform::run_return::EventLoopExtRunReturn,
};

use winit::{
  dpi::PhysicalSize,
  event::{
    DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent,
  },
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  window::WindowBuilder,
};

use std::{cell::{RefCell, UnsafeCell}, sync::Mutex};
use std::ptr::replace;
use std::{
  borrow::{Borrow, BorrowMut},
  cell::Cell,
  mem::MaybeUninit,
  ops::IndexMut,
  rc::Rc,
};
use std::{
  ffi::{c_void, CStr, CString},
  mem,
  os::raw::c_char,
  sync::Arc,
};


use crate::{sys::{wswapchain::WSwapchain, wcommandpool::WCommandPool}, res::{wimage::WImage, wbuffer::WBuffer, wrendertarget::WRenderTarget, wbindings::{WBindingUBO, WBindingImageArray, WBindingBufferArray}, wshader::WProgram}, wvulkan::WVulkan};

use super::wcomputepipeline::WComputePipeline;

pub struct Globals{
  pub shared_buffers_arena: *mut Arena<WBuffer>,
  pub shared_images_arena: *mut Arena<WImage>,
  pub shared_render_targets_arena: *mut Arena<WRenderTarget>,
  pub shared_ubo_arena: *mut Arena<WBindingUBO>,
  pub shared_binding_images_array: *mut WBindingImageArray,
  pub shared_binding_buffers_array: *mut WBindingBufferArray,

  pub shared_compute_pipelines: *mut Arena<WComputePipeline>,
  pub shaders_arena: *mut Arc<Mutex<Arena<WProgram>>>,
  pub w_vulkan: *mut WVulkan,
}


pub static mut GLOBALS: Globals = Globals{
  shared_buffers_arena: std::ptr::null_mut(),
  shared_images_arena: std::ptr::null_mut(),
  shared_render_targets_arena: std::ptr::null_mut(),
  shared_ubo_arena: std::ptr::null_mut(),
  shared_binding_images_array: std::ptr::null_mut(),
  shared_binding_buffers_array: std::ptr::null_mut(),
  shaders_arena: std::ptr::null_mut(),

  shared_compute_pipelines: std::ptr::null_mut(),
  w_vulkan: std::ptr::null_mut(),
};



pub const fn pipeline_library_extension_name() -> &'static ::std::ffi::CStr {
  unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"VK_KHR_pipeline_library\0") }
}


unsafe extern "system" fn debug_callback(
  _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
  _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
  p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
  _p_user_data: *mut c_void,
) -> vk::Bool32 {
  let mut s = CStr::from_ptr((*p_callback_data).p_message).to_string_lossy();

  // println!("\x1b[0;31mSO\x1b[0m");
  // _message_types
  // vk::DebugUtilsMessageTypeFlagsEXT::

  if (_message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE) {
    return vk::FALSE;
  }

  let re = regex::Regex::new(r"Validation Warning")
    .unwrap()
    .replace_all(
      &s,
      "\x1b[0;38;2;0;0;0;48;2;250;190;0m Validation Warning \x1b[0m",
    )
    .to_string();

  let re = regex::Regex::new(r"Validation Error").unwrap().replace_all(
    &re,
    "\x1b[0;38;2;0;0;0;48;2;250;0;0m Validation Error \x1b[0m",
  );

  let re = regex::Regex::new(r"(\[ .* \])")
    .unwrap()
    .replace_all(&re, "\x1b[0;34m $1 \x1b[0m");

  let re = regex::Regex::new(r"(VK_[^ ]*)")
    .unwrap()
    .replace_all(&re, "\x1b[1;36m $1 \x1b[0m");

  let re = regex::Regex::new(r"(Vk[^ ]*)")
    .unwrap()
    .replace_all(&re, "\x1b[1;32m $1 \x1b[0m");

  let re = regex::Regex::new(r"(vk[A-Z][^ ]*)")
    .unwrap()
    .replace_all(&re, "\x1b[1;33m $1 \x1b[0m");

  let re = regex::Regex::new(r"(\(http.*\))")
    .unwrap()
    .replace_all(&re, "\x1b[0;3m  $1 \x1b[0m");

  println!("{}", re);

  let mut a = 0;
  if (_message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
    a += 1;
  }

  vk::FALSE
}

// !! ---------- DEFINES ---------- //

const SHADER_VERT: &[u8] = include_bytes!("../shaders/_triangle_vert.spv");
const SHADER_FRAG: &[u8] = include_bytes!("../shaders/_triangle_frag.spv");
const FRAMES_IN_FLIGHT: usize = 2;
const APP_NAME: &str = "Vulkan";
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

// !! ---------- MAIN ---------- //
pub struct WDevice {
  #[cfg(debug_assertions)]
  pub debug_messenger: DebugUtilsMessengerEXT,
  pub instance: Arc<ash::Instance>,
  pub _entry: Entry,
  pub device: Arc<ash::Device>,
  pub allocator: GpuAllocator<vk::DeviceMemory>,
  // pub command_pool: CommandPool,
  pub pong_idx: usize,
  pub command_pools: SmallVec<[WCommandPool;2]>,
  pub descriptor_pool: vk::DescriptorPool,
  pub queue: Queue,
}


impl WDevice {
  pub fn curr_pool(&mut self)->&mut WCommandPool{
    &mut self.command_pools[self.pong_idx]
  }
  pub fn init_device_and_swapchain<'a>(window: &'a Window) -> (Self, WSwapchain) {
    
    // unsafe{
    //   println!("{}", LEVELS);
    // }
    
    
    let entry = unsafe { Entry::load().unwrap() };

    println!("{} - Vulkan Instance", APP_NAME,);

    // let app_name = CString::new(APP_NAME).unwrap();
    let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0") };

    let engine_name = CString::new("No Engine").unwrap();

    let app_info = ApplicationInfo {
      p_application_name: wtransmute!(app_name.as_ptr()),
      p_engine_name: wtransmute!(app_name.as_ptr()),
      // application_version : make_api_version(0, 1, 3, 0),
      // engine_version : make_api_version(0, 1, 3, 0),
      // application_version: 0,
      // engine_version: 0,
      application_version: make_api_version(0, 1, 3, 0),
      engine_version: make_api_version(0, 1, 3, 0),
      api_version: make_api_version(0, 1, 3, 0),
      ..Default::default()
    };

    let create_flags = vk::InstanceCreateFlags::default();

    // .application_name(app_name)
    // .application_version(make_api_version(0, 1, 3, 0))
    // .engine_name(&engine_name)
    // .engine_version(make_api_version(0, 1, 3, 0))
    // .api_version(vk::make_api_version(0, 1, 3, 0));

    // -- EXTENSIONS -- //

    let mut instance_extensions = ash_window::enumerate_required_extensions(&window)
      .unwrap()
      .to_vec();
    // sussy bakki
    // instance_extensions.push(ash::extensions::ext::DebugUtils::name().as_ptr());
    instance_extensions.push(DebugUtils::name().as_ptr());

    let vk_layer_khronos_validation =
      unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };
    let layers_names_raw: Vec<*const c_char> =
      vec![unsafe { vk_layer_khronos_validation.as_ptr() }];

    let mut instance_layers = layers_names_raw;
    

    let device_extensions = vec![
      extensions::khr::Swapchain::name().as_ptr(),
      extensions::khr::DynamicRendering::name().as_ptr(),
      extensions::khr::RayTracingPipeline::name().as_ptr(),
      extensions::khr::AccelerationStructure::name().as_ptr(),
      extensions::khr::DeferredHostOperations::name().as_ptr(),
      pipeline_library_extension_name().as_ptr(),
    ];

    // let mut device_layers: Vec<*const i8> = layers_names_raw.clone();
    let mut device_layers: Vec<*const i8> = vec![];
    // device_layers.push(LAYER_KHRONOS_VALIDATION);

    let instance_info = vk::InstanceCreateInfo::builder()
      .application_info(&app_info)
      .enabled_layer_names(&instance_layers)
      .enabled_extension_names(&instance_extensions)
      .flags(create_flags);

    let (instance, device_extensions, device_layers) = {
      (
        // Arc::new(unsafe { Instance::new(&entry, &instance_info) }.unwrap()),
        Arc::new(unsafe { entry.create_instance(&instance_info, None).unwrap() }),
        device_extensions,
        device_layers,
      )
    };

    let mut messenger_info = vk::DebugUtilsMessengerCreateInfoEXT {
      // message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
      //   | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
      //   | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,

      // message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
      //   | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
      //   | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
      message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
      message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
      pfn_user_callback: Some(debug_callback),
      ..Default::default()
    };
    messenger_info.pfn_user_callback = Some(debug_callback);

    let debug_utils_loader = DebugUtils::new(&entry, &instance);

    let debug_call_back = unsafe {
      debug_utils_loader
        .create_debug_utils_messenger(&messenger_info, None)
        .unwrap()
    };
    // let debug_messenger = wmemzeroed!();

    // #[cfg(debug_assertions)]
    // let surface = unsafe { entry.create_surface(&instance, &window, None) }.unwrap();
    let surface = unsafe { ash_window::create_surface(&entry, &instance, window, None) }.unwrap();

    let surface_loader = Surface::new(&entry, &instance);

    // let surface = ash_window::create_surface(&entry, &instance, &window, None).unwrap();

    // !! ---------- device/formats/extensions ---------- //
    // https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families
    let (physical_device, queue_family, surface_format, present_mode, device_properties) =
      unsafe { instance.enumerate_physical_devices() }
        .unwrap()
        .into_iter()
        .filter_map(|physical_device| unsafe {
          let queue_family = match instance
            .get_physical_device_queue_family_properties(physical_device)
            .into_iter()
            .enumerate()
            .position(|(i, queue_family_properties)| {
              queue_family_properties
                .queue_flags
                .contains(vk::QueueFlags::GRAPHICS)
                && surface_loader
                  .get_physical_device_surface_support(physical_device, i as u32, surface)
                  .unwrap()
            }) {
            Some(queue_family) => queue_family as u32,
            None => return None,
          };

          let formats = surface_loader
            .get_physical_device_surface_formats(physical_device, surface)
            .unwrap();
          let format = match formats
            .iter()
            .find(|surface_format| {
              (surface_format.format == vk::Format::B8G8R8A8_SRGB
                || surface_format.format == vk::Format::R8G8B8A8_SRGB)
                && surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .or_else(|| formats.get(0))
          {
            Some(surface_format) => *surface_format,
            None => return None,
          };

          let present_mode = surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface)
            .unwrap()
            .into_iter()
            .find(|present_mode| present_mode == &vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

          let supported_device_extensions = instance
            .enumerate_device_extension_properties(physical_device)
            .unwrap();
          let device_extensions_supported = device_extensions.iter().all(|device_extension| {
            let device_extension = CStr::from_ptr(*device_extension);

            supported_device_extensions.iter().any(|properties| {
              CStr::from_ptr(properties.extension_name.as_ptr()) == device_extension
            })
          });

          if !device_extensions_supported {
            return None;
          }

          let device_properties = instance.get_physical_device_properties(physical_device);

          Some((
            physical_device,
            queue_family,
            format,
            present_mode,
            device_properties,
          ))
        })
        .max_by_key(|(_, _, _, _, properties)| match properties.device_type {
          vk::PhysicalDeviceType::DISCRETE_GPU => 2,
          vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
          _ => 0,
        })
        .expect("No suitable physical device found");

    println!("Using physical device: {:?}", unsafe {
      CStr::from_ptr(device_properties.device_name.as_ptr())
    });

    // !! ---------- QUEUE AND DEVICE ---------- //
    // https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Logical_device_and_queues

    // unsafe{
    //     instance.get_physical_device_features2(physical_device,  Some(features2.build_dangling()));
    // }

    let queue_info = vec![vk::DeviceQueueCreateInfo::builder()
      .queue_family_index(queue_family)
      .queue_priorities(&[1.0])
      .build()];

    let vkfeatures = vk::PhysicalDeviceFeatures::builder().shader_float64(true);

    let mut vk1_1features = vk::PhysicalDeviceVulkan11Features::builder();
    let mut vk1_2features = vk::PhysicalDeviceVulkan12Features::builder()
      .buffer_device_address(true)
      .timeline_semaphore(true);
    let mut vk1_3features = vk::PhysicalDeviceVulkan13Features::builder()
      .dynamic_rendering(true)
      .synchronization2(true);

    let mut vk1_3dynamic_state_feature =
      vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::builder()
        .extended_dynamic_state(true)
        .build();

    let mut vk1_3dynamic_state_2_feature =
      vk::PhysicalDeviceExtendedDynamicState2FeaturesEXT::builder()
        .extended_dynamic_state2(true)
        .extended_dynamic_state2_logic_op(true)
        .extended_dynamic_state2_patch_control_points(true)
        .build();

    // vk:
    let mut vk1_3raytracing_feature = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::builder()
      .ray_tracing_pipeline(true)
      .ray_traversal_primitive_culling(true)
      .build();

    // vk::physicalDevicpipeline

    // let mut vk1_3_pipeline_library =
    //   vk::PipelineLibrary::builder()
    //     .extended_dynamic_state(true)
    //     .build();

    // vk::PhysicalDeviceExtendedDynamicState2FeaturesEXT

    // let mut mesh_shaderfeatures = vk::PhysicalDeviceMeshShaderFeaturesNV::builder().mesh_shader(true);
    // let mut graphics_pipeline_library = vk::graphicspipelinelib

    let mut features2 = vk::PhysicalDeviceFeatures2::builder()
      .features(*vkfeatures)
      .push_next(&mut vk1_1features)
      .push_next(&mut vk1_2features)
      .push_next(&mut vk1_3features)
      .push_next(&mut vk1_3dynamic_state_feature)
      .push_next(&mut vk1_3dynamic_state_2_feature)
      .push_next(&mut vk1_3raytracing_feature)
      // .extend_from(
      // &mut mesh_shaderfeatures
      // )
      // .extend_from(vk::P)
      ;

    let device_info = {
      vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&device_extensions)
        .enabled_layer_names(&device_layers)
        .push_next(&mut features2)
    };

    // let device =
    //   Arc::new(unsafe { Device.new(&instance, physical_device, &device_info) }.unwrap());
    let device =
      Arc::new(unsafe { instance.create_device(physical_device, &device_info, None) }.unwrap());

    let queue = unsafe { device.get_device_queue(queue_family, 0) };

    // let version = entry
    //     .try_enumerate_instance_version()
    //     .unwrap_or(vk::make_api_version(0, 1, 3, 0));
    let version = vk::make_api_version(0, 1, 3, 0);

    let mut allocator = unsafe {
      GpuAllocator::new(
        gpu_alloc::Config::i_am_potato(),
        gpu_alloc_ash::device_properties(&instance, version, physical_device).unwrap(),
      )
    };

    {
      let mut block = unsafe {
        allocator.alloc(
          AshMemoryDevice::wrap(&device),
          Request {
            size: 10,
            align_mask: 1,
            usage: UsageFlags::HOST_ACCESS,
            memory_types: !0,
          },
        )
      }
      .unwrap();

      let data_in = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

      unsafe { block.write_bytes(AshMemoryDevice::wrap(&device), 0, data_in) }.unwrap();
      let mut arr: Vec<u8> = vec![0; 10];

      unsafe {
        block
          .read_bytes(AshMemoryDevice::wrap(&device), 0, unsafe { &mut arr })
          .unwrap()
      };
    }
    
    let command_pools = (0..2).map(|_|{
      WCommandPool::new(&device, queue_family)
    }).collect();

    let cnts = 100;

    let unfiltered_counts = [
      (vk::DescriptorType::SAMPLER, cnts),
      (vk::DescriptorType::SAMPLED_IMAGE, cnts),
      (vk::DescriptorType::STORAGE_IMAGE, cnts),
      (vk::DescriptorType::UNIFORM_BUFFER, cnts),
      (vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, cnts),
      (vk::DescriptorType::STORAGE_BUFFER, cnts),
      (vk::DescriptorType::STORAGE_BUFFER_DYNAMIC, cnts),
    ]
    .iter()
    .cloned()
    .map(|(ty, cnt)| vk::DescriptorPoolSize {
      ty,
      descriptor_count: cnt,
    })
    .collect::<ArrayVec<_, 8>>();

    let descriptor_pool_flags = vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND
      | vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET;

    let descriptor_pool_info = unsafe {
      vk::DescriptorPoolCreateInfo::builder()
        .max_sets(100)
        .flags(descriptor_pool_flags)
        .pool_sizes(mem::transmute(unfiltered_counts.as_ref()))
    };

    let descriptor_pool =
      unsafe { device.create_descriptor_pool(&descriptor_pool_info, None) }.unwrap();

    let swapchain = WSwapchain::new(
      &device,
      &physical_device,
      &instance,
      &surface_loader,
      &surface,
      &surface_format,
      &present_mode,
      window,
      FRAMES_IN_FLIGHT
    );

    (
      WDevice {
        #[cfg(debug_assertions)]
        debug_messenger: debug_call_back,
        instance,
        _entry: entry,
        device,
        allocator,
        pong_idx: 0,
        command_pools,
        descriptor_pool,
        queue,
      },
      swapchain,
    )
  }
}
