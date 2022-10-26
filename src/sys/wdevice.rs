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
    ApplicationInfo, 
    ApplicationInfoBuilder,
    CommandPool,
    DebugUtilsMessengerEXT,
    Device,
    Extent2D,
    Framebuffer,
    ImageView,
    Instance,
    InstanceCreateInfoBuilder,
    Queue,
    SurfaceFormatKHR,
    SwapchainKHR,
    API_VERSION_1_0,

    API_VERSION_1_3, InstanceFnV1_0, InstanceCreateInfo,
  },
  Entry,
};

use generational_arena::Arena;
use gpu_alloc::{Config, GpuAllocator, Request, UsageFlags};
use gpu_alloc_ash::AshMemoryDevice;

use imgui::{FontConfig, FontGlyphRanges, FontSource, sys::*};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use smallvec::SmallVec;
use winit::error::OsError;
use winit::{
  dpi::{LogicalPosition, LogicalSize},
  platform::run_return::EventLoopExtRunReturn,
};

use imgui_rs_vulkan_renderer::{self, Options};

use winit::{
  dpi::PhysicalSize,
  event::{
    DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent,
  },
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  window::WindowBuilder,
};

// use ;
use std::{ptr::replace, path::Path, fs};
use std::{
  borrow::{Borrow, BorrowMut},
  cell::Cell,
  mem::MaybeUninit,
  ops::IndexMut,
  rc::Rc,
};
use std::{
  cell::{RefCell, UnsafeCell},
  mem::ManuallyDrop,
  sync::Mutex,
};
use std::{
  ffi::{c_void, CStr, CString},
  mem,
  os::raw::c_char,
  sync::Arc,
};

use crate::{
  res::{
    buff::wbuffer::WBuffer,
    img::wimage::WImage,
    img::{wrendertarget::WRenderTarget, self},
    wbindings::{WBindingBufferArray, WBindingImageArray, WBindingUBO},
    wshader::WProgram,
  },
  sys::{wcommandpool::WCommandPool, wswapchain::WSwapchain},
  wvulkan::WVulkan,
};

use super::{
  warenaitems::{WAIdxImage, WArenaItem},
  wbarr::WBarr,
  wcommandencoder::WCommandEncoder,
  wcomputepipeline::WComputePipeline,
  wrenderpipeline::WRenderPipeline,
};

pub struct Globals {
  pub shared_buffers_arena: *mut Arena<WBuffer>,
  pub shared_images_arena: *mut Arena<WImage>,
  pub shared_render_targets_arena: *mut Arena<WRenderTarget>,
  pub shared_ubo_arena: *mut Arena<WBindingUBO>,
  pub shared_binding_images_array: *mut WBindingImageArray,
  pub shared_binding_buffers_array: *mut WBindingBufferArray,

  pub shared_compute_pipelines: *mut Arena<WComputePipeline>,
  pub shared_render_pipelines: *mut Arena<WRenderPipeline>,
  pub shader_programs_arena: *mut Arena<WProgram>,
  pub w_vulkan: *mut WVulkan,

  pub imgui: *mut RefCell<imgui::Context>,

  pub compiler: *mut shaderc::Compiler,
}

pub static mut GLOBALS: Globals = Globals {
  shared_buffers_arena: std::ptr::null_mut(),
  shared_images_arena: std::ptr::null_mut(),
  shared_render_targets_arena: std::ptr::null_mut(),
  shared_ubo_arena: std::ptr::null_mut(),
  shared_binding_images_array: std::ptr::null_mut(),
  shared_binding_buffers_array: std::ptr::null_mut(),
  shader_programs_arena: std::ptr::null_mut(),

  shared_compute_pipelines: std::ptr::null_mut(),
  shared_render_pipelines: std::ptr::null_mut(),
  w_vulkan: std::ptr::null_mut(),

  imgui: std::ptr::null_mut(),

  compiler: std::ptr::null_mut(),
};

use lazy_static::lazy_static;

