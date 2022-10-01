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
  abs::{wcomputepass::WComputePass, wthing::WThing},
  c_str,
  res::{
    wimage::WImage,
    wrendertarget::{WRenderTarget, WRenderTargetCreateInfo},
    wshader::WProgram,
  },
  sys::{
    wdevice::WDevice,
    wmanagers::{WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxUbo, WGrouper, WTechLead},
    wswapchain::WSwapchain,
  },
  wdef, wmemzeroed,
};
use gpu_alloc_ash::AshMemoryDevice;
use renderdoc::{RenderDoc, V120, V141};

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

use std::cell::RefCell;
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

use super::{wbarr::WBarr, wsemaphore::WSemaphore};

pub struct WCommandEncoder {
  // pub command_buffs: SmallVec<[vk::CommandBuffer;40]>,
  pub command_buffs: SmallVec<[vk::CommandBufferSubmitInfo; 32]>,
}

impl WCommandEncoder {
  pub fn new() -> Self {
    Self {
      command_buffs: SmallVec::new(),
    }
  }
  pub fn add_command(
    &mut self,
    command_buff: vk::CommandBuffer,
  ) {
    self.command_buffs.push(
      vk::CommandBufferSubmitInfo::builder()
        .command_buffer(command_buff)
        .build(),
    );
  }

  pub fn add_barr(
    &mut self,
    w_device: &WDevice,
    barrier: &WBarr,
  ) {
    let cmd_buf_allocate_info = vk::CommandBufferAllocateInfo::builder()
      .command_pool(w_device.command_pool)
      .level(vk::CommandBufferLevel::PRIMARY)
      .command_buffer_count(1);

    // TODO: not do this lmao
    unsafe {
      let cmd_buff = w_device
        .device
        .allocate_command_buffers(&cmd_buf_allocate_info)
        .unwrap()[0];

      let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
      w_device
        .device
        .begin_command_buffer(cmd_buff, &cmd_buf_begin_info);

      barrier.run(w_device, cmd_buff);

      w_device.device.end_command_buffer(cmd_buff);
    }
  }

  pub fn run(
    &mut self,
    w_device: &WDevice,
  ) {
    let submit_info = vk::SubmitInfo2::builder()
      .command_buffer_infos(&self.command_buffs)
      .build();

    unsafe {
      w_device
        .device
        .queue_submit2(w_device.queue, &[submit_info], vk::Fence::null())
        .unwrap();
    }
  }

  pub fn run_wait_semaphore(
    &mut self,
    w_device: &WDevice,
    semaphore: &mut WSemaphore,
    wait_value: u64,
  ) {
    let submit_info = vk::SubmitInfo2::builder()
      .command_buffer_infos(&self.command_buffs)
      .build();

    let wait_info = vk::SemaphoreWaitInfo::builder()
      .semaphores(&[semaphore.handle])
      .values(&[wait_value])
      .build();

    unsafe {
      w_device
        .device
        .queue_submit2(w_device.queue, &[submit_info], vk::Fence::null())
        .unwrap();
    }
  }
  
  pub fn reset( &mut self, w_device: &WDevice,){
    unsafe{
      self.command_buffs.iter().map(|__| {
        w_device.device.free_command_buffers(w_device.command_pool, &[__.command_buffer]);
      });
      self.command_buffs.set_len(0);
    }
  }
  // pub fn add_semaphore(&mut self, semaphore: &mut WSemaphore){
  //   self.command_buffs.push(command_buff);
  // }
}
