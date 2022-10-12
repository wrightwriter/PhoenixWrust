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

use notify::{
  Error, Event, ReadDirectoryChangesWatcher, RecommendedWatcher, RecursiveMode, Watcher,
};

use gpu_alloc::{Config, GpuAllocator, Request, UsageFlags};

use generational_arena::Arena;
use smallvec::SmallVec;

use crate::{
  abs::wcomputepass::WComputePass,
  abs::wthing::WThing,
  res::wbuffer::WBuffer,
  res::wimage::WImage,
  res::wshader::WProgram,
  res::{wrendertarget::WRenderTarget, wshader::WShaderEnumPipelineBind},
  wmemzeroed, wtransmute,
};
use crate::{
  c_str,
  res::{
    self,
    wbindings::{WBindingBufferArray, WBindingImageArray, WBindingUBO},
    wpongabletrait::WPongableTrait,
    wrendertarget::WRenderTargetCreateInfo,
    wshader::WShader,
  },
  sys::wbindgroup::WBindGroup,
  sys::wdevice::WDevice,
  sys::wswapchain::{self, WSwapchain},
};
use gpu_alloc_ash::AshMemoryDevice;
use winit::error::OsError;
use winit::{
  dpi::{LogicalPosition, LogicalSize},
  platform::run_return::EventLoopExtRunReturn,
};