pub const fn pipeline_library_extension_name() -> &'static ::std::ffi::CStr {
  unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"VK_KHR_pipeline_library\0") }
}

pub const fn shader_non_semantic_info_extension_name() -> &'static ::std::ffi::CStr {
  unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"VK_KHR_shader_non_semantic_info\0") }
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
    println!("{}", a);
  }

  vk::FALSE
}

// !! ---------- DEFINES ---------- //

const FRAMES_IN_FLIGHT: usize = 2;
const APP_NAME: &str = "Vulkan";
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

// !! ---------- MAIN ---------- //
pub struct WDevice {
  #[cfg(debug_assertions)]
  pub debug_messenger: DebugUtilsMessengerEXT,
  pub instance: ash::Instance,
  pub _entry: Entry,
  pub device: ash::Device,
  pub allocator: GpuAllocator<vk::DeviceMemory>,
  pub new_allocator: Arc<Mutex<gpu_allocator::vulkan::Allocator>>,

  // pub egui_integration:
  //   egui_winit_ash_integration::Integration<Arc<Mutex<gpu_allocator::vulkan::Allocator>>>,
  pub imgui_renderer : imgui_rs_vulkan_renderer::Renderer,

  pub platform: WinitPlatform,

  // pub command_pool: CommandPool,
  pub pong_idx: usize,
  pub command_pools: SmallVec<[WCommandPool; 2]>,
  pub descriptor_pool: vk::DescriptorPool,
  pub queue: Queue,
}

