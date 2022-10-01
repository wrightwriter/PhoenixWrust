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

#[derive(Clone, Copy)]
pub struct WSemaphore {
  pub handle: vk::Semaphore,
}
impl WSemaphore {
  pub fn new(w_device: &mut WDevice) -> Self {
    let mut type_info = vk::SemaphoreTypeCreateInfo::builder()
      .semaphore_type(vk::SemaphoreType::TIMELINE)
      .initial_value(0)
      .build();

    let info = vk::SemaphoreCreateInfo::builder().push_next(&mut type_info);

    let handle = unsafe { w_device.device.create_semaphore(&info, None).unwrap() };

    Self { handle }
  }

  pub fn signal_from_host(
    &self,
    w_device: &WDevice,
    signal_value: u64,
  ) {
    let signal_info = vk::SemaphoreSignalInfo::builder()
      .value(signal_value)
      .semaphore(self.handle)
      .build();
    unsafe {
      w_device.device.signal_semaphore(&signal_info);
    }
  }

  pub fn wait_from_host(
    &self,
    w_device: &WDevice,
    wait_value: u64,
  ) {
    let wait_info = vk::SemaphoreWaitInfo::builder()
      .semaphores(&[self.handle])
      .values(&[wait_value])
      .build();
    unsafe {
      w_device.device.wait_semaphores(&wait_info, u64::MAX);
    }
  }
  pub fn submit(
    &self,
    w_device: &mut WDevice,
  ) {
    let submit_info = vk::SubmitInfo2::builder();
    // .waitSe
  }
}
