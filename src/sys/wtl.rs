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
    // vk::{
    ApplicationInfoBuilder,
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
    API_VERSION_1_3, // },
  },
  Entry,
};

use bytemuck::Contiguous;
use image::{GenericImageView, ImageDecoder};
use notify::{Error, Event, ReadDirectoryChangesWatcher, RecommendedWatcher, RecursiveMode, Watcher};

use gpu_alloc::{Config, GpuAllocator, Request, UsageFlags};

use generational_arena::Arena;
use smallvec::SmallVec;
use stb_image::stb_image::bindgen::{stbi_image_free, stbi_load, stbi_set_flip_vertically_on_load, stbi_uc};

use crate::{sys::warenaitems::WAIdxRenderPipeline, wvulkan::WVulkan, res::img::wrendertarget::WRPConfig, abs::wthingnull::WThingNull};
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::warenaitems::WAIdxUbo;
use crate::sys::warenaitems::WArenaItem;
use crate::{
  res::{buff::wbuffer::WBuffer, img::wimage::WImageInfo},
  sys::warenaitems::WAIdxBindGroup,
};

use crate::{
  abs::wcomputepass::WComputePass,
  abs::wthing::WThing,
  res::img::wimage::WImage,
  res::wshader::WProgram,
  res::{img::wrendertarget::WRenderTarget, wshader::WShaderEnumPipelineBind},
  wmemzeroed, wtransmute,
};
use crate::{
  c_str,
  res::{
    self,
    img::wrendertarget::WRTInfo,
    wbindings::{WBindingBufferArray, WBindingImageArray, WBindingUBO},
    wpongabletrait::WPongableTrait,
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

use std::{
  borrow::{Borrow, BorrowMut},
  cell::Cell,
  mem::MaybeUninit,
  ops::IndexMut,
  rc::Rc, io::{Cursor, BufReader}, fs::File,
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
use std::{cell::UnsafeCell, ops::BitAnd, ptr::replace};
use std::{
  ffi::{c_void, CStr, CString},
  mem,
  os::raw::c_char,
  sync::Arc,
};

use super::{
  warenaitems::{WAIdxBuffer, WAIdxImage, WAIdxRt},
  wcomputepipeline::WComputePipeline,
  wdevice::{Globals, GLOBALS},
  wrenderpipeline::WRenderPipeline, wbarr::WBarr, wformattools::WFormatTools,
};

pub enum WBindingAttachmentEnum {
  UBO(WBindingUBO),
  ImageArray(WBindingImageArray),
}


pub struct WTechLead {
  // pub bind_groups_arena: Arena<WBindGroup>,
}

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
        1,
        false,
        false,
        WImageInfo::default().usage_flags,
      );
      
      let cmd_buff = w_device.curr_pool().get_cmd_buff();
      img.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);

      shared_images_arena.insert(img)
    };
    let dummy_image_ref = shared_images_arena[dummy_image_idx].borrow_mut();
    let dummy_image_idx = WAIdxImage { idx: dummy_image_idx };
    dummy_image_ref.arena_index = dummy_image_idx;

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
    let dummy_buff_idx = WAIdxBuffer { idx: dummy_buff_idx };

    unsafe {
      // -- init binding images array
      GLOBALS.shared_binding_images_array = ptralloc!(WBindingImageArray);
      std::ptr::write(
        GLOBALS.shared_binding_images_array,
        WBindingImageArray::new(w_device, (dummy_image_ref, &dummy_image_idx), 10000),
      );

      // -- init binding buffers array
      GLOBALS.shared_binding_buffers_array = ptralloc!(WBindingBufferArray);
      std::ptr::write(
        GLOBALS.shared_binding_buffers_array,
        WBindingBufferArray::new(w_device, (dummy_buff_ref, &dummy_buff_idx), 10000),
      );

      // -- init shared arenas
      GLOBALS.bind_groups_arena = ptralloc!(Arena<WBindGroup>);
      std::ptr::write(GLOBALS.bind_groups_arena, Arena::new());

      GLOBALS.shared_render_targets_arena = ptralloc!(Arena<WRenderTarget>);
      std::ptr::write(GLOBALS.shared_render_targets_arena, Arena::new());

      GLOBALS.shared_ubo_arena = ptralloc!(Arena<WBindingUBO>);
      std::ptr::write(GLOBALS.shared_ubo_arena, Arena::new());

      GLOBALS.shared_compute_pipelines = ptralloc!(Arena<WComputePipeline>);
      std::ptr::write(GLOBALS.shared_compute_pipelines, Arena::new());

      GLOBALS.shared_render_pipelines = ptralloc!(Arena<WRenderPipeline>);
      std::ptr::write(GLOBALS.shared_render_pipelines, Arena::new());
    }

    Self { 
      // bind_groups_arena: Arena::new(),
   }
  }

  pub fn new_render_target(
    &mut self,
    // w_device: &mut WDevice,
    w_v: &mut WVulkan,
    create_info: WRTInfo,
  ) -> (WAIdxRt, &mut WRenderTarget) {
    let ci = create_info.create(w_v, self);
    let idx = w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena).insert(ci);

    let rt = w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena)[idx].borrow_mut();
    let rt_idx = WAIdxRt { idx };

    (rt_idx, rt)
  }

  fn copy_gpu_buff_to_gpu_image(
    w_device: &mut WDevice,
    img: WAIdxImage,
    input_channels: usize,
    sz_bytes: usize,
    height: usize,
    width: usize,
  ) {
  }

  pub fn copy_swapchain_to_cpu_image(
    w_device: &mut WDevice,
    // w_swapchain: &mut WSwapchain,
    // img: WAIdxImage,
    img: &WImage,
    pixels: &mut Vec<u8>,
    // pixels: *const u8,
    // input_channels: usize,
  ) {
    let img_borrow = img;
    unsafe {
      let sz_bytes = img_borrow.resx * img_borrow.resy * 4;
      let mut staging_buff = WBuffer::new(
        &w_device.device,
        &mut w_device.allocator,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC,
        sz_bytes,
        false,
      );

      staging_buff.map(&w_device.device);

      // uuh
      w_device.device.queue_wait_idle(w_device.queue);

      let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder().build();

      let cmd_buff = w_device.curr_pool().get_cmd_buff();
      w_device.device.begin_command_buffer(cmd_buff, &cmd_buf_begin_info);

      // img_borrow.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);
      w_device.device.queue_wait_idle(w_device.queue);


      WBarr::image()
        .old_layout(img_borrow.descriptor_image_info.image_layout)
        // .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .set_image(img_borrow.handle)
        .src_access(vk::AccessFlags2::MEMORY_READ)
        .dst_access(vk::AccessFlags2::TRANSFER_READ)
        .src_stage(vk::PipelineStageFlags2::TRANSFER)
        .dst_stage(vk::PipelineStageFlags2::TRANSFER)
        .run_on_cmd_buff(&w_device, cmd_buff);

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
        .image_extent(vk::Extent3D {
          width: img_borrow.resx,
          height: img_borrow.resy,
          depth: 1,
        })
        .build();

      // let cmd_buff = w_device.curr_pool().get_cmd_buff();

      w_device.device.cmd_copy_image_to_buffer(
        cmd_buff,
        img_borrow.handle,
        // vk::ImageLayout::GENERAL,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        staging_buff.get_handle(),
        &[region],
      );

      WBarr::image()
        .set_image(img.handle)
        .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .src_access(vk::AccessFlags2::TRANSFER_READ)
        .dst_access(vk::AccessFlags2::MEMORY_READ)
        .src_stage(vk::PipelineStageFlags2::TRANSFER)
        .dst_stage(vk::PipelineStageFlags2::TRANSFER)
        .run_on_cmd_buff(w_device, cmd_buff);


      w_device.single_command_end_submit(cmd_buff);

      w_device.device.queue_wait_idle(w_device.queue);

      staging_buff.delete(&w_device.device, &mut w_device.allocator);

      pixels.set_len((img_borrow.resx * img_borrow.resy * 4) as usize);
      let ptr = staging_buff.get_mapped_ptr();
      for i in 0..(img_borrow.resx * img_borrow.resy) as isize {
        let idx = i * 4;
        let val_b = *ptr.offset(idx);
        let val_g = *ptr.offset(idx+1);
        let val_r = *ptr.offset(idx+2);
        let val_a = *ptr.offset(idx+3);
        // let val = (val as f64)/(u8::MAX_VALUE as f64);
        // let val = val.powf(1./0.45454545);
        // let val = (val*(u8::MAX_VALUE as f64)) as u8;

        pixels[idx as usize] = val_r;
        pixels[(idx + 1) as usize] = val_g;
        pixels[(idx + 2) as usize] = val_b;
        pixels[(idx + 3) as usize] = u8::MAX_VALUE;
        // if i%4 == 3{
        // } else{
        // }
      }

      w_device.device.queue_wait_idle(w_device.queue);
    }
  }

  fn copy_cpu_to_gpu_image(
    w_device: &mut WDevice,
    img: WAIdxImage,
    pixels: *const u8,
    format: vk::Format,
    sz_bytes: usize,
    height: usize,
    width: usize,
  ) {
    // let input_channels = format. usize;
    let input_channels = format.chan_cnt() as usize;
    let bytes_per_chan = format.bytes_per_chan() as isize;

    let img_borrow = img.get_mut();
    unsafe {
      let mut staging_buff = WBuffer::new(
        &w_device.device,
        &mut w_device.allocator,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC,
        sz_bytes as u32,
        false,
      );

      staging_buff.map(&w_device.device);

      let ptr = staging_buff.get_mapped_ptr();


      let mut i_cnt = 0;
      if input_channels == 3 {
        for i in 0..(height * width * 3) as isize {
          let i_arr = i * bytes_per_chan;
          let i = (i + i / 3) * bytes_per_chan;
          
          i_cnt = i;


          for k in 0..bytes_per_chan{
            let val = *pixels.offset(i_arr + k);
            *ptr.offset(i + k ) = val;
          }
        }
      } else {
        for i in 0..(height * width * 4) as isize {
          let i = i * bytes_per_chan;
          i_cnt = i;

          for k in 0..bytes_per_chan{
            *ptr.offset(i + k) = *pixels.offset(i + k);
          }
          // if i % 4 == 3{
          //   *ptr.offset(i ) = u8::MAX - 1;
          // } else {
          //   *ptr.offset(i ) = *pixels.offset(i);
          // }
        }
      }
      println!("{}", sz_bytes);
      println!("{}", i_cnt);

      // uuh
      w_device.device.queue_wait_idle(w_device.queue);

      let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder().build();

      let cmd_buff = w_device.curr_pool().get_cmd_buff();
      img_borrow.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);
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
        .image_extent(vk::Extent3D {
          width: img_borrow.resx,
          height: img_borrow.resy,
          depth: 1,
        })
        .build();

      // let cmd_buff = w_device.curr_pool().get_cmd_buff();
      // w_device.device.begin_command_buffer(cmd_buff, &cmd_buf_begin_info);
      let cmd_buff = w_device.single_command_begin();
      w_device.device.cmd_copy_buffer_to_image(
        cmd_buff,
        staging_buff.get_handle(),
        img_borrow.handle,
        vk::ImageLayout::GENERAL,
        &[region],
      );
      w_device.single_command_end_submit(cmd_buff);
      w_device.device.queue_wait_idle(w_device.queue);

      staging_buff.delete(&w_device.device, &mut w_device.allocator);
    }
  }
  #[inline(never)]
  fn load_file_image_internal(
    &mut self,
    mut file_name: String,
    // w_device: &mut WDevice,
    w_v: &mut WVulkan,
    // w_tl: &mut WTechLead,
    mut create_info: WImageInfo,
  ) -> WAIdxImage{
      let mut folder_name = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\images\\";

      // this will crash on linux LOL
      if (file_name.find(":\\").is_none()) {
        file_name = folder_name + &file_name;
      }

      let ext = file_name.split(".").last().unwrap();
      
      if ext == "exr"{
        let prog_render = w_v
          .w_shader_man
          .new_render_program(&mut w_v.w_device, "cubemap.vert", "cubemap.frag");

        create_info.format = vk::Format::R32G32B32A32_SFLOAT;

        create_info.usage_flags = vk::ImageUsageFlags::TRANSFER_DST
          | vk::ImageUsageFlags::TRANSFER_SRC
          | vk::ImageUsageFlags::SAMPLED
          | vk::ImageUsageFlags::STORAGE
          | vk::ImageUsageFlags::COLOR_ATTACHMENT;

        let mut cubemap_info = create_info.clone();
          cubemap_info.resx = 1024;
          cubemap_info.resy = 1024;
          cubemap_info.is_cubemap = true;
          cubemap_info.file_path = None;
        let cubemap_idx = self.new_image_internal(&mut w_v.w_device,  cubemap_info, true).0;

        let img = image::open(file_name.clone()).unwrap();
        let width = img.bounds().2;
        let height = img.bounds().3;

        let mut pixels: *const u8;
        let mut _pixels: Vec<u8> = Vec::new();
        let hdr_img_idx = {
          let mut r = BufReader::new(File::open(file_name.clone()).unwrap());
          let decoder = image::codecs::openexr::OpenExrDecoder::with_alpha_preference(r, Some(true)).unwrap();
          
          _pixels.reserve(decoder.total_bytes() as usize);
          unsafe{
            _pixels.set_len(decoder.total_bytes() as usize);
          }
          decoder.read_image(&mut _pixels).unwrap();

          // println!("{}", _pixels.len());
          pixels = _pixels.as_ptr() as *const u8;
          create_info.resx = width.try_into().unwrap();
          create_info.resy = height.try_into().unwrap();

          let mut hdr_create_info = create_info.clone();
          hdr_create_info.file_path = None;

          let img = self.new_image(w_v, hdr_create_info).0;
          img
        };
        let chan_cnt = create_info.format.chan_cnt();
        let bytes_per_chan = create_info.format.bytes_per_chan();
        let sz_bytes = height * width * chan_cnt * bytes_per_chan;
        WTechLead::copy_cpu_to_gpu_image(&mut w_v.w_device, hdr_img_idx, pixels, create_info.format, sz_bytes as usize, height as usize, width as usize);
      

        let mut thing = WThingNull::new(w_v, self, prog_render);
        
        
        // -- Draw cubemap -- //
        let mut rt = self.new_render_target(w_v, WRTInfo::from_images(&[cubemap_idx])).0;

        unsafe {
          let cmd_buf = rt.get_mut().begin_pass_ext(&mut w_v.w_device, WRPConfig{ layer_cnt: 6 });

          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);
          println!("{:?}", hdr_img_idx.idx.index);

          
          thing.push_constants.reset();
          thing.push_constants.add(hdr_img_idx);
          
          // thing.push_constants.add(hdr_img_idx);
          
          thing.draw_cnt(w_v, self, Some(rt), &cmd_buf,4,6);

          rt.get_mut().end_pass(&mut w_v.w_device);
          
          w_v.w_device.single_command_submit(cmd_buf);
          w_v.w_device.device.queue_wait_idle(w_v.w_device.queue);
        }

        cubemap_idx
      } else if ext == "hdr"{
        // let img = image::open(file_name.clone()).unwrap();
        // let width = img.bounds().2;
        // let height = img.bounds().3;
        // println!("{}", width);
        // println!("{}", height);
        // println!("{}", img.bounds().0);
        // println!("{}", img.bounds().1);

        // let mut r = BufReader::new(File::open(file_name.clone()).unwrap());
        // let decoder = image::codecs::hdr::HdrDecoder::new(r).unwrap();
        // let pixels = decoder.read_image_hdr().unwrap();

        // //   image::io::Reader::new(
        // //   Cursor::new(inner)
        // // )

        // println!("{}", pixels.len());

        // // let pixels = image::hdr::read_raw_file(file_name).unwrap().as_ptr();
        // let pixels = &pixels[0][0] as *const f32;
        // let pixels = pixels as *const u8;

        // create_info.resx = width.try_into().unwrap();
        // create_info.resy = height.try_into().unwrap();
        // create_info.format = vk::Format::R32G32B32A32_SFLOAT;

        // create_info.usage_flags = vk::ImageUsageFlags::TRANSFER_DST
        //   | vk::ImageUsageFlags::TRANSFER_SRC
        //   | vk::ImageUsageFlags::SAMPLED
        //   | vk::ImageUsageFlags::STORAGE
        //   | vk::ImageUsageFlags::COLOR_ATTACHMENT;

        let img_idx = {
          let img = self.new_image_internal(&mut w_v.w_device, create_info.clone(), true);
          img.1.arena_index = img.0;
          img.0
        };
        // let chan_cnt = create_info.format.chan_cnt();
        // let bytes_per_chan = create_info.format.bytes_per_chan();

        // let sz_bytes = height * width * chan_cnt * bytes_per_chan;


        // WTechLead::copy_cpu_to_gpu_image(&mut w_v.w_device, img_idx, pixels, vk::Format::R32G32B32_SFLOAT, sz_bytes as usize, height as usize, width as usize);
        img_idx
      } else {
        unsafe {
          stbi_set_flip_vertically_on_load(1i32);

          // stb_image::image::Image::new(width, height, depth, data);
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
          // create_info.usage_flags = vk::ImageUsageFlags::TRANSFER_DST
          //   | vk::ImageUsageFlags::SAMPLED
          //   | vk::ImageUsageFlags::STORAGE;
          create_info.usage_flags = vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::STORAGE
            | vk::ImageUsageFlags::COLOR_ATTACHMENT;
          
          let mut ci = create_info.clone();
          ci.file_path = None;

          let img_idx = {
            let img = self.new_image_internal(&mut w_v.w_device, create_info.clone(), true);
            img.1.arena_index = img.0;
            img.0
          };

          // let img_idx = {
          //   let img = self.new_image_internal(&mut w_v.w_device, create_info.clone());
          //   img.1.arena_index = img.0;
          //   img.0
          // };


          let sz_bytes = height * width * 4;

          WTechLead::copy_cpu_to_gpu_image(&mut w_v.w_device, img_idx, pixels.as_ptr(), vk::Format::R8G8B8_UNORM, sz_bytes, height, width);

          img_idx
        }
      }

  }
  
  fn load_image_pixels_internal(
    &mut self,
    raw_pixels: *mut u8,
    // w_device: &mut WDevice,
    w_v: &mut WVulkan,
    mut create_info: WImageInfo,
  ) -> WAIdxImage{
    create_info.usage_flags = vk::ImageUsageFlags::TRANSFER_DST
      | vk::ImageUsageFlags::TRANSFER_SRC
      | vk::ImageUsageFlags::SAMPLED
      | vk::ImageUsageFlags::STORAGE
      | vk::ImageUsageFlags::COLOR_ATTACHMENT;
    let img_idx = {
      // let img = self.new_render_image(w_device, create_info.clone());
      let mut create_info_edit = create_info.clone();
      create_info_edit.format = vk::Format::R8G8B8A8_UNORM;

      let img = self.new_image_internal(&mut w_v.w_device, create_info_edit.clone(), true);
      img.1.arena_index = img.0;
      img.0
    };

    let sz_bytes = create_info.resx * create_info.resy * 4;
    WTechLead::copy_cpu_to_gpu_image(
      &mut w_v.w_device,
      img_idx,
      raw_pixels,
      // if create_info.format == vk::Format::R8G8B8_UNORM { 3 } else { 4 },
      create_info.format,

      sz_bytes as usize,
      create_info.resy as usize,
      create_info.resx as usize,
    );

    img_idx
  }
  
  fn add_image_to_imgui(&mut self, w_v: &mut WVulkan,img_borrow: &mut WImage){
    unsafe {
      let layout = imgui_rs_vulkan_renderer::vulkan::create_vulkan_descriptor_set_layout(&w_v.w_device.device).unwrap();

      let linear_sampler_info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .build();
      let linear_sampler = w_v.w_device.device.create_sampler(&linear_sampler_info, None).unwrap();

      if img_borrow.is_cubemap {
        let imgui_id = wmemzeroed!();
        img_borrow.imgui_id = imgui_id;
      } else {
        let descriptor_set = imgui_rs_vulkan_renderer::vulkan::create_vulkan_descriptor_set(
          &w_v.w_device.device,
          layout,
          w_v.w_device.descriptor_pool,
          img_borrow.view,
          linear_sampler,
        ).unwrap();

        let textures = w_v.w_device.imgui_renderer.textures();

        let imgui_id = textures.insert(descriptor_set);
        println!("-------- IMGUI ID: {}", imgui_id.id());
        img_borrow.imgui_id = imgui_id;
      }
    }
  }

  pub fn new_image(
    &mut self,
    // w_device: &mut WDevice,
    w_v: &mut WVulkan,
    mut create_info: WImageInfo,
  ) -> (WAIdxImage, &mut WImage) {
    let img = if let Some(mut file_name) = create_info.clone().file_path {
      create_info.usage_flags |= vk::ImageUsageFlags::STORAGE;
      let i = self.load_file_image_internal(
        file_name,
        w_v,
        // self,
        create_info.clone(),
      );
      println!("{:?}","AMOGUS");
      println!("{:?}","AMOGUS");
      println!("{:?}","AMOGUS");
      println!("{:?}","AMOGUS");
      println!("{:?}","AMOGUS");
      println!("{:?}","AMOGUS");
      println!("{:?}","AMOGUS");
      println!("{:?}","AMOGUS");
      println!("{:?}","AMOGUS");
      println!("{:?}",i);
      i
    } else if let Some(raw_pixels) = create_info.clone().raw_pixels {
      self.load_image_pixels_internal(
        raw_pixels,
        w_v,
        create_info.clone(),
        // self,
      )
    } else {
      self.new_image_internal(&mut w_v.w_device, create_info.clone(), true).0
    };

    
    let img_borrow =  unsafe{ (&mut *GLOBALS.shared_images_arena)[img.idx].borrow_mut()};

    
    // not needed
    unsafe{
      w_v.w_device.device.queue_wait_idle(w_v.w_device.queue);
    }

    if create_info.mip_levels > 1 {
      img_borrow.generate_mipmaps(&mut w_v.w_device);
    }

    let arr_idx = img.idx.index as u32 ;
    let is_storage_img = create_info.usage_flags.bitand(vk::ImageUsageFlags::STORAGE).as_raw() != 0;
    w_v.shared_bind_group.get_mut().update_descriptor_image(&w_v.w_device.device, is_storage_img, arr_idx);
    

    
    // add to imgui
    self.add_image_to_imgui( w_v,img_borrow);

    (img, img_borrow)
  }

  fn new_image_internal( 
    &mut self,
    w_device: &mut WDevice,
    create_info: WImageInfo,
    set_layout_to_general: bool
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
        create_info.mip_levels,
        create_info.is_depth,
        create_info.is_cubemap,
        create_info.usage_flags, // WImageCreateInfo::default().usage_flags
      ))
      .clone();

    let img = shared_images_arena[idx].borrow_mut();

    let img_idx = WAIdxImage { idx };

    img.arena_index = img_idx;

    let mut arr = w_ptr_to_mut_ref!(GLOBALS.shared_binding_images_array).borrow_mut();
    
    if set_layout_to_general {
      let cmd_buff = w_device.curr_pool().get_cmd_buff();
      img.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);
    }
    

    debug_assert!(arr.idx_counter == idx.index as u32);

    if img.usage_flags.bitand(vk::ImageUsageFlags::STORAGE).as_raw() != 0 {
      arr.vk_infos_storage[arr.idx_counter as usize] = img.descriptor_image_info;
      arr.vk_infos_sampled[arr.idx_counter as usize] = img.descriptor_image_info;
    } else {
      arr.vk_infos_sampled[arr.idx_counter as usize] = img.descriptor_image_info;
    }

    arr.idx_counter += 1;

    (img_idx, img)
  }

  pub fn new_buffer(
    &mut self,
    // w_device: &mut WDevice,
    w_v: &mut WVulkan,
    usage: vk::BufferUsageFlags,
    sz_bytes: u32,
    pongable: bool,
  ) -> (WAIdxBuffer, &mut WBuffer) {
    unsafe {
      let w_device = &mut w_v.w_device;
      let idx =
        (&mut *GLOBALS.shared_buffers_arena).insert(WBuffer::new(&w_device.device, &mut w_device.allocator, usage, sz_bytes, pongable));

      let buffer = (&mut *GLOBALS.shared_buffers_arena)[idx].borrow_mut();
      let buff_idx = WAIdxBuffer { idx };

      buffer.arena_index = buff_idx;

      let mut arr = w_ptr_to_mut_ref!(GLOBALS.shared_binding_buffers_array).borrow_mut();
      let arr_idx = arr.idx_counter as usize;

      // if img_borrow.usage_flags.bitand(vk::ImageUsageFlags::STORAGE).as_raw() != 0 {
      arr.vk_infos[arr_idx] = buffer.descriptor_buffer_info[0];

      arr.idx_counter += 1;
      
      w_v.shared_bind_group.get_mut().update_descriptor_buff(&w_v.w_device.device, arr_idx as u32);

      (buff_idx, buffer)
    }
  }

  pub fn new_uniform_buffer(
    &mut self,
    w_device: &mut WDevice,
    sz_bytes: u32,
  ) -> (WAIdxUbo, &mut WBindingUBO) {
    let idx = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena).insert(WBindingUBO::new(&w_device.device, &mut w_device.allocator, 4 * 500));

    let ubo = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[idx].borrow_mut();
    let ubo_idx = WAIdxUbo { idx };
    (ubo_idx, ubo)
  }
  pub fn new_group(
    &self,
    w_device: &mut WDevice,
  ) -> (WAIdxBindGroup, &mut WBindGroup) {
    unsafe{
      let idx = (*GLOBALS.bind_groups_arena)
        .insert(WBindGroup::new(&w_device.device, &mut w_device.descriptor_pool));

      let bind_group = (*GLOBALS.bind_groups_arena)[idx].borrow_mut();
      let bg_idx = WAIdxBindGroup { idx };
      (bg_idx, bind_group)
    }
  }
}


use std::sync::mpsc::channel;
