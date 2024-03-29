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
  res::img::wimage::WImage,
  res::{wshader::WProgram, buff::wbuffer::WBuffer},
  res::{img::wrendertarget::WRenderTarget, wshader::WShaderEnumPipelineBind, wvideo::WVideo},
  wmemzeroed, wtransmute,
};
use crate::{
  c_str,
  res::{
    self,
    wbindings::{WBindingBufferArray, WBindingImageArray, WBindingUBO},
    wpongabletrait::WPongableTrait,
    img::wrendertarget::WRTInfo,
    wshader::WShader,
  },
  sys::pipeline::wbindgroup::WBindGroup,
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
  pipeline::wcomputepipeline::WComputePipeline,
  wdevice::{Globals, GLOBALS},
  pipeline::wrenderpipeline::WRenderPipeline,
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
  fn get(&self) -> &T;
}

pub enum WBindType {
  WBindTypeImage,
  WBindTypeUbo,
  WBindTypeBuffer,
}

#[macro_export]
macro_rules! declare_arena_item {
    (
      // $struct:ident {
      //   $( $field:ident:$type:ty ),*
      // },
      $main_type:ty,
      $arena_type:ident,
      $arena_variable: expr
    ) =>{
      #[derive(Debug, Copy, Clone)]
      pub struct $arena_type {
          pub idx: generational_arena::Index,
      }
      impl WArenaItem<$main_type> for $arena_type {
        fn get_arena_idx(&self) -> generational_arena::Index {
          self.idx
        }
        fn get(&self) -> &$main_type {
          unsafe { 
            w_ptr_to_ref!($arena_variable)[self.idx].borrow() 
          }
        }

        fn get_mut(&self) -> &mut $main_type {
          unsafe { w_ptr_to_mut_ref!($arena_variable)[self.idx].borrow_mut() }
        }
      }
    };
}

// -----------
declare_arena_item!(WBindGroup, WAIdxBindGroup, GLOBALS.bind_groups_arena);
declare_arena_item!(WImage, WAIdxImage, GLOBALS.shared_images_arena);
impl WBinding for WAIdxImage {
  fn get_type(&self) -> WBindType {
    WBindType::WBindTypeImage
  }
}

declare_arena_item!(WRenderPipeline, WAIdxRenderPipeline, GLOBALS.shared_render_pipelines);
declare_arena_item!(WComputePipeline, WAIdxComputePipeline, GLOBALS.shared_compute_pipelines);
declare_arena_item!(WProgram, WAIdxShaderProgram, GLOBALS.shader_programs_arena);
declare_arena_item!(WBindingUBO, WAIdxUbo, GLOBALS.shared_ubo_arena);
impl WBinding for WAIdxUbo {
  fn get_type(&self) -> WBindType {
    WBindType::WBindTypeUbo
  }
}

declare_arena_item!(WRenderTarget, WAIdxRt, GLOBALS.shared_render_targets_arena);
declare_arena_item!(WBuffer, WAIdxBuffer, GLOBALS.shared_buffers_arena);
declare_arena_item!(WVideo, WAIdxVideo, GLOBALS.shared_videos_arena);











// #[derive(Debug, Copy, Clone)]
// pub struct WAIdxBindGroup {
//   pub idx: generational_arena::Index,
// }
// impl WArenaItem<WBindGroup> for WAIdxBindGroup {
//   fn get_arena_idx(&self) -> generational_arena::Index {
//     self.idx
//   }

//   fn get(&self) -> &WBindGroup {
//     unsafe {
//       let b = &*std::ptr::null_mut() as &WBindGroup;
//       b
//     }
//   }
//   fn get_mut(&self) -> &mut WBindGroup {
//     unsafe {
//       &mut (*GLOBALS.bind_groups_arena)[self.idx] as &mut WBindGroup
//     }
//   }
// }
// -----------

// #[derive(Debug, Copy, Clone)]
// pub struct WAIdxImage {
//   pub idx: generational_arena::Index,
// }
// impl WArenaItem<WImage> for WAIdxImage {
//   fn get_arena_idx(&self) -> generational_arena::Index {
//     self.idx
//   }
//   fn get(&self) -> &WImage {
//     unsafe { 
//       w_ptr_to_ref!(GLOBALS.shared_images_arena)[self.idx].borrow() 
//     }
//   }

//   fn get_mut(&self) -> &mut WImage {
//     unsafe { w_ptr_to_mut_ref!(GLOBALS.shared_images_arena)[self.idx].borrow_mut() }
//   }
// }
// impl WBinding for WAIdxImage {
//   fn get_type(&self) -> WBindType {
//     WBindType::WBindTypeImage
//   }
// }

// -----------


// #[derive(Debug, Copy, Clone)]
// pub struct WAIdxRenderPipeline {
//   pub idx: generational_arena::Index,
// }