use winit::{
  dpi::PhysicalSize,
  event::{DeviceEvent, ElementState, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  window::WindowBuilder,
};

use std::{ptr::replace, cell::{UnsafeCell}};
use std::{
  borrow::{Borrow, BorrowMut},
  cell::Cell,
  mem::MaybeUninit,
  ops::IndexMut,
  rc::Rc,
};
use std::{
  cell::RefCell,
  ops::Deref,
  path::Path,
  sync::{
    mpsc::{Receiver, Sender},
    Mutex,
  },
};
use std::{
  ffi::{c_void, CStr, CString},
  mem,
  os::raw::c_char,
  sync::Arc,
};

use super::{
  wcomputepipeline::WComputePipeline,
  wdevice::{Globals, GLOBALS},
  wrenderpipeline::WRenderPipeline,
};

#[derive(Debug, Copy, Clone)]
pub enum WEnumBind {
  WAIdxImage(WAIdxImage),
  // WBindingImageArray(Rc<RefCell<WBindingImageArray>>),
  WAIdxUbo(WAIdxUbo),
  WAIdxBuffer(WAIdxBuffer),
}

pub trait WBinding {
  fn get_type(&self) -> WBindType;
}

pub trait WArenaItem<T> {
  fn get_arena_idx(&self) -> generational_arena::Index;
  fn get_mut(&self) -> &mut T;
}

// -----------
#[derive(Debug, Copy, Clone)]
pub struct WAIdxBindGroup {
  pub idx: generational_arena::Index,
}
impl WArenaItem<WBindGroup> for WAIdxBindGroup {
  fn get_arena_idx(&self) -> generational_arena::Index {
    self.idx
  }
  fn get_mut(&self) -> &mut WBindGroup {
    unsafe {
      let b = &mut *std::ptr::null_mut() as &mut WBindGroup;
      b
    }
  }
}
// -----------

#[derive(Debug, Copy, Clone)]
pub struct WAIdxImage {
  pub idx: generational_arena::Index,
}
impl WArenaItem<WImage> for WAIdxImage {
  fn get_arena_idx(&self) -> generational_arena::Index {
    self.idx
  }

  fn get_mut(&self) -> &mut WImage {
    unsafe { w_ptr_to_mut_ref!(GLOBALS.shared_images_arena)[self.idx].borrow_mut() }
  }
}
impl WBinding for WAIdxImage {
  fn get_type(&self) -> WBindType {
    WBindType::WBindTypeImage
  }
}

// -----------

#[derive(Debug, Copy, Clone)]
pub struct WAIdxRenderPipeline {
  pub idx: generational_arena::Index,
}

impl WAIdxRenderPipeline{
  // fn get_ptr(&self) -> *mut WRenderPipeline {
  //   unsafe { (*GLOBALS.shared_render_pipelines)[self.idx]}
  // }
}
impl WArenaItem<WRenderPipeline> for WAIdxRenderPipeline {
  fn get_arena_idx(&self) -> generational_arena::Index {
    self.idx
  }
  fn get_mut(&self) -> &mut WRenderPipeline {
    unsafe { w_ptr_to_mut_ref!(GLOBALS.shared_render_pipelines)[self.idx].borrow_mut() }
  }

}

// -----------

#[derive(Debug, Copy, Clone)]
pub struct WAIdxComputePipeline {
  pub idx: generational_arena::Index,
}
impl WArenaItem<WComputePipeline> for WAIdxComputePipeline {
  fn get_arena_idx(&self) -> generational_arena::Index {
    self.idx
  }
  fn get_mut(&self) -> &mut WComputePipeline {
    unsafe { w_ptr_to_mut_ref!(GLOBALS.shared_compute_pipelines)[self.idx].borrow_mut() }
  }
}

// -----------

#[derive(Debug, Copy, Clone)]
pub struct WAIdxShaderProgram {
  pub idx: generational_arena::Index,
}
impl WArenaItem<WProgram> for WAIdxShaderProgram {
  fn get_arena_idx(&self) -> generational_arena::Index {
    self.idx
  }
  fn get_mut(&self) -> &mut WProgram {
    unsafe {
      w_ptr_to_mut_ref!(GLOBALS.shader_programs_arena).borrow_mut()[self.idx].borrow_mut()
      // let b = &mut *std::ptr::null_mut() as &mut WProgram;
      // b
    }
  }
}

// -----------

#[derive(Debug, Copy, Clone)]
pub struct WAIdxUbo {
  pub idx: generational_arena::Index,
}
impl WArenaItem<WBindingUBO> for WAIdxUbo {
  fn get_arena_idx(&self) -> generational_arena::Index {
    self.idx
  }
  fn get_mut(&self) -> &mut WBindingUBO {
    unsafe {
      w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[self.idx].borrow_mut()
      // wtransmute!(std::ptr::null_mut()) as &mut WBindingUBO
    }
  }
}
impl WBinding for WAIdxUbo {
  fn get_type(&self) -> WBindType {
    WBindType::WBindTypeUbo
  }
}

// // -----------

#[derive(Debug, Copy, Clone)]
pub struct WAIdxBuffer {
  pub idx: generational_arena::Index,
}
impl WArenaItem<WBuffer> for WAIdxBuffer {
  fn get_arena_idx(&self) -> generational_arena::Index {
    self.idx
  }

  fn get_mut(&self) -> &mut WBuffer {
    unsafe {
      w_ptr_to_mut_ref!(GLOBALS.shared_buffers_arena)[self.idx].borrow_mut()
      // wtransmute!(std::ptr::null_mut()) as &mut WBuffer
    }
  }
}
impl WBinding for WAIdxBuffer {
  fn get_type(&self) -> WBindType {
    WBindType::WBindTypeBuffer
  }
}

// // -----------

#[derive(Debug, Copy, Clone)]
pub struct WAIdxRt {
  pub idx: generational_arena::Index,
}
impl WArenaItem<WRenderTarget> for WAIdxRt {
  fn get_arena_idx(&self) -> generational_arena::Index {
    self.idx
  }

  fn get_mut(&self) -> &mut WRenderTarget {
    unsafe {
      w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena)[self.idx].borrow_mut()
      // wtransmute!(std::ptr::null_mut()) as &mut WRenderTarget
    }
  }
}

pub enum WBindType {
  WBindTypeImage,
  WBindTypeUbo,
  WBindTypeBuffer,
}

pub struct WTechLead {
  // pub shared_images_arena: Arena<WImage>,
  // pub shared_buffers_arena: Arena<WBuffer>,
  // pub shared_binding_images_array: Rc<RefCell<WBindingImageArray>>,
  // pub shared_binding_buffers_array: Rc<RefCell<WBindingBufferArray>>,
}

