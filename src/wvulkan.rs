use ash::vk::{self};

use generational_arena::Arena;
use nalgebra_glm::Vec2;

// use renderdoc::{RenderDoc, V120, V141};
use crate::{
  abs::{wcomputepass::WComputePass, wthing::WThing, wcam::WCamera},
  res::{
    wrendertarget::{WRenderTarget, WRenderTargetCreateInfo},
    wshader::WProgram,
  },
  sys::{
    wbarr::{VStage, WBarr},
    wcommandencoder::WCommandEncoder,
    wdevice::{WDevice, GLOBALS},

    warenaitems::{
      WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxRt, WAIdxUbo, WArenaItem,
    },
    wmanagers::{
       WGrouper, WTechLead, WShaderMan,
    },
    wswapchain::WSwapchain, winput::WInput,
  },
  w_ptr_to_mut_ref, wdef,
};

// use smallvec::SmallVec;
use winit::{dpi::LogicalSize, platform::run_return::EventLoopExtRunReturn};

use winit::{
  event::{
    DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent,
  },
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  window::WindowBuilder,
};

use std::{borrow::BorrowMut, cell::Cell, mem::MaybeUninit, ops::{IndexMut, Div}};

// !! ---------- DEFINES ---------- //

const FRAMES_IN_FLIGHT: usize = 2;
const APP_NAME: &str = "Vulkan";
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

// !! ---------- MAIN ---------- //

pub struct WVulkan {
  pub w_device: WDevice,
  pub w_swapchain: WSwapchain,
  pub w_tl: WTechLead,
  pub w_grouper: WGrouper,
  pub w_shader_man: WShaderMan,
  pub w_cam: WCamera,
  pub w_input: WInput,
  
  // w_render_doc: RenderDoc<V120>,
  pub default_render_targets: Cell<Vec<WRenderTarget>>,
  pub shared_ubo: WAIdxUbo,
  pub shared_bind_group: WAIdxBindGroup,
  pub frame: usize,
  pub width: u32,
  pub height: u32,
}

pub struct Sketch {
  pub test_img: WAIdxImage,
  pub command_encoder: WCommandEncoder,
  pub test_rt: WAIdxRt,
  pub test_buff: WAIdxBuffer,
  pub comp_pass: WComputePass,
  // pub test_rt: WRenderTarget<'a>,
  pub thing: WThing,
}

