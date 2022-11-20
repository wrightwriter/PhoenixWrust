#[allow(non_snake_case)]
use ash::vk::{self};
use ash::vk::{BufferUsageFlags, Semaphore};

use generational_arena::Arena;

use lazy_static::lazy_static;
use nalgebra_glm::vec2;
use sync_unsafe_cell::SyncUnsafeCell;
use tracy_client::span;

// use renderdoc::{RenderDoc, V120, V141};
use crate::{
  abs::{
    wcam::WCamera,
    wcomputepass::WComputePass,
    wfxcomposer::WFxComposer,
    wpostpass::{WFxPass, WKernelPass, WPassTrait},
    wthing::WThing,
    wthingshape::WThingPath,
    wthingtext::WThingText,
  },
  msdf::msdf::WFont,
  res::{
    buff::wwritablebuffertrait::WWritableBufferTrait,
    img::wimage::WImageInfo,
    img::wrendertarget::{WRenderTarget, WRTInfo},
    wmodel::WModel,
    wvideo::WVideo,
  },
  sys::{
    warenaitems::{WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxRt, WAIdxUbo, WArenaItem},
    wbarr::WBarr,
    wcommandencoder::WCommandEncoder,
    wdevice::{WDevice, GLOBALS},
    wgui::WGUI,
    winput::WInput,
    wmanagers::{ WTechLead},
    wrecorder::WRecorder,
    wshaderman::WShaderMan,
    wswapchain::WSwapchain,
    wtime::WTime,
  },
  wdef,
  wsketch::{init_sketch, render_sketch, Sketch}, wsketchflame::SketchFlame,
};

// use smallvec::SmallVec;
use winit::{dpi::LogicalSize, platform::run_return::EventLoopExtRunReturn, window};