impl WTechLead {
  // TODO: remove
  pub fn get_rt(
    &mut self,
    rt: &WAIdxRt,
  ) -> &mut WRenderTarget {
    w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena)[rt.idx].borrow_mut()
  }

  pub fn pong_all(&mut self) {
    unsafe {
      for __ in (&mut *GLOBALS.shared_buffers_arena) {
        let buff = __.1;
        buff.pong();
      }
    }

    unsafe {
      for __ in (&mut *GLOBALS.shared_ubo_arena) {
        let buff = __.1;
        buff.pong();
      }
    };

    unsafe {
      for __ in (&mut *GLOBALS.shared_render_targets_arena) {
        let rt = __.1;
        rt.pong();
      }
    }
  }

  pub fn new(w_device: &mut WDevice) -> Self {
    // -- init images arena
    let shared_images_arena = unsafe {
      GLOBALS.shared_images_arena = ptralloc!(Arena<WImage>);
      std::ptr::write(GLOBALS.shared_images_arena, Arena::new());
      &mut *GLOBALS.shared_images_arena
    };

    let dummy_image_idx = {
      let mut img = WImage::new(
        &w_device.device,
        &mut w_device.allocator,
        vk::Format::R32G32B32A32_SFLOAT,
        1024,
        1024,
        1,
      );
      let cmd_buff = w_device.curr_pool().get_cmd_buff();
      img.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);

      shared_images_arena.insert(img)
    };
    let dummy_image_ref = shared_images_arena[dummy_image_idx].borrow_mut();
    let dummy_image_idx = WAIdxImage {
      idx: dummy_image_idx,
    };

    // -- init buffers arena
    let shared_buffers_arena = unsafe {
      GLOBALS.shared_buffers_arena = ptralloc!(Arena<WBuffer>);
      std::ptr::write(GLOBALS.shared_buffers_arena, Arena::new());
      &mut *GLOBALS.shared_buffers_arena
    };

    let dummy_buff_idx = shared_buffers_arena.insert(WBuffer::new(
      &w_device.device,
      &mut w_device.allocator,
      // vk::Format::R32G32B32A32_SFLOAT, 16, 16, 1
      vk::BufferUsageFlags::STORAGE_BUFFER,
      1000,
      true,
    ));
    let dummy_buff_ref = shared_buffers_arena[dummy_buff_idx].borrow_mut();
    let dummy_buff_idx = WAIdxBuffer {
      idx: dummy_buff_idx,
    };

    unsafe {
      // -- init binding images array
      GLOBALS.shared_binding_images_array = ptralloc!(WBindingImageArray);
      std::ptr::write(
        GLOBALS.shared_binding_images_array,
        WBindingImageArray::new(w_device, (dummy_image_ref, &dummy_image_idx), 50),
      );

      // -- init binding buffers array
      GLOBALS.shared_binding_buffers_array = ptralloc!(WBindingBufferArray);
      std::ptr::write(
        GLOBALS.shared_binding_buffers_array,
        WBindingBufferArray::new(w_device, (dummy_buff_ref, &dummy_buff_idx), 50),
      );

      // -- init shared arenas
      GLOBALS.shared_render_targets_arena = ptralloc!(Arena<WRenderTarget>);
      std::ptr::write(GLOBALS.shared_render_targets_arena, Arena::new());

      GLOBALS.shared_ubo_arena = ptralloc!(Arena<WBindingUBO>);
      std::ptr::write(GLOBALS.shared_ubo_arena, Arena::new());

      GLOBALS.shared_compute_pipelines = ptralloc!(Arena<WComputePipeline>);
      std::ptr::write(GLOBALS.shared_compute_pipelines, Arena::new());

      GLOBALS.shared_render_pipelines = ptralloc!(Arena<WRenderPipeline>);
      std::ptr::write(GLOBALS.shared_render_pipelines, Arena::new());
    }

    Self {}
  }

  pub fn new_render_target(
    &mut self,
    w_device: &mut WDevice,
    create_info: WRenderTargetCreateInfo,
  ) -> (WAIdxRt, &mut WRenderTarget) {
    let ci = create_info.build(w_device, self);
    let idx = w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena).insert(ci);

    let rt = w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena)[idx].borrow_mut();
    let rt_idx = WAIdxRt { idx };

    (rt_idx, rt)
  }

  pub fn new_image(
    &mut self,
    w_device: &mut WDevice,
    format: vk::Format,
    resx: u32,
    resy: u32,
    resz: u32,
  ) -> (WAIdxImage, &mut WImage) {
    let (img) = { self.new_render_image(w_device, format, resx, resy, resz) }.0;

    // WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY
    // ?????????????????????????????????????
    // let img_borrow = (&mut *GLOBALS.shared_images_arena)[img.idx].borrow_mut();
    let img_borrow = w_ptr_to_mut_ref!(GLOBALS.shared_images_arena)[img.idx].borrow_mut();

    let cmd_buff = w_device.curr_pool().get_cmd_buff();
    img_borrow.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);

    // DONT USE THIS FN?
    // let descriptor_image_info  = img.1.descriptor_image_info;

    let mut arr = w_ptr_to_mut_ref!(GLOBALS.shared_binding_images_array).borrow_mut();
    let arr_idx = arr.idx_counter as usize - 1;
    arr.vk_infos[arr_idx] = img_borrow.descriptor_image_info;

    (img, img_borrow)
  }

  pub fn new_render_image(
    &mut self,
    w_device: &mut WDevice,
    format: vk::Format,
    resx: u32,
    resy: u32,
    resz: u32,
  ) -> (WAIdxImage, &mut WImage) {
    let shared_images_arena = w_ptr_to_mut_ref!(GLOBALS.shared_images_arena);
    let idx = shared_images_arena
      .insert(WImage::new(
        &w_device.device,
        &mut w_device.allocator,
        format,
        resx,
        resy,
        resz,
      ))
      .clone();

    let img = shared_images_arena[idx].borrow_mut();

    let img_idx = WAIdxImage { idx };

    // let arr = self.shared_binding_image_array.get_mut();
    // let arr = &*self.shared_binding_image_array;
    // let mut arr = arr.borrow_mut();

    let mut arr = w_ptr_to_mut_ref!(GLOBALS.shared_binding_images_array).borrow_mut();

    let arr_idx = arr.idx_counter as usize;

    arr.vk_infos[arr_idx] = img.descriptor_image_info;

    arr.idx_counter += 1;

    (img_idx, img)
  }

  pub fn new_buffer(
    &mut self,
    w_device: &mut WDevice,
    usage: vk::BufferUsageFlags,
    sz_bytes: u32,
  ) -> (WAIdxBuffer, &mut WBuffer) {
    unsafe {
      let idx = (&mut *GLOBALS.shared_buffers_arena).insert(WBuffer::new(
        &w_device.device,
        &mut w_device.allocator,
        usage,
        sz_bytes,
        true,
      ));

      let buffer = (&mut *GLOBALS.shared_buffers_arena)[idx].borrow_mut();
      let buff_idx = WAIdxBuffer { idx };
      (buff_idx, buffer)
    }
  }

  pub fn new_uniform_buffer(
    &mut self,
    w_device: &mut WDevice,
    sz_bytes: u32,
  ) -> (WAIdxUbo, &mut WBindingUBO) {
    let idx = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena).insert(WBindingUBO::new(
      &w_device.device,
      &mut w_device.allocator,
      4 * 100,
    ));

    let ubo = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[idx].borrow_mut();
    let ubo_idx = WAIdxUbo { idx };
    (ubo_idx, ubo)
  }
}

