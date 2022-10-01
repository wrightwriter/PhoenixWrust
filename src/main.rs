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

use gpu_alloc_ash::AshMemoryDevice;
use renderdoc::{RenderDoc, V120, V141};
use phoenix_wrust::{
  abs::{wcomputepass::WComputePass, wthing::WThing},
  c_str,
  res::{wimage::WImage, wrendertarget::{WRenderTarget, WRenderTargetCreateInfo}, wshader::WProgram},
  sys::{
    wdevice::WDevice,
    wmanagers::{WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxUbo, WGrouper, WTechLead},
    wswapchain::WSwapchain, wcommandencoder::WCommandEncoder, wbarr::{WBarr, VStage},
  },
  wmemzeroed, wdef,
};

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

// !! ---------- DEFINES ---------- //

const FRAMES_IN_FLIGHT: usize = 2;
const APP_NAME: &str = "Vulkan";
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

// !! ---------- MAIN ---------- //

struct WVulkan<'a> {
  w_device: WDevice,
  w_swapchain: WSwapchain<'a>,
  w_tl: WTechLead,
  w_grouper: WGrouper,
  // w_render_doc: RenderDoc<V120>,
  default_render_targets: Cell<Vec<WRenderTarget<'a>>>,
  shared_ubo: WAIdxUbo,
  shared_bind_group: WAIdxBindGroup,
  frame: usize,
  width: u32,
  height: u32,
}

pub struct Sketch<'a> {
  pub test_img: WAIdxImage,
  pub command_encoder: WCommandEncoder,
  pub test_rt: WRenderTarget<'a> ,
  pub test_buff: WAIdxBuffer,
  pub comp_pass: WComputePass<'a>,
  // pub test_rt: WRenderTarget<'a>,
  pub thing: WThing,
}