use winit::{
  event::{DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  window::WindowBuilder,
};

use std::{
  borrow::BorrowMut,
  cell::{Cell, RefMut},
  ops::IndexMut,
  time::{Duration, Instant},
};

// !! ---------- DEFINES ---------- //

const FRAMES_IN_FLIGHT: usize = 2;
const APP_NAME: &str = "Vulkan";
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

// !! ---------- MAIN ---------- //

pub struct WVulkan {
  pub w_device: WDevice,
  pub w_swapchain: WSwapchain,
  // pub w_tl: WTechLead,
  // pub w_grouper: WGrouper,
  pub w_shader_man: WShaderMan,
  pub w_recorder: WRecorder,
  pub w_cam: WCamera,
  pub w_input: WInput,
  pub w_gui: WGUI,
  pub w_time: WTime,

  // w_render_doc: RenderDoc<V120>,
  pub gui_enabled: bool,
  pub shared_ubo: WAIdxUbo,
  pub shared_bind_group: WAIdxBindGroup,
  pub frame: usize,
  pub width: u32,
  pub height: u32,
}

impl<'a> WVulkan {
  pub fn init_window(event_loop: &EventLoop<()>) -> Window {
    let window = WindowBuilder::new()
      .with_resizable(false)
      .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
      .with_title("👩‍💻")
      .build(event_loop)
      .unwrap();

    return window;
  }
  pub fn new(window: &'a Window) -> WVulkan {
    let (mut w_device, w_swapchain) = WDevice::init_device_and_swapchain(window);
    // let mut w_tl = WTechLead::new(&mut w_device);
    let w_tl = unsafe {
      let w_tl = Box::new(WTechLead::new(&mut w_device));
      GLOBALS.w_tl = Box::into_raw(w_tl);
      &mut *GLOBALS.w_tl
    };

    let shared_ubo = w_tl.new_uniform_buffer(&mut w_device, 32 * 20).0;
    let mut shared_bind_group = w_tl.new_group(&mut w_device).0;
    unsafe {
      {
        let sbg = &mut (*GLOBALS.bind_groups_arena)[shared_bind_group.idx];

        sbg.set_binding_ubo(0, shared_ubo.idx);
        sbg.image_array_binding = Some(GLOBALS.shared_binding_images_array);

        sbg.buffer_array_binding = Some(GLOBALS.shared_binding_buffers_array);
      }

      (*GLOBALS.bind_groups_arena)[shared_bind_group.idx]
        .borrow_mut()
        .rebuild_all(&w_device.device, &w_device.descriptor_pool, w_tl);
    }

    let mut w_cam = WCamera::new(w_swapchain.width, w_swapchain.height);
    w_cam.update_matrices();

    let wv = Self {
      width: w_swapchain.width,
      height: w_swapchain.height,
      gui_enabled: true,
      // w_tl,
      w_swapchain,
      shared_ubo,
      shared_bind_group: shared_bind_group,
      frame: 0,
      w_device,
      // w_grouper,
      w_shader_man: WShaderMan::new(),
      w_cam,
      w_input: WInput::new(),
      w_time: WTime::new(),
      w_gui: WGUI::new(),
      w_recorder: WRecorder::new(),
      // w_render_doc,
    };
    wv
  }

  pub fn run(
    mut event_loop: EventLoop<()>,
    window: &Window,
  ) -> () {
    // !! ---------- Init rendering ---------- //

    let mut sketch = crate::wsketchflame::init_sketch();
    // unsafe {
    //   let WV = &mut (*GLOBALS.w_vulkan);
    //   let w_tl = &mut (*GLOBALS.w_tl);

    //   (*GLOBALS.bind_groups_arena)[WV.shared_bind_group.idx].borrow_mut().rebuild_all(
    //     &WV.w_device.device,
    //     &WV.w_device.descriptor_pool,
    //     w_tl,
    //   );
    // }

    #[profiling::function]
    fn render(
      s: &mut SketchFlame,
      rt: &mut WRenderTarget,
      imgui_cmd_buff: vk::CommandBuffer,
      wait_semaphore: vk::Semaphore,
      signal_semaphore: vk::Semaphore,
    ) {
      crate::wsketchflame::render_sketch(s, rt, imgui_cmd_buff, wait_semaphore, signal_semaphore);
    }

    event_loop.run_return(move |event, _, control_flow| {
      unsafe {
        let mut imgui = (*GLOBALS.imgui).borrow_mut();
        (*GLOBALS.w_vulkan).w_device.platform.handle_event(imgui.io_mut(), &window, &event);
      }

      match event {
        Event::NewEvents(StartCause::Init) => {
          *control_flow = ControlFlow::Poll;
        }
        Event::WindowEvent { event, .. } => {handle_window_event(event, &window, control_flow)} 
        Event::DeviceEvent { event, .. } => match event {
          DeviceEvent::Key(KeyboardInput {
            virtual_keycode: Some(keycode),
            state,
            ..
          }) => match (keycode, state) {
            _ => unsafe {
              (*GLOBALS.w_vulkan).w_input.handle_key_press(keycode, state);
            },
          },
          _ => (),
        },
        Event::NewEvents(_) => {
          let now = Instant::now();
          unsafe {
            let mut dt = (*GLOBALS.w_vulkan).w_time.dt;
            if (*GLOBALS.w_vulkan).w_recorder.recording {
              dt = Duration::from_secs_f64(0.1);
            }
            let mut imgui = unsafe { (*GLOBALS.imgui).borrow_mut() };
            imgui.io_mut().update_delta_time(dt);
          }
        }

        // Wait fence -> wait RT semaphore ->                     -> Reset Fence -> Render with fence
        // !! ---------- LOOP ---------- //
        Event::MainEventsCleared => unsafe {
          let (rt, signal_semaphore, wait_semaphore, image_index, imgui_cmd_buf) 
            = begin_frame(window);

          render(&mut sketch, rt, imgui_cmd_buf, wait_semaphore, signal_semaphore);

          {
            let w_input = &mut (*GLOBALS.w_vulkan).w_input;
            w_input.refresh_keys();
            w_input.mouse_state.delta_pos = vec2(0.0, 0.0);
            w_input.mouse_state.delta_pos_normalized = vec2(0.0, 0.0);
          }

          end_frame(rt, signal_semaphore, wait_semaphore, image_index);
        },
        _ => (),
      }
    });
  }

  // fn destroy(&self) {
  //   // unsafe{ self.instance.destroy_instance(None); }
  // }
}


