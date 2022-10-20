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
use stb_image::stb_image::bindgen::{stbi_uc, stbi_load, stbi_image_free, stbi_set_flip_vertically_on_load};

use crate::{sys::warenaitems::WAIdxBindGroup, res::wimage::WImageCreateInfo};
use crate::sys::warenaitems::WAIdxRenderPipeline;
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::warenaitems::WAIdxUbo;
use crate::sys::warenaitems::WArenaItem;

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

use std::{ptr::replace, cell::{UnsafeCell}, ops::BitAnd};
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
  wrenderpipeline::WRenderPipeline, warenaitems::{WAIdxRt, WAIdxImage, WAIdxBuffer},
};

pub struct WTechLead { }

impl WTechLead {

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
        false,
        WImageCreateInfo::default().usage_flags
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
    let ci = create_info.create(w_device, self);
    let idx = w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena).insert(ci);

    let rt = w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena)[idx].borrow_mut();
    let rt_idx = WAIdxRt { idx };

    (rt_idx, rt)
  }

  // same as new_render_image, but with GENERAL layout. 🧠
  pub fn new_image(
    &mut self,
    w_device: &mut WDevice,
    mut create_info: WImageCreateInfo,
  ) -> (WAIdxImage, &mut WImage) {
    let img = if let Some(mut file_name) = create_info.clone().file_name{

      let mut folder_name = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\images\\";
      file_name = folder_name + &file_name;
      
      unsafe{
        stbi_set_flip_vertically_on_load(1i32);

        let (width, height, pixels, channels) = match stb_image::image::load(file_name){
            stb_image::image::LoadResult::ImageU8(__) => {
              (__.width, __.height, __.data, 3)
            },
            _ => { todo!(); (0,0,vec![],0)}
            // stb_image::image::LoadResult::Error(_) => {debug_assert!(false)},
            // stb_image::image::LoadResult::ImageF32(_) => {debug_assert!(false)},
        };
        
        create_info.resx = width.try_into().unwrap();
        create_info.resy = height.try_into().unwrap();
        // create_info.format = vk::Format::R8G8B8A8_UNORM;
        create_info.format = vk::Format::R8G8B8A8_UNORM;
        create_info.usage_flags = vk::ImageUsageFlags::TRANSFER_DST
          | vk::ImageUsageFlags::SAMPLED
          | vk::ImageUsageFlags::STORAGE;
        
        
        let sz_bytes = height * width * 4;
  
        let mut staging_buff = WBuffer::new(&w_device.device, &mut w_device.allocator, 
          vk::BufferUsageFlags::STORAGE_BUFFER |
          vk::BufferUsageFlags::TRANSFER_DST | 
          vk::BufferUsageFlags::TRANSFER_SRC
          ,
          sz_bytes as u32,
          false, 
        );
        
        staging_buff.map(&w_device.device);
        

        let ptr = staging_buff.get_mapped_ptr();
        for i in 0..(height * width * 3) as isize{
          *ptr.offset(i + i/3) = *pixels.as_ptr().offset(i);
        }

        // uuh
        w_device.device.queue_wait_idle(w_device.queue);


        let (img_idx, img)= { self.new_render_image(w_device, create_info.clone()) };
        

        let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder().build();

        let cmd_buff = w_device.curr_pool().get_cmd_buff();
        img.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);
        w_device.device.queue_wait_idle(w_device.queue);

        let subresource = vk::ImageSubresourceLayers::builder()
          .aspect_mask(vk::ImageAspectFlags::COLOR)
          .mip_level(0)
          .base_array_layer(0)
          .layer_count(1)
          .build();

        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(subresource)
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D { width: img.resx, height: img.resy, depth: 1 })
            .build();

        let cmd_buff = w_device.curr_pool().get_cmd_buff();
        w_device.device.begin_command_buffer(cmd_buff,&cmd_buf_begin_info);
        w_device.device.cmd_copy_buffer_to_image(cmd_buff, staging_buff.get_handle(), img.handle,vk::ImageLayout::GENERAL, &[region]);
        w_device.device.end_command_buffer(cmd_buff);
        w_device.device.queue_submit(w_device.queue, &[vk::SubmitInfo::builder().command_buffers(&[cmd_buff]).build()], vk::Fence::null());
        w_device.device.queue_wait_idle(w_device.queue);



        staging_buff.delete(&w_device.device, &mut w_device.allocator);

        // stbi_image_free(pixels as *mut c_void);
        img_idx
      } 
    } else {
      self.new_render_image(w_device, create_info.clone()).0
    };


    // WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY WHY
    // oooooh i know why now. because rust.
    // ?????????????????????????????????????
    let img_borrow = w_ptr_to_mut_ref!(GLOBALS.shared_images_arena)[img.idx].borrow_mut();

    let cmd_buff = w_device.curr_pool().get_cmd_buff();
    img_borrow.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);


    let mut arr = w_ptr_to_mut_ref!(GLOBALS.shared_binding_images_array).borrow_mut();
    let arr_idx = arr.idx_counter as usize - 1;

    // hello future person debugging why smth is broken. 
    // it is because of this.
    // if img_borrow.usage_flags.intersects(vk::ImageUsageFlags::STORAGE){
    if img_borrow.usage_flags.bitand(vk::ImageUsageFlags::STORAGE).as_raw() != 0 {
      arr.vk_infos[arr_idx] = img_borrow.descriptor_image_info;
    } 

    (img, img_borrow)
  }

  pub fn new_render_image(
    &mut self,
    w_device: &mut WDevice,
    create_info: WImageCreateInfo,
    // format: vk::Format,
    // resx: u32,
    // resy: u32,
    // resz: u32,
  ) -> (WAIdxImage, &mut WImage) {
    let shared_images_arena = w_ptr_to_mut_ref!(GLOBALS.shared_images_arena);
    let idx = shared_images_arena
      .insert(WImage::new(
        &w_device.device,
        &mut w_device.allocator,
        create_info.format,
        create_info.resx,
        create_info.resy,
        create_info.resz,
        create_info.is_depth,
        create_info.usage_flags
        // WImageCreateInfo::default().usage_flags
      ))
      .clone();

    let img = shared_images_arena[idx].borrow_mut();

    let img_idx = WAIdxImage { idx };

    let mut arr = w_ptr_to_mut_ref!(GLOBALS.shared_binding_images_array).borrow_mut();

    let arr_idx = arr.idx_counter as usize;


    if img.usage_flags.bitand(vk::ImageUsageFlags::STORAGE).as_raw() != 0 {
      arr.vk_infos[arr_idx] = img.descriptor_image_info;  
      // arr.vk_infos[arr_idx] = img_borrow.descriptor_image_info;
    } 

    arr.idx_counter += 1;

    (img_idx, img)
  }

  pub fn new_buffer(
    &mut self,
    w_device: &mut WDevice,
    usage: vk::BufferUsageFlags,
    sz_bytes: u32,
    pongable: bool,
  ) -> (WAIdxBuffer, &mut WBuffer) {
    unsafe {
      let idx = (&mut *GLOBALS.shared_buffers_arena).insert(WBuffer::new(
        &w_device.device,
        &mut w_device.allocator,
        usage,
        sz_bytes,
        pongable,
      ));

      let buffer = (&mut *GLOBALS.shared_buffers_arena)[idx].borrow_mut();
      let buff_idx = WAIdxBuffer { idx };

      let mut arr = w_ptr_to_mut_ref!(GLOBALS.shared_binding_buffers_array).borrow_mut();
      let arr_idx = arr.idx_counter as usize;

      // if img_borrow.usage_flags.bitand(vk::ImageUsageFlags::STORAGE).as_raw() != 0 {
      arr.vk_infos[arr_idx] = buffer.descriptor_buffer_info[0];

      arr.idx_counter += 1;
      // } 
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

    let (chan_sender_start_shader_comp, chan_receiver_start_shader_comp) = channel();
    let (chan_sender_end_shader_comp, chan_receiver_end_shader_comp) = channel();

    unsafe{
      let comp = Box::new(shaderc::Compiler::new().unwrap());
      let comp = Box::into_raw(comp);
      GLOBALS.compiler = comp;
    }

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
                } 
                if let Some(vert_shader) = &mut shader_program.1.vert_shader{
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
                },
                res::wshader::WShaderEnumPipelineBind::RenderPipeline(pipeline) => unsafe {
                  refresh_pipeline!(pipeline);
                },
              }
            }
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
  pub fn new_render_program<S: Into<String>>(
    &mut self,
    w_device: &mut WDevice,
    mut vert_file_name: S,
    mut frag_file_name: S,
  ) -> WAIdxShaderProgram {
    let vert_file_name = Self::sanitize_path(vert_file_name.into());
    let frag_file_name = Self::sanitize_path(frag_file_name.into());

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

  pub fn new_compute_program<S: Into<String>>(
    &mut self,
    w_device: &mut WDevice,
    mut compute_file_name: S,
  ) -> WAIdxShaderProgram {
    let compute_file_name = Self::sanitize_path(compute_file_name.into());

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