// impl WAIdxRenderPipeline{
//   // fn get_ptr(&self) -> *mut WRenderPipeline {
//   //   unsafe { (*GLOBALS.shared_render_pipelines)[self.idx]}
//   // }
// }
// impl WArenaItem<WRenderPipeline> for WAIdxRenderPipeline {
//   fn get_arena_idx(&self) -> generational_arena::Index {
//     self.idx
//   }
//   fn get_mut(&self) -> &mut WRenderPipeline {
//     unsafe { w_ptr_to_mut_ref!(GLOBALS.shared_render_pipelines)[self.idx].borrow_mut() }
//   }
//   fn get(&self) -> &WRenderPipeline {
//     unsafe { w_ptr_to_ref!(GLOBALS.shared_render_pipelines)[self.idx].borrow() }
//   }
// }

// -----------


// #[derive(Debug, Copy, Clone)]
// pub struct WAIdxComputePipeline {
//   pub idx: generational_arena::Index,
// }
// impl WArenaItem<WComputePipeline> for WAIdxComputePipeline {
//   fn get_arena_idx(&self) -> generational_arena::Index {
//     self.idx
//   }
//   fn get_mut(&self) -> &mut WComputePipeline {
//     unsafe { w_ptr_to_mut_ref!(GLOBALS.shared_compute_pipelines)[self.idx].borrow_mut() }
//   }
//   fn get(&self) -> & WComputePipeline {
//     unsafe { w_ptr_to_ref!(GLOBALS.shared_compute_pipelines)[self.idx].borrow() }
//   }
// }
// -----------

// #[derive(Debug, Copy, Clone)]
// pub struct WAIdxShaderProgram {
//   pub idx: generational_arena::Index,
// }
// impl WArenaItem<WProgram> for WAIdxShaderProgram {
//   fn get_arena_idx(&self) -> generational_arena::Index {
//     self.idx
//   }
//   fn get(&self) -> & WProgram {
//     unsafe { w_ptr_to_ref!(GLOBALS.shader_programs_arena).borrow_mut()[self.idx].borrow() }
//   }
//   fn get_mut(&self) -> &mut WProgram {
//     unsafe { w_ptr_to_mut_ref!(GLOBALS.shader_programs_arena).borrow_mut()[self.idx].borrow_mut() }
//   }
// }

// -----------

// #[derive(Debug, Copy, Clone)]
// pub struct WAIdxUbo {
//   pub idx: generational_arena::Index,
// }
// impl WArenaItem<WBindingUBO> for WAIdxUbo {
//   fn get_arena_idx(&self) -> generational_arena::Index {
//     self.idx
//   }
//   fn get_mut(&self) -> &mut WBindingUBO {
//     unsafe { w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[self.idx].borrow_mut() }
//   }
//   fn get(&self) -> & WBindingUBO {
//     unsafe { w_ptr_to_ref!(GLOBALS.shared_ubo_arena)[self.idx].borrow() }
//   }
// }
// impl WBinding for WAIdxUbo {
//   fn get_type(&self) -> WBindType {
//     WBindType::WBindTypeUbo
//   }
// }

// // -----------


// #[derive(Debug, Copy, Clone)]
// pub struct WAIdxBuffer {
//   pub idx: generational_arena::Index,
// }
// impl WArenaItem<WBuffer> for WAIdxBuffer {
//   fn get_arena_idx(&self) -> generational_arena::Index {
//     self.idx
//   }
//   fn get(&self) -> &WBuffer {
//     unsafe {
//       w_ptr_to_ref!(GLOBALS.shared_buffers_arena)[self.idx].borrow()
//     }
//   }

//   fn get_mut(&self) -> &mut WBuffer {
//     unsafe {
//       w_ptr_to_mut_ref!(GLOBALS.shared_buffers_arena)[self.idx].borrow_mut()
//     }
//   }
// }
// impl WBinding for WAIdxBuffer {
//   fn get_type(&self) -> WBindType {
//     WBindType::WBindTypeBuffer
//   }
// }

// // // -----------

// #[derive(Debug, Copy, Clone)]
// pub struct WAIdxRt {
//   pub idx: generational_arena::Index,
// }
// impl WArenaItem<WRenderTarget> for WAIdxRt {
//   fn get_arena_idx(&self) -> generational_arena::Index {
//     self.idx
//   }
//   fn get(&self) -> &WRenderTarget {
//     unsafe {
//       w_ptr_to_ref!(globals.shared_render_targets_arena)[self.idx].borrow()
//     }
//   }

//   fn get_mut(&self) -> &mut WRenderTarget {
//     unsafe {
//       w_ptr_to_mut_ref!(GLOBALS.shared_render_targets_arena)[self.idx].borrow_mut()
//     }
//   }
// }