fn update_uniforms(WV: &mut WVulkan) {
  unsafe {
    let ubo = WV.shared_ubo.get_mut();
    let time = &mut WV.w_time;

    // let mut mem_ptr = ubo.buff.mapped_mems[ubo.buff.pong_idx as usize] as *mut f32;
    let ubo_buff = &mut ubo.buff;
    ubo_buff.reset_ptr();

    let cam = &mut (*GLOBALS.w_vulkan).w_cam;
    // vec3
    ubo_buff.write(cam.eye_pos);
    // ubo_buff.write(0.0f32); // padding

    // vec2
    ubo_buff.write(cam.width as f32);
    ubo_buff.write(cam.height as f32);

    ubo_buff.write((*GLOBALS.w_vulkan).w_input.mouse_state.pos_normalized);
    ubo_buff.write((*GLOBALS.w_vulkan).w_input.mouse_state.delta_pos_normalized);
    // ubo_buff.write(0.0f32); // padding
    // ubo_buff.write(0.0f32); // padding

    // float
    ubo_buff.write((*GLOBALS.w_vulkan).w_time.t_f32);
    ubo_buff.write((*GLOBALS.w_vulkan).w_time.dt_f32);
    ubo_buff.write((*GLOBALS.w_vulkan).w_time.frame as u32);

    ubo_buff.write(if (*GLOBALS.w_vulkan).w_input.mouse_state.rmb_down {
      1.0f32
    } else {
      0.0f32
    });
    ubo_buff.write(if (*GLOBALS.w_vulkan).w_input.mouse_state.lmb_down {
      1.0f32
    } else {
      0.0f32
    });

    ubo_buff.write(cam.near as f32);
    ubo_buff.write(cam.far as f32);

    // ubo_buff.write(0.0f32); // padding

    // mat4
    ubo_buff.write(cam.view_mat);
    ubo_buff.write(cam.proj_mat);
    ubo_buff.write(cam.view_proj_mat);
    ubo_buff.write(cam.inv_view_mat);
    ubo_buff.write(cam.inv_proj_mat);

    ubo_buff.write(cam.prev_view_mat);
    ubo_buff.write(cam.prev_proj_mat);
    ubo_buff.write(cam.prev_view_proj_mat);
    ubo_buff.write(cam.prev_inv_view_mat);
    ubo_buff.write(cam.prev_inv_proj_mat);

    // ubo_buff.write(cam.prev_view_proj_mat);
  }
}

fn prepare_ui(
  WV: &mut WVulkan,
  im_gui: &mut RefMut<imgui::Context>,
  window: &winit::window::Window,
  rt: &mut WRenderTarget,
  // imgui_cmd_buf: &vk::CommandBuffer,
  // window
) -> vk::CommandBuffer{
  WV.w_device
    .platform
    .prepare_frame(im_gui.io_mut(), &window)
    .expect("Failed to prepare frame");

  let mut im_ui = im_gui.frame();

  // // #[macro_export]
  // macro_rules! def_im_var {
  //   ($name: expr ,$t: ty, $val: expr) => {unsafe{
  //     pub static ref $name: SyncUnsafeCell<$t> = SyncUnsafeCell::new($val);
  //   }};
  // }
  type ImVar<T> = SyncUnsafeCell<T>;

  lazy_static! {
    pub static ref im_var_run: ImVar<bool> = ImVar::new(false);
    pub static ref imgui_enabled: ImVar<bool> = ImVar::new(false);
    // pub static ref imgui_enabled: ImVar<bool> = ImVar::new(false);
  };

  if !WV.w_recorder.recording && WV.gui_enabled {
    WV.w_gui.draw_internal(
      &mut WV.w_device,
      &mut WV.w_time,
      &mut im_ui,
      &mut WV.w_shader_man,
      &mut WV.w_cam,
      &mut WV.w_recorder,
    );
  }

    WV.w_device.platform.prepare_render(&im_ui, &window);

    let draw_data = im_ui.render();

    let imgui_cmd_buf = rt.begin_pass(&mut WV.w_device);
    WV.w_device.imgui_renderer.cmd_draw(imgui_cmd_buf, draw_data).unwrap();
imgui_cmd_buf
}