impl WDevice {
  pub fn curr_pool(&mut self) -> &mut WCommandPool {
    &mut self.command_pools[self.pong_idx]
  }
  pub fn init_device_and_swapchain<'a>(window: &'a Window) -> (Self, WSwapchain) {
    let entry = unsafe { Entry::load().unwrap() };

    println!("{} - Vulkan Instance", APP_NAME,);

    let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0") };

    let engine_name = CString::new("No Engine").unwrap();

    let app_info = ApplicationInfo {
      p_application_name: wtransmute!(app_name.as_ptr()),
      p_engine_name: wtransmute!(app_name.as_ptr()),
      application_version: make_api_version(0, 1, 3, 0),
      engine_version: make_api_version(0, 1, 3, 0),
      api_version: make_api_version(0, 1, 3, 0),
      ..Default::default()
    };

    let create_flags = vk::InstanceCreateFlags::default();

    // -- EXTENSIONS -- //

    let mut instance_extensions = ash_window::enumerate_required_extensions(&window)
      .unwrap()
      .to_vec();

    instance_extensions.push(DebugUtils::name().as_ptr());
    

    let vk_layer_khronos_validation =
      unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };
    let layers_names_raw: Vec<*const c_char> =
      vec![unsafe { vk_layer_khronos_validation.as_ptr() }];

    let mut instance_layers = layers_names_raw;

    // unsafe extern "system" fn(
    //     flags: DebugReportFlagsEXT,
    //     object_type: DebugReportObjectTypeEXT,
    //     object: u64,
    //     location: usize,
    //     message_code: i32,
    //     p_layer_prefix: *const c_char,
    //     p_message: *const c_char,
    //     p_user_data: *mut c_void,
    // ) -> Bool32{
    // };
    
    // let debug_report_callback = vk::DebugReportCallbackCreateInfoEXT::builder().pfn_callback(b

    // ).build();

    let device_extensions = vec![
      extensions::khr::Swapchain::name().as_ptr(),
      extensions::khr::DynamicRendering::name().as_ptr(),
      extensions::khr::RayTracingPipeline::name().as_ptr(),
      extensions::khr::AccelerationStructure::name().as_ptr(),
      extensions::khr::DeferredHostOperations::name().as_ptr(),
      extensions::khr::CopyCommands2::name().as_ptr(),
      shader_non_semantic_info_extension_name().as_ptr(),
      pipeline_library_extension_name().as_ptr(),
    ];

    let mut device_layers: Vec<*const i8> = vec![];

    let mut validation_features = vk::ValidationFeaturesEXT::builder()
      .enabled_validation_features(&[
        vk::ValidationFeatureEnableEXT::DEBUG_PRINTF,
      ]).build();

    let instance_info = vk::InstanceCreateInfo::builder()
      .application_info(&app_info)
      .enabled_layer_names(&instance_layers)
      .enabled_extension_names(&instance_extensions)
      .flags(create_flags)
      .push_next(&mut validation_features)
      .build();

    // // beautiful ☺
    // let instance_info_ptr = unsafe{(&instance_info as *const InstanceCreateInfo)};
    // validation_features.p_next = wtransmute!(instance_info_ptr);
    
    // instance_info.p_next = w ;

    let (instance, device_extensions, device_layers) = {
      (
        unsafe { entry.create_instance(&instance_info, None).unwrap() },
        device_extensions,
        device_layers,
      )
    };

    let mut messenger_info = vk::DebugUtilsMessengerCreateInfoEXT {
      //   | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
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

    let surface = unsafe { ash_window::create_surface(&entry, &instance, window, None) }.unwrap();

    let surface_loader = Surface::new(&entry, &instance);

    // !! ---------- device/formats/extensions ---------- //
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

    let queue_info = vec![vk::DeviceQueueCreateInfo::builder()
      .queue_family_index(queue_family)
      .queue_priorities(&[1.0])
      .build()];

    let vkfeatures = vk::PhysicalDeviceFeatures::builder()
      .shader_float64(true)
      .shader_storage_image_read_without_format(true)
      .shader_storage_image_write_without_format(true);

    let mut vk1_1features = vk::PhysicalDeviceVulkan11Features::builder();
    let mut vk1_2features = vk::PhysicalDeviceVulkan12Features::builder()
      .buffer_device_address(true)
      .timeline_semaphore(true)
      .uniform_buffer_standard_layout(true)
      .shader_int8(true)
      .storage_push_constant8(true)
      .shader_float16(true)
      .scalar_block_layout(true)
      .runtime_descriptor_array(true);

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
      .push_next(&mut vk1_3raytracing_feature);

    let device_info = {
      vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&device_extensions)
        .enabled_layer_names(&device_layers)
        .push_next(&mut features2)
    };

    let device = unsafe { instance.create_device(physical_device, &device_info, None) }.unwrap();

    let queue = unsafe { device.get_device_queue(queue_family, 0) };

    let version = vk::make_api_version(0, 1, 3, 0);

    let mut allocator = unsafe {
      GpuAllocator::new(
        gpu_alloc::Config::i_am_potato(),
        gpu_alloc_ash::device_properties(&instance, version, physical_device).unwrap(),
      )
    };

    let mut new_allocator = {
      gpu_allocator::vulkan::Allocator::new(&gpu_allocator::vulkan::AllocatorCreateDesc {
        // instance: instance.clone(),
        instance: instance.clone(),
        device: device.clone(),
        physical_device,
        debug_settings: Default::default(),
        buffer_device_address: true,
      })
      .unwrap()
    };
    let new_allocator = Arc::new(Mutex::new(new_allocator));

    let command_pools: SmallVec<[WCommandPool; 2]> = (0..2)
      .map(|_| WCommandPool::new(&device, queue_family))
      .collect();

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
      FRAMES_IN_FLIGHT,
    );

    // !! -- IMGUI INIT --

    unsafe{
      let mut imgui = Box::new(RefCell::new(imgui::Context::create()));
      let mut imgui = Box::into_raw(imgui);

      std::mem::replace(&mut GLOBALS.imgui, imgui);
    }
    

    let mut imgui = unsafe{(*GLOBALS.imgui).borrow_mut()};

    imgui.style_mut().use_dark_colors();

    let imgui_path = Path::new(&"./imgui.ini".to_string()).to_path_buf();
    imgui.set_ini_filename(imgui_path.clone());
    imgui.load_ini_settings(&fs::read_to_string(imgui_path).unwrap());

    let im_style = imgui.style_mut();
    let im_colours = &mut im_style.colors;
              

    im_colours[ImGuiCol_Text as usize] = [1.00, 1.00, 1.00, 0.95];
    im_colours[ImGuiCol_TextDisabled as usize] = [0.50, 0.50, 0.50, 1.00];
    im_colours[ImGuiCol_WindowBg as usize] = [0.13, 0.12, 0.12, 1.00];

    im_colours[ImGuiCol_ChildBg as usize] = [1.00, 1.00, 1.00, 0.00];
    im_colours[ImGuiCol_PopupBg as usize] = [0.05, 0.05, 0.05, 0.94];
    im_colours[ImGuiCol_Border as usize] = [0.53, 0.53, 0.53, 0.46];
    im_colours[ImGuiCol_BorderShadow as usize] = [0.00, 0.00, 0.00, 0.00];
    im_colours[ImGuiCol_FrameBg as usize] = [0.00, 0.00, 0.00, 0.85];
    im_colours[ImGuiCol_FrameBgHovered as usize] = [0.22, 0.22, 0.22, 0.40];
    im_colours[ImGuiCol_FrameBgActive as usize] = [0.16, 0.16, 0.16, 0.53];
    im_colours[ImGuiCol_TitleBg as usize] = [0.00, 0.00, 0.00, 1.00];
    im_colours[ImGuiCol_TitleBgActive as usize] = [0.00, 0.00, 0.00, 1.00];
    im_colours[ImGuiCol_TitleBgCollapsed as usize] = [0.00, 0.00, 0.00, 0.51];
    im_colours[ImGuiCol_MenuBarBg as usize] = [0.12, 0.12, 0.12, 1.00];
    im_colours[ImGuiCol_ScrollbarBg as usize] = [0.02, 0.02, 0.02, 0.53];
    im_colours[ImGuiCol_ScrollbarGrab as usize] = [0.31, 0.31, 0.31, 1.00];
    im_colours[ImGuiCol_ScrollbarGrabHovered as usize] = [0.41, 0.41, 0.41, 1.00];
    im_colours[ImGuiCol_ScrollbarGrabActive as usize] = [0.48, 0.48, 0.48, 1.00];
    im_colours[ImGuiCol_CheckMark as usize] = [0.79, 0.79, 0.79, 1.00];
    im_colours[ImGuiCol_SliderGrab as usize] = [0.48, 0.47, 0.47, 0.91];
    im_colours[ImGuiCol_SliderGrabActive as usize] = [0.56, 0.55, 0.55, 0.62];
    im_colours[ImGuiCol_Button as usize] = [0.50, 0.50, 0.50, 0.63];
    im_colours[ImGuiCol_ButtonHovered as usize] = [0.67, 0.67, 0.68, 0.63];
    im_colours[ImGuiCol_ButtonActive as usize] = [0.4, 0.26, 0.26, 0.63];
    im_colours[ImGuiCol_Header as usize] = [0.54, 0.54, 0.54, 0.58];
    im_colours[ImGuiCol_HeaderHovered as usize] = [0.64, 0.65, 0.65, 0.80];
    im_colours[ImGuiCol_HeaderActive as usize] = [0.25, 0.25, 0.25, 0.80];
    im_colours[ImGuiCol_Separator as usize] = [0.58, 0.58, 0.58, 0.50];
    im_colours[ImGuiCol_SeparatorHovered as usize] = [0.81, 0.81, 0.81, 0.64];
    im_colours[ImGuiCol_SeparatorActive as usize] = [0.81, 0.81, 0.81, 0.64];
    im_colours[ImGuiCol_ResizeGrip as usize] = [0.87, 0.87, 0.87, 0.53];
    im_colours[ImGuiCol_ResizeGripHovered as usize] = [0.87, 0.87, 0.87, 0.74];
    im_colours[ImGuiCol_ResizeGripActive as usize] = [0.87, 0.87, 0.87, 0.74];
    im_colours[ImGuiCol_Tab as usize] = [0.01, 0.01, 0.01, 0.86];
    im_colours[ImGuiCol_TabHovered as usize] = [0.29, 0.29, 0.29, 1.00];
    im_colours[ImGuiCol_TabActive as usize] = [0.31, 0.31, 0.31, 1.00];
    im_colours[ImGuiCol_TabUnfocused as usize] = [0.02, 0.02, 0.02, 1.00];
    im_colours[ImGuiCol_TabUnfocusedActive as usize] = [0.19, 0.19, 0.19, 1.00];
  //            imguiStys/colors[ImGuiCol.DockingPreview] = new float[]{0.38f, 0.48f, 0.60f, 1.00f};
  //            imguiStys/colors[ImGuiCol.DockingEmptyBg] = new float[]{0.20f, 0.20f, 0.20f, 1.00f};
    im_colours[ImGuiCol_PlotLines as usize] = [0.61, 0.61, 0.61, 1.00];
    im_colours[ImGuiCol_PlotLinesHovered as usize] = [0.68, 0.68, 0.68, 1.00];
    im_colours[ImGuiCol_PlotHistogram as usize] = [0.90, 0.77, 0.33, 1.00];
    im_colours[ImGuiCol_PlotHistogramHovered as usize] = [0.87, 0.55, 0.08, 1.00];
    im_colours[ImGuiCol_TextSelectedBg as usize] = [0.47, 0.60, 0.76, 0.47];
    im_colours[ImGuiCol_DragDropTarget as usize] = [0.58, 0.58, 0.58, 0.90];
    im_colours[ImGuiCol_NavHighlight as usize] = [0.60, 0.60, 0.60, 1.00];
    im_colours[ImGuiCol_NavWindowingHighlight as usize] = [1.00, 1.00, 1.00, 0.70];
    im_colours[ImGuiCol_NavWindowingDimBg as usize] = [0.80, 0.80, 0.80, 0.20];
    im_colours[ImGuiCol_ModalWindowDimBg as usize] = [0.80, 0.80, 0.80, 0.35];
    im_colours[ImGuiCol_WindowBg as usize] = [0.05, 0.05, 0.05, 0.5];
    im_colours[ImGuiCol_PopupBg as usize] = [0.05, 0.05, 0.05, 0.5];
    im_colours[ImGuiCol_TitleBg as usize] = [0.05, 0.05, 0.05, 0.5];
    im_colours[ImGuiCol_TitleBgActive as usize] = [0.05, 0.05, 0.05, 0.5];
    im_colours[ImGuiCol_Border as usize][3] = 0.5;

    let mut platform = WinitPlatform::init(&mut imgui);

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
      FontSource::DefaultFontData {
        config: Some(FontConfig {
          size_pixels: font_size,
          ..FontConfig::default()
        }),
      },
      FontSource::TtfData {
        data: include_bytes!("../../mplus-1p-regular.ttf"),
        size_pixels: font_size,
        config: Some(FontConfig {
          rasterizer_multiply: 1.75,
          glyph_ranges: FontGlyphRanges::japanese(),
          ..FontConfig::default()
        }),
      },
    ]);
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

    let imgui_renderer = imgui_rs_vulkan_renderer::Renderer::with_gpu_allocator(
      new_allocator.clone(),
      device.clone(),
      queue,
      command_pools[0].command_pool,
      imgui_rs_vulkan_renderer::DynamicRendering {
        color_attachment_format: swapchain.default_render_targets[0].images[0].format,
        depth_attachment_format: None,
      },
      &mut imgui,
      Some(Options {
        in_flight_frames: 2,
        enable_depth_test: false,
        enable_depth_write: false,
        ..Default::default()
      }),
    ).unwrap();


    (
      WDevice {
        #[cfg(debug_assertions)]
        debug_messenger: debug_call_back,
        instance,
        _entry: entry,
        device,
        allocator,
        new_allocator,
        pong_idx: 0,
        command_pools,
        descriptor_pool,
        queue,
        platform,
        imgui_renderer,
      },
      swapchain,
    )
  }

  pub fn blit_image_to_swapchain(
    w: &mut WVulkan,
    command_encoder: &mut WCommandEncoder,
    mut src_img: WAIdxImage,
    swapchain_rt: &WRenderTarget,
  ) {
    let rt = swapchain_rt;

    // BLIT
    {
      let cmd_buff = command_encoder.get_and_begin_buff(&mut w.w_device);
      let src_img = src_img.get_mut();
      let dst_img = &rt.images[0];

      // let mut barr_src = WBarr::new_image_barr();
      // barr_src.old_layout(src_img.descriptor_image_info.image_layout);
      // barr_src.new_layout(src_img.descriptor_image_info.image_layout);

      let barr_dst_in = WBarr::new_image_barr()
        .old_layout(dst_img.descriptor_image_info.image_layout)
        // .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .image(dst_img.handle)
        .src_access(vk::AccessFlags2::MEMORY_READ)
        .dst_access(vk::AccessFlags2::TRANSFER_READ)
        .src_stage(vk::PipelineStageFlags2::TRANSFER)
        .dst_stage(vk::PipelineStageFlags2::TRANSFER)
        .run_on_cmd_buff(&w.w_device, cmd_buff);

      // let barr_src_in = WBarr::new_image_barr()
      //   .old_layout(src_img.descriptor_image_info.image_layout)
      //   .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
      //   .image(src_img.handle)
      //   .src_access(vk::AccessFlags2::MEMORY_READ)
      //   .dst_access(vk::AccessFlags2::TRANSFER_WRITE)
      //   .src_stage(vk::PipelineStageFlags2::TRANSFER)
      //   .dst_stage(vk::PipelineStageFlags2::TRANSFER)
      //   .run_on_cmd_buff(&w.w_device, cmd_buff);

      let blank_sz = vk::Offset3D::builder().build();
      let blit_sz = vk::Offset3D::builder()
        .x(src_img.resx as i32)
        .y(src_img.resy as i32)
        .z(1)
        .build();

      let subresource_layers = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .layer_count(1)
        .build();

      let region = vk::ImageBlit2::builder()
        .src_offsets([blank_sz, blit_sz])
        .dst_offsets([blank_sz, blit_sz])
        .src_subresource(subresource_layers)
        .dst_subresource(subresource_layers)
        .build();

      let blit_image_info = vk::BlitImageInfo2::builder()
        .src_image(src_img.handle)
        .dst_image(dst_img.handle)
        .src_image_layout(src_img.descriptor_image_info.image_layout)
        .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .regions(&[region])
        .filter(vk::Filter::NEAREST)
        .build();
      unsafe {
        w.w_device
          .device
          .cmd_blit_image2(cmd_buff, &blit_image_info);
      }

      let barr_dst_out = WBarr::new_image_barr()
        .image(dst_img.handle)
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .src_access(vk::AccessFlags2::TRANSFER_READ)
        .dst_access(vk::AccessFlags2::MEMORY_READ)
        .src_stage(vk::PipelineStageFlags2::TRANSFER)
        .dst_stage(vk::PipelineStageFlags2::TRANSFER)
        .run_on_cmd_buff(&w.w_device, cmd_buff);

      // let barr_src_out = WBarr::new_image_barr()
      //   .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
      //   .new_layout(src_img.descriptor_image_info.image_layout)
      //   .image(src_img.handle)
      //   .src_access(vk::AccessFlags2::MEMORY_READ)
      //   .dst_access(vk::AccessFlags2::TRANSFER_WRITE)
      //   .src_stage(vk::PipelineStageFlags2::TRANSFER)
      //   .dst_stage(vk::PipelineStageFlags2::TRANSFER)
      //   .run_on_cmd_buff(&w.w_device, cmd_buff);

      command_encoder.end_and_push_buff(&mut w.w_device, cmd_buff);
    }
  }
}