pub enum WBindingAttachmentEnum {
  UBO(WBindingUBO),
  ImageArray(WBindingImageArray),
}

pub struct WGrouper {
  pub bind_groups_arena: Arena<WBindGroup>,
}


impl WGrouper {
  pub fn new_group(
    &mut self,
    w_device: &mut WDevice,
  ) -> (WAIdxBindGroup, &mut WBindGroup) {
    let idx = self.bind_groups_arena.insert(WBindGroup::new(
      &w_device.device,
      &mut w_device.descriptor_pool,
    ));

    let bind_group = self.bind_groups_arena[idx].borrow_mut();
    let bg_idx = WAIdxBindGroup { idx };
    (bg_idx, bind_group)
  }
}
use std::sync::mpsc::channel;

pub struct WShaderMan {
  pub root_shader_dir: String,
  pub shader_was_modified: Arc<Mutex<bool>>,

  watcher: ReadDirectoryChangesWatcher,

  pub chan_sender_start_shader_comp: Sender<()>,
  pub chan_receiver_end_shader_comp: Receiver<()>,
}

impl WShaderMan {
  pub fn new() -> Self {
    let root_shader_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\shaders\\";
    let root_shader_dir = Self::sanitize_path(root_shader_dir);

    let rsd = root_shader_dir.clone();

    println!("{}", root_shader_dir);

    let shader_was_modified = Arc::new(Mutex::new(false));
    let shader_was_modified_clone = shader_was_modified.clone();

    // let shader_modification_in_progress = Arc::new(Mutex::new(false));
    // let shader_modification_in_progress_clone = shader_modification_in_progress.clone();

    let (chan_sender_start_shader_comp, chan_receiver_start_shader_comp) = channel();
    let (chan_sender_end_shader_comp, chan_receiver_end_shader_comp) = channel();

    // let compiler = shaderc::Compiler::new().unwrap();
    // let compiler_clone = compiler.get().clone();
    
    unsafe{
      let comp = Box::new(shaderc::Compiler::new().unwrap());
      let comp = Box::into_raw(comp);
      GLOBALS.compiler = comp;
    }


    // tx.send(())
    // rx.recv();

    // let shaders_arena = Arc::new(Mutex::new(Arena::new()));

    unsafe {
      GLOBALS.shader_programs_arena = ptralloc!(Arena<WProgram>);
      std::ptr::write(GLOBALS.shader_programs_arena, Arena::new());
    };

    let mut watcher = RecommendedWatcher::new(
      move |result: Result<Event, Error>| {
        let event = result.unwrap();

        *shader_was_modified_clone.lock().unwrap() = true;
        chan_receiver_start_shader_comp
          .recv()
          .expect("Error: timed out.");

        if event.kind.is_modify() {
          for __ in &event.paths {
            let mut path = __.as_os_str().to_str().unwrap();
            let mut path = String::from(path);
            path = Self::sanitize_path(path);
            path = path.replace(&root_shader_dir, "");

            let mut pipelines_which_need_reloading: SmallVec<[WShaderEnumPipelineBind; 10]> =
              SmallVec::new();
            

            macro_rules! reload_shader {
              ($shader: expr ) => {unsafe{
                if ($shader.file_name == path) {
                  $shader.try_compile(unsafe { &(&*GLOBALS.w_vulkan).w_device.device });

                  println!("-- SHADER RELOAD --");
                  println!("{}", path);

                  if ($shader.compilation_error != "") {
                    println!("{}", $shader.compilation_error);
                  } else {
                    for pipeline in &$shader.pipelines {
                      pipelines_which_need_reloading.push(*pipeline)
                    }
                  }
                }
              }};
            }

            unsafe {
              for shader_program in &mut *GLOBALS.shader_programs_arena {
                if let Some(frag_shader) = &mut shader_program.1.frag_shader{
                    reload_shader!(frag_shader);
                } else if let Some(vert_shader) = &mut shader_program.1.vert_shader{
                    reload_shader!(vert_shader);
                } else if let Some(comp_shader) = &mut shader_program.1.comp_shader{
                    reload_shader!(comp_shader);
                }
              }
            }

            macro_rules! refresh_pipeline {
              ($pipeline: expr ) => {unsafe{
                  {
                    $pipeline.get_mut().shader_program.get_mut().refresh_program_stages();
                  }
                  $pipeline.get_mut().refresh_pipeline(
                    &(*GLOBALS.w_vulkan).w_device.device,
                    &(*GLOBALS.w_vulkan).w_grouper,
                  );
              }};
            }

            for pipeline in pipelines_which_need_reloading {
              match pipeline {
                res::wshader::WShaderEnumPipelineBind::ComputePipeline(pipeline) => unsafe {
                  refresh_pipeline!(pipeline);
                  // {
                  //   pipeline.get_mut().shader_program.get_mut().refresh_program_stages();
                  // }
                  // pipeline.get_mut().refresh_pipeline(
                  //   &(*GLOBALS.w_vulkan).w_device.device,
                  //   &(*GLOBALS.w_vulkan).w_grouper,
                  // );
                },
                res::wshader::WShaderEnumPipelineBind::RenderPipeline(pipeline) => unsafe {
                  refresh_pipeline!(pipeline);
                  // let pip = pipeline.get_mut();
                  // {
                  //   let sp = pip.shader_program.get_mut();
                  //   sp.refresh_program_stages();
                  // }
                  // pipeline.get_mut().refresh_pipeline(
                  //   &(*GLOBALS.w_vulkan).w_device.device,
                  //   &(*GLOBALS.w_vulkan).w_grouper,
                  // );
                },
              }
            }

            // }

            // println!("{}", path);
          }
        }
        *shader_was_modified_clone.lock().unwrap() = false;
        chan_sender_end_shader_comp.send(());
      },
      notify::Config::default(),
    )
    .unwrap();

    watcher
      .watch(Path::new(&rsd), RecursiveMode::Recursive)
      .unwrap();

    Self {
      root_shader_dir: rsd,
      shader_was_modified,
      // shader_modification_in_progress,
      watcher,
      chan_sender_start_shader_comp,
      chan_receiver_end_shader_comp,
    }
  }