fn begin_frame<'a>(window: &winit::window::Window) -> (&'a mut WRenderTarget, Semaphore, Semaphore, u32, vk::CommandBuffer) {
  unsafe {
    profiling::scope!("outer loop");
    // -- profile -- //
    {
      let input = &mut (*GLOBALS.w_vulkan).w_input;
      if GLOBALS.profiling == true {
        profiling::finish_frame!();
        GLOBALS.profiling = false;
      } else if input.get_key(VirtualKeyCode::F12).pressed == true {
        GLOBALS.profiling = true;
      }
      if input.get_key(VirtualKeyCode::LAlt).pressed == true {
        (*GLOBALS.w_vulkan).gui_enabled = !(*GLOBALS.w_vulkan).gui_enabled;
      }
    }

    // -- update time -- //
    {
      let rec = &mut (*GLOBALS.w_vulkan).w_recorder;
      let time = &mut (*GLOBALS.w_vulkan).w_time;
      if rec.recording {
        let dt = (1.0 / (rec.frame_rate as f64)) as f64;
        time.tick_fixed(dt);
      } else {
        time.tick();
      }
    }

    // -- RELOAD SHADERS -- //
    {
      let shader_man = &(*GLOBALS.w_vulkan).w_shader_man;
      if *shader_man.shader_was_modified.lock().unwrap() {
        shader_man.chan_sender_start_shader_comp.send(());
        shader_man.chan_receiver_end_shader_comp.recv().expect("Error: timed out.");
        println!("-- SHADER RELOAD END --")
      }
    }
    fn update_cam() {}

    // -- UPDATE CAM -- //
    {
      let cam = &mut (*GLOBALS.w_vulkan).w_cam;
      let time = &mut (*GLOBALS.w_vulkan).w_time;
      cam.update_movement((*GLOBALS.w_vulkan).w_input.mouse_state, &(*GLOBALS.w_vulkan).w_input, time.dt_f32);
      cam.update_matrices();
    }

    // -- UPDATE UNIFORMS -- //
    {
      update_uniforms(&mut *GLOBALS.w_vulkan);
    }

    let WV = &mut *GLOBALS.w_vulkan;

    WV.w_device
      .device
      .wait_for_fences(&[WV.w_swapchain.in_flight_fences[WV.frame]], true, u64::MAX)
      .unwrap();

    WV.w_device.command_pools[WV.w_device.pong_idx].reset(&WV.w_device.device);

    // ! ---------- Render Loop ---------- //
    // ! WAIT GPU TO BE DONE WITH OTHER FRAME
    // ! Wait for other image idx from swapchain
    let image_index = WV
      .w_swapchain
      .swapchain_loader
      .acquire_next_image(
        WV.w_swapchain.swapchain,
        u64::MAX,
        WV.w_swapchain.image_available_semaphores[WV.frame as usize],
        vk::Fence::null(),
      )
      .unwrap()
      .0;

    // !
    // * Submit stuff
    // !
    let rt = (WV.w_swapchain.default_render_targets.index_mut(image_index as usize) as *mut WRenderTarget)
      .as_mut()
      .unwrap();
    let signal_semaphore = WV.w_swapchain.render_finished_semaphores[WV.frame as usize];
    let wait_semaphore = WV.w_swapchain.image_available_semaphores[WV.frame as usize];

    // UI
    let mut im_gui = (*GLOBALS.imgui).borrow_mut();
    let imgui_cmd_buf = prepare_ui(WV, &mut im_gui, window, rt);

    // imgui_cmd_buf

    rt.end_pass(&mut WV.w_device);

    unsafe {
      let w_tl = &mut (*GLOBALS.w_tl);
      w_tl.pong_all();
    }

    rt.images[0].descriptor_image_info.image_layout = vk::ImageLayout::UNDEFINED;

    (rt, signal_semaphore, wait_semaphore, image_index, imgui_cmd_buf)
  }
}

