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

use gpu_alloc::{Config, GpuAllocator, Request, UsageFlags};

use generational_arena::Arena;

use crate::{
  c_str,
  sys::wbindgroup::WBindGroup,
  res::{wbindings::{WBindingImageArray, WBindingUBO, WBindingBufferArray}, wpongabletrait::WPongableTrait, wrendertarget::WRenderTargetCreateInfo},
  sys::wdevice::WDevice,
  sys::wswapchain::{self, WSwapchain},
};
use crate::{
  res::wbuffer::WBuffer, abs::wcomputepass::WComputePass, res::wimage::WImage, wmemzeroed,
  res::wrendertarget::WRenderTarget, res::wshader::WProgram, abs::wthing::WThing, wtransmute,
};
use gpu_alloc_ash::AshMemoryDevice;
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

use std::{cell::RefCell, ops::Deref};
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

use super::wdevice::{GLOBALS, Globals};


#[derive(Debug, Copy, Clone)]
pub enum WEnumBind {
  WAIdxImage(WAIdxImage),
  // WBindingImageArray(Rc<RefCell<WBindingImageArray>>),
  WAIdxUbo(WAIdxUbo),
  WAIdxBuffer(WAIdxBuffer),
}

pub trait WBinding{
  fn get_type(&self)->WBindType;
}

pub trait WArenaItem{
  fn get_arena_idx(&self)->generational_arena::Index;
}

// ----------- 
#[derive(Debug, Copy, Clone)]
pub struct WAIdxBindGroup {
  pub idx: generational_arena::Index,
}
impl WArenaItem for WAIdxBindGroup{
  fn get_arena_idx(&self)->generational_arena::Index {
    self.idx
  }

}
// ----------- 

#[derive(Debug, Copy, Clone)]
pub struct WAIdxImage {
  pub idx: generational_arena::Index,
}
impl WArenaItem for WAIdxImage{
  fn get_arena_idx(&self)->generational_arena::Index {
    self.idx
  }
}
impl WBinding for WAIdxImage{
    fn get_type(&self)->WBindType {
      WBindType::WBindTypeImage
    }
}

// ----------- 

#[derive(Debug, Copy, Clone)]
pub struct WAIdxUbo {
  pub idx: generational_arena::Index,
}
impl WArenaItem for WAIdxUbo{
  fn get_arena_idx(&self)->generational_arena::Index {
    self.idx
  }
}
impl WBinding for WAIdxUbo{
    fn get_type(&self)->WBindType {
      WBindType::WBindTypeUbo
    }
}

// // ----------- 

#[derive(Debug, Copy, Clone)]
pub struct WAIdxBuffer {
  pub idx: generational_arena::Index,
}
impl WArenaItem for WAIdxBuffer{
  fn get_arena_idx(&self)->generational_arena::Index {
    self.idx
  }
}
impl WBinding for WAIdxBuffer{
    fn get_type(&self)->WBindType {
      WBindType::WBindTypeBuffer
    }
}

// // ----------- 

#[derive(Debug, Copy, Clone)]
pub struct WAIdxRt {
  pub idx: generational_arena::Index,
}
impl WArenaItem for WAIdxRt{
  fn get_arena_idx(&self)->generational_arena::Index {
    self.idx
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
  pub fn get_rt (&mut self, rt: &WAIdxRt)-> &mut WRenderTarget{
    w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena)[rt.idx].borrow_mut()
  }
  
