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

pub type VStage = vk::PipelineStageFlags2;

#[derive(Clone, Copy)]
enum BarrierType {
  Image(vk::ImageMemoryBarrier2),
  General(vk::MemoryBarrier2),
  Buffer(vk::BufferMemoryBarrier2),
}
#[derive(Clone, Copy)]
pub struct WBarr {
  barrier: BarrierType,
}
impl WBarr {
  pub fn run(
    &self,
    w_device: &WDevice,
    command_buffer: vk::CommandBuffer,
  ) -> WBarr {
    unsafe {
      match &self.barrier {
        BarrierType::Image(__) => {
          let mem_bar = [*__];
          let dep = vk::DependencyInfo::builder()
            .image_memory_barriers(&mem_bar)
            .build();
          w_device.device.cmd_pipeline_barrier2(command_buffer, &dep);
        }
        BarrierType::General(__) => {
          let mem_bar = [*__];
          let dep = vk::DependencyInfo::builder()
            .memory_barriers(&mem_bar)
            .build();
          w_device.device.cmd_pipeline_barrier2(command_buffer, &dep);
        }
        BarrierType::Buffer(__) => {
          // let mem_bar = [ &*vk::DependencyInfo::builder().buffer_memory_barriers(__).build()],
          let mem_bar = [*__];
          let dep = vk::DependencyInfo::builder()
            .buffer_memory_barriers(&mem_bar)
            .build();
          w_device.device.cmd_pipeline_barrier2(command_buffer, &dep);
        }
      }
    };
    *self
  }
  pub fn new_image_barr() -> WBarr {
    let subresource_range = vk::ImageSubresourceRange::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1)
      .build();
    let barrier = BarrierType::Image(
      vk::ImageMemoryBarrier2::builder()
        .subresource_range(subresource_range)
        .build(),
    );
    WBarr { barrier }
  }
  pub fn new_general_barr() -> WBarr {
    WBarr {
      barrier: BarrierType::General(vk::MemoryBarrier2::builder().build()),
    }
  }
  pub fn new_buffer_barr() -> WBarr {
    WBarr {
      barrier: BarrierType::Buffer(vk::BufferMemoryBarrier2::builder().build()),
    }
  }
  pub fn old_layout(
    &mut self,
    layout: vk::ImageLayout,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.old_layout = layout;
      }
      BarrierType::General(_) => {}
      BarrierType::Buffer(_) => {}
    };
    *self
  }
  pub fn new_layout(
    &mut self,
    layout: vk::ImageLayout,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.new_layout = layout;
      }
      BarrierType::General(_) => {}
      BarrierType::Buffer(_) => {}
    };
    *self
  }
  // fn image(&mut self, layout: vk::ImageLayout )->WBarr{
  //   match &mut self.barrier {
  //     BarrierType::Image(__) => {__.new_layout = layout;},
  //     BarrierType::General(_) => {},
  //     BarrierType::Buffer(_) => {},
  //   };
  //   *self
  // }
  pub fn src_stage(
    &mut self,
    stage: vk::PipelineStageFlags2,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.src_stage_mask = stage;
      }
      BarrierType::General(__) => {
        __.src_stage_mask = stage;
      }
      BarrierType::Buffer(__) => {
        __.src_stage_mask = stage;
      }
    };
    *self
  }
  pub fn dst_stage(
    &mut self,
    stage: vk::PipelineStageFlags2,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.dst_stage_mask = stage;
      }
      BarrierType::General(__) => {
        __.dst_stage_mask = stage;
      }
      BarrierType::Buffer(__) => {
        __.dst_stage_mask = stage;
      }
    };
    *self
  }
  pub fn src_access(
    &mut self,
    access: vk::AccessFlags2,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.src_access_mask = access;
      }
      BarrierType::General(__) => {
        __.src_access_mask = access;
      }
      BarrierType::Buffer(__) => {
        __.src_access_mask = access;
      }
    };
    *self
  }
  pub fn dst_access(
    &mut self,
    access: vk::AccessFlags2,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.dst_access_mask = access;
      }
      BarrierType::General(__) => {
        __.dst_access_mask = access;
      }
      BarrierType::Buffer(__) => {
        __.dst_access_mask = access;
      }
    };
    *self
  }
  // fn image(&mut self, image: &WImage)->WBarr{
  // }
}