impl<'a> WVulkan<'a> {
  fn run(
    mut self,
    mut event_loop: EventLoop<()>,
    window: &Window,
  ) -> () {
    // !! ---------- Init rendering ---------- //
    
    // let test_rt: WRenderTarget = wmemzeroed!();
    // let test_rt = WRenderTarget::new(self., command_pool, format, images);
      
    // let test_rt = WRenderTargetCreateInfo{..Default::default()};
    // wmemzeroed!();
    // wdef!();

    let command_encoder = WCommandEncoder::new();

    let binding = WRenderTargetCreateInfo{ ..wdef!() };
    let test_rt = binding.build(&mut self.w_device, &mut self.w_tl);

    let mut test_img = self
      .w_tl
      .new_image(
        &mut self.w_device,
        vk::Format::R32G32B32A32_SFLOAT,
        1000,
        1000,
        1,
      )
      .0;

    {
      self.w_grouper.bind_groups_arena[self.shared_bind_group.idx]
        .borrow_mut()
        .rebuild_all(
          &self.w_device.device,
          &self.w_device.descriptor_pool,
          &mut self.w_tl,
        );
    }

    // self.shared_ubo.

    // let test_buff = WBuffer::new(
    //   &self.w_device.device,
    //   &mut self.w_device.allocator,
    //   vk::BufferUsageFlags::STORAGE_BUFFER,
    //   1000,
    // );
    //
    //

    let mut test_buff = self
      .w_tl
      .new_buffer(
        &mut self.w_device,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        1000,
      )
      .0;

    // !! ---------- SHADER ---------- //
    let prog_render = WProgram::new_render_program(
      &self.w_device.device,
      "./shaders".to_string(),
      "D:\\Programming\\Demoscene\\PhoenixWrust\\src\\shaders\\triangle.vert".to_string(),
      "D:\\Programming\\Demoscene\\PhoenixWrust\\src\\shaders\\triangle.frag".to_string(),
    );

    let prog_compute = WProgram::new_compute_program(
      &self.w_device.device,
      "./shaders".to_string(),
      "D:\\Programming\\Demoscene\\PhoenixWrust\\src\\shaders\\compute.comp".to_string(),
    );

    // !! ---------- COMP ---------- //
    let mut comp_pass = WComputePass::new(
      &mut self.w_device,
      &mut self.w_grouper,
      &mut self.w_tl,
      self.shared_bind_group,
      &prog_compute,
    );

    // let mut arr = self.w_tech_lead.ubo_arena[thing.ubo.idx]
    //   .borrow_mut()
    //   .buff
    //   .mapped_array
    //   .as_ptr();

    // !! ---------- Thing ---------- //

    let mut thing = WThing::new(
      &mut self.w_device,
      &mut self.w_grouper,
      &mut self.w_tl,
      self.shared_bind_group,
      &self.w_swapchain.default_render_targets[0],
      &prog_render, // &self.w_device.descriptor_pool,
                    // &mut self.ubo_arena,
    );



    let mut sketch = Sketch {
      test_img,
      test_buff,
      comp_pass,
      thing,
      test_rt,
      command_encoder
    };

    fn aaaaaa(
      w: &mut WVulkan,
      sketch: &mut Sketch,
      rt: &mut WRenderTarget,
      wait_semaphore: vk::Semaphore,
      signal_semaphore: vk::Semaphore,
    ) {
      unsafe {

    // !! ---------- RECORD ---------- //
        sketch.command_encoder.reset(&mut w.w_device);

        {
          rt.begin_pass(&w.w_device.device);
          sketch
            .thing
            .draw(&mut w.w_device, &mut w.w_grouper,  &mut w.w_tl,&rt.command_buffer);
          rt.end_pass(&w.w_device.command_pool, &w.w_device.device);
        }


        {
          sketch.test_rt.begin_pass(&w.w_device.device);
          sketch
            .thing
            .draw(&mut w.w_device, &mut w.w_grouper,  &mut w.w_tl,&sketch.test_rt.command_buffer);
          sketch.test_rt.end_pass(&w.w_device.command_pool, &w.w_device.device);
        }


        sketch
          .comp_pass
          .dispatch(&w.w_device, &w.w_grouper, 1, 1, 1);


    // !! ---------- SUBMIT ---------- //

        sketch.command_encoder.add_barr(&mut w.w_device, 
          &WBarr::new_general_barr()
            .src_stage(VStage::BOTTOM_OF_PIPE)
            .dst_stage(VStage::TOP_OF_PIPE)
        );

        sketch.command_encoder.add_command(sketch.test_rt.command_buffer);

        sketch.command_encoder.add_barr(&mut w.w_device, 
          &WBarr::new_general_barr()
            .src_stage(VStage::BOTTOM_OF_PIPE)
            .dst_stage(VStage::TOP_OF_PIPE)
        );

        sketch.command_encoder.add_command(sketch.comp_pass.command_buffer);

        sketch.command_encoder.add_barr(&mut w.w_device, 
          &WBarr::new_general_barr()
            .src_stage(VStage::BOTTOM_OF_PIPE)
            .dst_stage(VStage::TOP_OF_PIPE)
        );

        sketch.command_encoder.run(&mut w.w_device);

        // w.w_device.device.queue_wait_idle(w.w_device.queue);


        let mut cmd_buffs = [vk::CommandBufferSubmitInfo::builder()
          .command_buffer(rt.command_buffer)
          .build()];

        // ! Reset curr fence and submit

        unsafe {
          let wait_semaphore_submit_infos = [vk::SemaphoreSubmitInfo::builder()
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .semaphore(wait_semaphore)
            .build()];
          let signal_semaphore_submit_infos = [vk::SemaphoreSubmitInfo::builder()
            .semaphore(signal_semaphore)
            .build()];

          let submit_info = vk::SubmitInfo2::builder()
            .wait_semaphore_infos(&wait_semaphore_submit_infos)
            .command_buffer_infos(&cmd_buffs)
            .signal_semaphore_infos(&signal_semaphore_submit_infos)
            .build();
          let in_flight_fence = w.w_swapchain.in_flight_fences[w.frame as usize];
          w.w_device.device.reset_fences(&[in_flight_fence]).unwrap();
          w.w_device
            .device
            .queue_submit2(w.w_device.queue, &[submit_info], in_flight_fence)
            .unwrap();
        }
      };
    }

    event_loop.run_return(move |event, _, control_flow| {
      // *control_flow = ControlFlow::Wait;
      match event {
        Event::NewEvents(StartCause::Init) => {
          *control_flow = ControlFlow::Poll;
        }
        Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
          _ => (),
        },
        Event::DeviceEvent { event, .. } => match event {
          DeviceEvent::Key(KeyboardInput {
            virtual_keycode: Some(keycode),
            state,
            ..
          }) => match (keycode, state) {
            (VirtualKeyCode::Escape, ElementState::Released) => *control_flow = ControlFlow::Exit,
            _ => (),
          },
          _ => (),
        },

        // Wait fence -> wait RT semaphore ->                     -> Reset Fence -> Render with fence
        Event::MainEventsCleared => unsafe {
          // ! ---------- Render Loop ---------- //
          // ! WAIT GPU TO BE DONE WITH OTHER FRAME
          unsafe {
            self
              .w_device
              .device
              .wait_for_fences(
                &[self.w_swapchain.in_flight_fences[self.frame]],
                true,
                u64::MAX,
              )
              .unwrap();
          }

          // ! Wait for other image idx from swapchain
          let image_index = unsafe {
            self
              .w_swapchain
              .swapchain_loader
              .acquire_next_image(
                self.w_swapchain.swapchain,
                u64::MAX,
                self.w_swapchain.image_available_semaphores[self.frame as usize],
                vk::Fence::null(),
              )
              .unwrap()
              .0
          };

          // !
          // * Submit stuff
          // !
          let rt = (self
            .w_swapchain
            .default_render_targets
            .index_mut(image_index as usize) as *mut WRenderTarget)
            .as_mut()
            .unwrap();
          let signal_semaphore = self.w_swapchain.render_finished_semaphores[self.frame as usize];
          let wait_semaphore = self.w_swapchain.image_available_semaphores[self.frame as usize];

          // |w_device: &mut WDevice, rt: &mut WRenderTarget, signal_semaphore: &vk::Semaphore, wait_semaphore: &vk::Semaphore| {
          // bbb(rt, &signal_semaphore, &wait_semaphore);
          // $e
          aaaaaa(
            &mut self,
            &mut sketch,
            rt,
            wait_semaphore,
            signal_semaphore,
          );

          {
            // ! Present

            let sem = [signal_semaphore];
            unsafe {
              let swapchains = vec![self.w_swapchain.swapchain];
              let image_indices = vec![image_index];
              let present_info = {
                vk::PresentInfoKHR::builder()
                  .wait_semaphores(&sem)
                  .swapchains(&swapchains)
                  .image_indices(&image_indices)
              };
              self
                .w_swapchain
                .swapchain_loader
                .queue_present(self.w_device.queue, &present_info)
            }
            .unwrap();

            self.frame = (self.frame + 1) % FRAMES_IN_FLIGHT;
          }
        },
        _ => (),
      }
    });
  }

  fn init_window(event_loop: &EventLoop<()>) -> Window {
    let window = WindowBuilder::new()
      .with_resizable(false)
      .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
      .build(event_loop)
      .unwrap();

    return window;
  }
  fn new(window: &'a Window) -> WVulkan {
    let (mut w_device, w_swapchain) = WDevice::init_device_and_swapchain(window);
    let default_render_targets = unsafe { MaybeUninit::zeroed().assume_init() };

    let mut w_tech_lead = WTechLead::new(&mut w_device);

    // let shared_binding_image_array = w_tech_lead.shared_binding_image_array.clone();

    let mut w_grouper = WGrouper {
      bind_groups_arena: Arena::new(),
    };

    // ubo_shared.buff.mapped_array
    // let shared_images = vec![].reserve( 32);

    let mut shared_ubo = w_tech_lead.new_uniform_buffer(&mut w_device, 32 * 10).0;

    let mut shared_bind_group = w_grouper.new_group(&mut w_device);

    shared_bind_group.1.set_binding_ubo(0, shared_ubo.idx);

    // shared_bind_group.1.image_array_binding = Some( shared_binding_image_array);
    shared_bind_group.1.image_array_binding = Some(w_tech_lead.shared_binding_image_array.clone());

    // shared_bind_group.1.set_binding(2,WBindingImageArray(shared_binding_image_array));

    let wv = Self {
      width: w_swapchain.width,
      height: w_swapchain.height,
      w_tl: w_tech_lead,
      w_swapchain,
      default_render_targets,
      shared_ubo,
      shared_bind_group: shared_bind_group.0,
      frame: 0,
      w_device,
      w_grouper,
      // w_render_doc,
    };

    // wv.default_render_targets.set(
    //     Cell::new(
    //         default_render_targets
    //     )
    // );

    wv
  }

  fn draw_frame(&mut self) {}

  fn destroy(&self) {
    // unsafe{ self.instance.destroy_instance(None); }
  }
}

fn main() {
  // let w_render_doc:RenderDoc<V141> = RenderDoc::new().expect("Unable to set up renderdoc");

  let event_loop: EventLoop<()>;
  event_loop = EventLoop::new();

  let window = WVulkan::init_window(&event_loop);

  let wvulkan = WVulkan::new(&window);

  wvulkan.run(event_loop, &window);
}