  fn sanitize_path(path: String) -> String {
    let re = regex::Regex::new(r"/")
      .unwrap()
      .replace_all(&path, "\\")
      .to_string();

    re
  }
  pub fn new_render_program(
    &mut self,
    w_device: &mut WDevice,
    mut vert_file_name: String,
    mut frag_file_name: String,
  ) -> WAIdxShaderProgram {
    vert_file_name = Self::sanitize_path(vert_file_name);
    frag_file_name = Self::sanitize_path(frag_file_name);

    let idx = unsafe {
      (*GLOBALS.shader_programs_arena).insert(WProgram::new_render_program(
        &w_device.device,
        self.root_shader_dir.clone(),
        vert_file_name,
        frag_file_name,
      ))
    };
    WAIdxShaderProgram { idx }
  }

  pub fn new_compute_program(
    &mut self,
    w_device: &mut WDevice,
    mut compute_file_name: String,
  ) -> WAIdxShaderProgram {
    compute_file_name = Self::sanitize_path(compute_file_name);

    let idx = unsafe {
      (*GLOBALS.shader_programs_arena).insert(WProgram::new_compute_program(
        &w_device.device,
        self.root_shader_dir.clone(),
        compute_file_name,
      ))
    };
    WAIdxShaderProgram { idx }
  }
}

// pub trait WBindingAttachmentIndex {
//   fn get_idx() -> generational_arena::Index;
// }
// pub struct WImageArenaIndex {
//   idx: generational_arena::Index,
// }
// impl WBindingAttachmentIndex for WImageArenaIndex {
//   fn get_idx(&self) {
//     self.idx
//   }
// }
// pub struct WUboIndex {
//   idx: generational_arena::Index,
// }
// impl WBindingAttachmentIndex for WUboIndex {
//   fn get_idx(&self) {
//     self.idx
//   }
// }
// pub struct WBindGroupIndex {
//   idx: generational_arena::Index,
// }
// impl WBindingAttachmentIndex for WBindGroupIndex {
//   fn get_idx(&self) {
//     self.idx
//   }
// }
// pub struct WBufferIndex {
//   idx: generational_arena::Index,
// }
// impl WBindingAttachmentIndex for WBufferIndex {
//   fn get_idx(&self) {
//     self.idx
//   }
// }
// pub type WBindingAttachmentIndex = generational_arena::Index;