fn end_frame(
  rt: &mut WRenderTarget,
  signal_semaphore: Semaphore,
  wait_semaphore: Semaphore,
  image_index: u32,
) {
  // ! Reset curr fence and submit
  unsafe {
    let WV = &mut *GLOBALS.w_vulkan;

    unsafe {
      let mut cmd_buffs = [vk::CommandBufferSubmitInfo::builder().command_buffer(rt.cmd_buf).build()];
      let wait_semaphore_submit_infos = [vk::SemaphoreSubmitInfo::builder()
        .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
        .semaphore(wait_semaphore)
        .build()];
      let signal_semaphore_submit_infos = [vk::SemaphoreSubmitInfo::builder().semaphore(signal_semaphore).build()];

      let submit_info = vk::SubmitInfo2::builder()
        .wait_semaphore_infos(&wait_semaphore_submit_infos)
        // .command_buffer_infos(&cmd_buffs)
        .command_buffer_infos(&[])
        .signal_semaphore_infos(&signal_semaphore_submit_infos)
        .build();
      let in_flight_fence = WV.w_swapchain.in_flight_fences[WV.frame as usize];
      WV.w_device.device.reset_fences(&[in_flight_fence]).unwrap();
      WV.w_device
        .device
        .queue_submit2(WV.w_device.queue, &[submit_info], in_flight_fence)
        .unwrap();
    }

    unsafe {
      WV.w_recorder.try_recording(&rt.images[0], &mut WV.w_device);
    }

    unsafe {
      // ! Present
      let sem = [signal_semaphore];

      let swapchains = vec![WV.w_swapchain.swapchain];
      let image_indices = vec![image_index];
      let present_info = {
        vk::PresentInfoKHR::builder()
          .wait_semaphores(&sem)
          .swapchains(&swapchains)
          .image_indices(&image_indices)
      };
      WV.w_swapchain
        .swapchain_loader
        .queue_present(WV.w_device.queue, &present_info)
        .unwrap();

      WV.frame = (WV.frame + 1) % FRAMES_IN_FLIGHT;
      WV.w_device.pong_idx = 1 - WV.w_device.pong_idx;
    }
  }
}

#[allow(deprecated)]
fn handle_window_event(
  event: WindowEvent,
  window: &winit::window::Window,
  control_flow: &mut ControlFlow
){
  match event {
      WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
    WindowEvent::MouseInput {
      device_id,
      state,
      button,
      modifiers,
    } => {
      match button {
        winit::event::MouseButton::Right => match state {
          ElementState::Pressed => {
            window.set_cursor_grab(true);
            window.set_cursor_visible(false);
          }
          ElementState::Released => {
            window.set_cursor_grab(false);
            window.set_cursor_visible(true);
          }
        },
        _ => (),
      }
      unsafe {
        (*GLOBALS.w_vulkan).w_input.handle_mouse_press(button, state);
      }
    }
    WindowEvent::CursorMoved {
      device_id,
      position,
      modifiers,
    } => unsafe {
      (*GLOBALS.w_vulkan).w_input.handle_mouse_move(
        position,
        (*GLOBALS.w_vulkan).w_cam.width as f32,
        (*GLOBALS.w_vulkan).w_cam.height as f32,
      );
    },
    _ => (),
  }

}