impl<'a> WVulkan {
  pub fn run(
    mut event_loop: EventLoop<()>,
    window: &Window,
  ) -> () {
    // !! ---------- Init rendering ---------- //

    let mut sketch = unsafe {
      let WV = &mut *GLOBALS.w_vulkan;
      let command_encoder = WCommandEncoder::new();

      let test_rt = WRenderTargetCreateInfo { ..wdef!() };
      let test_rt = WV.w_tl.new_render_target(&mut WV.w_device, test_rt).0;

      let mut test_img = WV
        .w_tl
        .new_image(
          &mut WV.w_device,
          vk::Format::R32G32B32A32_SFLOAT,
          1000,
          1000,
          1,
        )
        .0;

      {
        WV.w_grouper.bind_groups_arena[WV.shared_bind_group.idx]
          .borrow_mut()
          .rebuild_all(
            &WV.w_device.device,
            &WV.w_device.descriptor_pool,
            &mut WV.w_tl,
          );
      }

      let mut test_buff = WV
        .w_tl
        .new_buffer(
          &mut WV.w_device,
          vk::BufferUsageFlags::STORAGE_BUFFER,
          1000,
        )
        .0;
      


      // !! ---------- SHADER ---------- //
      let prog_render = WV.w_shader_man.new_render_program(
        &mut WV.w_device,
        // "./shaders".to_string(),
        "triangle.vert".to_string(),
        "triangle.frag".to_string(),
      );
      // let prog_render = WProgram::new_render_program(
      // );

      let prog_compute = WV.w_shader_man.new_compute_program(
        &mut WV.w_device,
        // "./shaders".to_string(),
        "compute.comp".to_string(),
      );
      // );

      // !! ---------- COMP ---------- //
      let mut comp_pass = WComputePass::new(
        &mut WV.w_device,
        &mut WV.w_grouper,
        &mut WV.w_tl,
        WV.shared_bind_group,
        prog_compute
        // &WV.w_shader_man.shaders_arena.lock().unwrap()[prog_compute.idx],
        // &prog_compute,
      );

      // let mut arr = WV.w_tech_lead.ubo_arena[thing.ubo.idx]
      //   .borrow_mut()
      //   .buff
      //   .mapped_array
      //   .as_ptr();

      // !! ---------- Thing ---------- //

      let mut thing = WThing::new(
        &mut WV.w_device,
        &mut WV.w_grouper,
        &mut WV.w_tl,
        WV.shared_bind_group,
        &WV.w_swapchain.default_render_targets[0],
        prog_render, // &WV.w_device.descriptor_pool,
                      // &mut WV.ubo_arena,
      );

      let mut sketch = Sketch {
        test_img,
        test_buff,
        comp_pass,
        thing,
        test_rt,
        command_encoder,
      };


      // big brain 🧠🧠
      WV.w_device.device.queue_wait_idle(WV.w_device.queue);
      sketch
    };

    fn render(
      // w: &mut WVulkan,
      s: &mut Sketch,
      rt: &mut WRenderTarget,
      wait_semaphore: vk::Semaphore,
      signal_semaphore: vk::Semaphore,
    ) {
      unsafe {
        let w = &mut *GLOBALS.w_vulkan;
        // !! ---------- RECORD ---------- //
        s.command_encoder.reset(&mut w.w_device);

        w.w_tl.pong_all();

        // {
        //   let ubo = s.thing.ubo.get_mut();
        //   unsafe {
        //     *(ubo.buff.mapped_mems[ubo.buff.pong_idx as usize] as *mut f32) = 0f32;
        //   }
        // }

        {
          rt.begin_pass(&mut w.w_device);
          s.thing
            .draw(&mut w.w_device, &mut w.w_grouper, &mut w.w_tl, &rt.cmd_buf);
          rt.end_pass(&w.w_device);
        }

        s.command_encoder
          .add_barr(&mut w.w_device, &WBarr::new_general_barr());

        {
          let test_rt = s.test_rt.get_mut();

          test_rt.begin_pass(&mut w.w_device);
          s.thing
            .draw(&mut w.w_device, &mut w.w_grouper, &w.w_tl, &test_rt.cmd_buf);
          test_rt.end_pass(&w.w_device);

          s.command_encoder.push(test_rt.cmd_buf);
        }

        s.command_encoder
          .add_barr(&mut w.w_device, &WBarr::new_general_barr());

        {
          s.comp_pass.dispatch(&mut w.w_device, &w.w_grouper, 1, 1, 1);
          s.command_encoder.push(s.comp_pass.command_buffer);
        }

        s.command_encoder
          .add_barr(&mut w.w_device, &WBarr::new_general_barr());


        // !! ---------- SUBMIT ---------- //

        s.command_encoder.run(&mut w.w_device);

        // w.w_device.device.queue_wait_idle(w.w_device.queue);

        let mut cmd_buffs = [vk::CommandBufferSubmitInfo::builder()
          .command_buffer(rt.cmd_buf)
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
        w.w_input.refresh_keys();
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
          WindowEvent::MouseInput { 
            device_id, 
            state, 
            button, 
            modifiers } => {
              unsafe{
                (*GLOBALS.w_vulkan).w_input.handle_mouse_press(button, state);
              }
          },
          WindowEvent::CursorMoved { 
            device_id, 
            position, 
            modifiers } =>{
              unsafe{
                (*GLOBALS.w_vulkan).w_input.handle_mouse_move(
                    position,
                    (*GLOBALS.w_vulkan).w_cam.width as f32,
                    (*GLOBALS.w_vulkan).w_cam.height as f32,
                );
              }
          }
          // WindowEvent::CursorMoved => {
          // }
          _ => (),
        },

        Event::DeviceEvent { event, .. } => match event {
          DeviceEvent::Key(KeyboardInput {
            virtual_keycode: Some(keycode),
            state,
            ..
          }) => match (keycode, state) {
            // (VirtualKeyCode::Escape, ElementState::Released) => *control_flow = ControlFlow::Exit,
            _ => {
              unsafe{
                (*GLOBALS.w_vulkan).w_input.handle_key_press(
                    keycode, state
                );
              }
            // _ => (),
            },
          },
          _ => (),
        },

        // Wait fence -> wait RT semaphore ->                     -> Reset Fence -> Render with fence
        Event::MainEventsCleared => unsafe {
          // ! ---------- Render Loop ---------- //
          // ! WAIT GPU TO BE DONE WITH OTHER FRAME
          let (rt, signal_semaphore, wait_semaphore, image_index ) = unsafe {

            // -- update time -- //
            {
              let shader_man = & (*GLOBALS.w_vulkan).w_shader_man;
            
            }

            // -- RELOAD SHADERS -- //
            {
              let shader_man = & (*GLOBALS.w_vulkan).w_shader_man;
              if *shader_man.shader_was_modified.lock().unwrap() {
                shader_man.chan_sender_start_shader_comp.send(());
                shader_man.chan_receiver_end_shader_comp.recv().expect("Error: timed out.");
                println!("-- SHADER RELOAD END --")
              } 
            }
            

            // -- UPDATE CAM -- //
            {
              {
                macro_rules! write_float {
                    ($mem_ptr: expr, $value: expr ) => {unsafe{
                      *($mem_ptr) = $value;
                      $mem_ptr = $mem_ptr.add(1);
                    }};
                        
                }

                macro_rules! write_mat4x4 {
                    ($mem_ptr: expr, $value: expr ) => {unsafe{
                      write_float!($mem_ptr, $value[0]); 
                      write_float!($mem_ptr, $value[1]); 
                      write_float!($mem_ptr, $value[2]); 
                      write_float!($mem_ptr, $value[3]); 
                      write_float!($mem_ptr, $value[4]); 
                      write_float!($mem_ptr, $value[5]); 
                      write_float!($mem_ptr, $value[6]); 
                      write_float!($mem_ptr, $value[7]); 
                      write_float!($mem_ptr, $value[8]); 
                      write_float!($mem_ptr, $value[9]); 
                      write_float!($mem_ptr, $value[10]); 
                      write_float!($mem_ptr, $value[11]); 
                      write_float!($mem_ptr, $value[12]); 
                      write_float!($mem_ptr, $value[13]); 
                      write_float!($mem_ptr, $value[14]); 
                      write_float!($mem_ptr, $value[15]); 
                    }};
                        
                }
                let ubo = (*GLOBALS.w_vulkan).shared_ubo.get_mut();
                let cam = &mut (*GLOBALS.w_vulkan).w_cam;
                cam.refresh();

                let mut mem_ptr = ubo.buff.mapped_mems[ubo.buff.pong_idx as usize] as *mut f32;
                unsafe {
                  write_mat4x4!(mem_ptr, cam.view_mat); 
                  write_mat4x4!(mem_ptr, cam.proj_mat); 
                }
              }
            
            }

            let WV = &mut *GLOBALS.w_vulkan;

            WV
              .w_device
              .device
              .wait_for_fences(
                &[WV.w_swapchain.in_flight_fences[WV.frame]],
                true,
                u64::MAX,
              )
              .unwrap();


            WV.w_device.command_pools[WV.w_device.pong_idx].reset(&WV.w_device.device);
            

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
            let rt = (WV
              .w_swapchain
              .default_render_targets
              .index_mut(image_index as usize) as *mut WRenderTarget)
              .as_mut()
              .unwrap();
            let signal_semaphore = WV.w_swapchain.render_finished_semaphores[WV.frame as usize];
            let wait_semaphore = WV.w_swapchain.image_available_semaphores[WV.frame as usize];

            (rt, signal_semaphore, wait_semaphore, image_index )
          };

          render(&mut sketch, rt, wait_semaphore, signal_semaphore);

          {
            // ! Present

            let WV = &mut *GLOBALS.w_vulkan;

            let sem = [signal_semaphore];

            let swapchains = vec![WV.w_swapchain.swapchain];
            let image_indices = vec![image_index];
            let present_info = {
              vk::PresentInfoKHR::builder()
                .wait_semaphores(&sem)
                .swapchains(&swapchains)
                .image_indices(&image_indices)
            };
            WV
              .w_swapchain
              .swapchain_loader
              .queue_present(WV.w_device.queue, &present_info)
              .unwrap();

            WV.frame = (WV.frame + 1) % FRAMES_IN_FLIGHT;
            WV.w_device.pong_idx = 1 - WV.w_device.pong_idx;
          }
        },
        _ => (),
      }
    });
  }

  pub fn init_window(event_loop: &EventLoop<()>) -> Window {
    let window = WindowBuilder::new()
      .with_resizable(false)
      .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
      .build(event_loop)
      .unwrap();

    return window;
  }
  pub fn new(window: &'a Window) -> WVulkan {
    let (mut w_device, w_swapchain) = WDevice::init_device_and_swapchain(window);
    let default_render_targets = unsafe { MaybeUninit::zeroed().assume_init() };

    let mut w_tech_lead = WTechLead::new(&mut w_device);

    // let shared_binding_image_array = w_tech_lead.shared_binding_image_array.clone();

    let mut w_grouper = WGrouper {
      bind_groups_arena: Arena::new(),
    };

    // ubo_shared.buff.mapped_array
    // let shared_images = vec![].reserve( 32);

    let shared_ubo = w_tech_lead.new_uniform_buffer(&mut w_device, 32 * 10).0;

    let mut shared_bind_group = w_grouper.new_group(&mut w_device);

    shared_bind_group.1.set_binding_ubo(0, shared_ubo.idx);

    // shared_bind_group.1.image_array_binding = Some( shared_binding_image_array);
    shared_bind_group.1.image_array_binding =
      Some(w_ptr_to_mut_ref!(GLOBALS.shared_binding_images_array));

    // shared_bind_group.1.set_binding(2,WBindingImageArray(shared_binding_image_array));

    let mut w_cam = WCamera::new(w_swapchain.width, w_swapchain.height);
    w_cam.refresh();

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
      w_shader_man: WShaderMan::new(),
      w_cam,
      w_input: WInput::new(),
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