  pub fn pong_all(
    &mut self
  ){
    unsafe {
      ((&mut* GLOBALS.shared_buffers_arena)).into_iter().map(|__|{
          let buff = __.1;
          buff.pong();
      })
    };
    w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena).into_iter().map(|__|{
      let rt = __.1;
      rt.pong();
    });
    
  }



  pub fn new(
    w_device: &mut WDevice,
  )-> Self { 
    // -- init images arena
    let shared_images_arena =  unsafe{ 
      GLOBALS.shared_images_arena = ptralloc!( Arena<WImage>);
      std::ptr::write(GLOBALS.shared_images_arena, Arena::new());
      &mut *GLOBALS.shared_images_arena
    };

    let dummy_image_idx = {
      let mut img = WImage::new(
        &w_device.device,&mut w_device.allocator, vk::Format::R32G32B32A32_SFLOAT, 1024, 1024, 1
      );
      let cmd_buff = w_device.curr_pool().get_cmd_buff();
      img.change_layout(w_device, vk::ImageLayout::GENERAL, cmd_buff);

      shared_images_arena.insert(img)
    };
    let dummy_image_ref = shared_images_arena[dummy_image_idx].borrow_mut();
    let dummy_image_idx = WAIdxImage { idx: dummy_image_idx };




    // -- init buffers arena
    let shared_buffers_arena =  unsafe{ 
      GLOBALS.shared_buffers_arena = ptralloc!( Arena<WBuffer>);
      std::ptr::write(GLOBALS.shared_buffers_arena, Arena::new());
      &mut *GLOBALS.shared_buffers_arena
    };

    let dummy_buff_idx =  shared_buffers_arena.insert(
      WBuffer::new(
        &w_device.device,
        &mut w_device.allocator, 
        // vk::Format::R32G32B32A32_SFLOAT, 16, 16, 1
      vk::BufferUsageFlags::STORAGE_BUFFER,
      1000,
      true
      )
    );
    let dummy_buff_ref = shared_buffers_arena[dummy_buff_idx].borrow_mut();
    let dummy_buff_idx = WAIdxBuffer { idx: dummy_buff_idx };


    unsafe{
      // -- init binding images array
      GLOBALS.shared_binding_images_array = ptralloc!( WBindingImageArray);
      std::ptr::write(
        GLOBALS.shared_binding_images_array, 
          WBindingImageArray::new( 
            w_device,
            (dummy_image_ref, &dummy_image_idx),
            50
          )
      );

      // -- init binding buffers array
      GLOBALS.shared_binding_buffers_array = ptralloc!( WBindingBufferArray);
      std::ptr::write(
        GLOBALS.shared_binding_buffers_array, 
          WBindingBufferArray::new( 
            w_device,
            (dummy_buff_ref, &dummy_buff_idx),
            50
          )
      );


      // -- init shared arenas
      GLOBALS.shared_render_targets_arena = ptralloc!( Arena<WRenderTarget>);
      std::ptr::write(GLOBALS.shared_render_targets_arena, Arena::new());

      GLOBALS.shared_ubo_arena = ptralloc!( Arena<WBindingUBO>);
      std::ptr::write(GLOBALS.shared_ubo_arena, Arena::new());

    }

    Self { }

  }
  

  pub fn new_render_target(
    &mut self,
    w_device: &mut WDevice,
    create_info: WRenderTargetCreateInfo
  ) -> (WAIdxRt, &mut WRenderTarget) {
    let ci = create_info.build(w_device,self);
    let idx = w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena).insert( ci);

    let rt = w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena)[idx].borrow_mut();
    let rt_idx = WAIdxRt { idx};

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
    let (img)= {
      self.new_render_image(
        w_device,
        format,
        resx,
        resy,
        resz,
      )
    }.0;

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
    let idx = shared_images_arena.insert(
      WImage::new(
        &w_device.device,
        &mut w_device.allocator,
        format,
        resx,
        resy,
        resz,
      )
    ).clone();

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
      unsafe{
        let idx = (&mut *GLOBALS.shared_buffers_arena).insert(WBuffer::new(
          &w_device.device,
          &mut w_device.allocator,
          usage,
          sz_bytes,
          true
        ));

        let buffer = (&mut *GLOBALS.shared_buffers_arena)[idx].borrow_mut();
        let buff_idx = WAIdxBuffer { idx};
        (buff_idx, buffer)
      }
  }

  pub fn new_uniform_buffer(
    &mut self,
    w_device: &mut WDevice,
    sz_bytes: u32,
  ) -> (WAIdxUbo, &mut WBindingUBO) {
    let idx = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena).insert(
      WBindingUBO::new(
        &w_device.device,
        &mut w_device.allocator,
        4 * 100,
      )
    );

    let ubo = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[idx].borrow_mut();
    let ubo_idx = WAIdxUbo { idx};
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
    let bg_idx = WAIdxBindGroup { idx};
    (bg_idx, bind_group)
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