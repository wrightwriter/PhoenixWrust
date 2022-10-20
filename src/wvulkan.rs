#[allow(non_snake_case)]
use ash::vk::{self};

use generational_arena::Arena;
use gpu_alloc::GpuAllocator;
use gpu_alloc_ash::AshMemoryDevice;
use nalgebra_glm::{vec2, Vec2};

// use renderdoc::{RenderDoc, V120, V141};
use crate::{
  abs::{wcam::WCamera, wcomputepass::WComputePass, wthing::WThing},
  res::{
    wimage::{WImageCreateInfo, WImage},
    wmodel::WModel,
    wrendertarget::{WRenderTarget, WRenderTargetCreateInfo},
    wshader::WProgram, wwritablebuffertrait::WWritableBufferTrait,
  },
  sys::{
    warenaitems::{WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxRt, WAIdxUbo, WArenaItem},
    wbarr::{VStage, WBarr},
    wcommandencoder::WCommandEncoder,
    wdevice::{WDevice, GLOBALS},
    winput::WInput,
    wmanagers::{WGrouper, WShaderMan, WTechLead},
    wswapchain::WSwapchain,
    wtime::WTime,
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

use std::{
  borrow::{BorrowMut, Borrow},
  cell::Cell,
  mem::{ManuallyDrop, MaybeUninit},
  ops::{Div, IndexMut},
  sync::{Arc, Mutex},
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
  pub w_tl: WTechLead,
  pub w_grouper: WGrouper,
  pub w_shader_man: WShaderMan,
  pub w_cam: WCamera,
  pub w_input: WInput,
  pub w_time: WTime,

  // w_render_doc: RenderDoc<V120>,
  pub shared_ubo: WAIdxUbo,
  pub shared_bind_group: WAIdxBindGroup,
  pub frame: usize,
  pub width: u32,
  pub height: u32,
}

pub struct Sketch {
  pub test_img: WAIdxImage,
  pub test_file_img: WAIdxImage,
  pub command_encoder: WCommandEncoder,
  pub rt_gbuffer: WAIdxRt,
  pub rt_composite: WAIdxRt,
  pub test_buff: WAIdxBuffer,
  pub comp_pass: WComputePass,
  pub thing: WThing,
  pub thing_mesh: WThing,
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

      let test_model = WModel::new("test.gltf".to_string(), WV);

      // !! ---------- RT ---------- //
      let rt_create_info = WRenderTargetCreateInfo { 
        resx: WV.w_cam.width,
        resy: WV.w_cam.height,
        attachments: vec![
          WImageCreateInfo{ ..wdef!() },
          WImageCreateInfo{ ..wdef!() },
        ],
        ..wdef!() };
      let rt_gbuffer = WV.w_tl.new_render_target(&mut WV.w_device, rt_create_info.clone()).0;

      let rt_composite = WV.w_tl.new_render_target(&mut WV.w_device, rt_create_info.clone()).0;


      let mut test_img = WV
        .w_tl
        .new_image(&mut WV.w_device, WImageCreateInfo { ..wdef!() })
        .0;

      let mut test_file_img = WV
        .w_tl
        .new_image(
          &mut WV.w_device,
          WImageCreateInfo {
            file_name: Some("test.png".to_string()),
            ..wdef!()
          },
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
      
      // WV.w_device.device.cmd_set_depth_compare_op(command_buffer, depth_compare_op)

      let mut test_buff = WV
        .w_tl
        .new_buffer(
          &mut WV.w_device,
          vk::BufferUsageFlags::STORAGE_BUFFER,
          1000,
          false,
        ).0;

      // !! ---------- SHADER ---------- //
      let prog_mesh = WV.w_shader_man.new_render_program(
        &mut WV.w_device,
        "mesh.vert",
        "mesh.frag",
      );

      let prog_render = WV.w_shader_man.new_render_program(
        &mut WV.w_device,
        "triangle.vert",
        "triangle.frag",
      );

      let prog_compute = WV
        .w_shader_man
        .new_compute_program(&mut WV.w_device, "compute.comp".to_string());

      // !! ---------- COMP ---------- //
      let mut comp_pass = WComputePass::new( WV, prog_compute, );

      // let mut arr = WV.w_tech_lead.ubo_arena[thing.ubo.idx]
      //   .borrow_mut()
      //   .buff
      //   .mapped_array
      //   .as_ptr();

      // !! ---------- Thing ---------- //

      let mut thing = WThing::new(
        WV,
        prog_render,
      );

      let mut thing_mesh = WThing::new(
        WV,
        prog_mesh,
      );
      thing_mesh.model = Some(test_model);

      // !! ---------- POSTFX ---------- //

      // !! ---------- END INIT ---------- //
      let mut sketch = Sketch {
        test_img,
        test_buff,
        comp_pass,
        thing,
        rt_gbuffer,
        command_encoder,
        thing_mesh,
        test_file_img,
        rt_composite,
        // test_model,
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
        // let cwd = std::env::current_dir().unwrap().to_str().unwrap().to_string();



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

        // {
          // rt.begin_pass(&mut w.w_device);

        //   s.thing
        //     .draw(&mut w.w_device, &mut w.w_grouper, &mut w.w_tl, None, &rt.cmd_buf);
        //   s.thing_mesh
        //     .draw(&mut w.w_device, &mut w.w_grouper, &mut w.w_tl, None,&rt.cmd_buf);

          // rt.end_pass(&w.w_device);
        // }

        s.command_encoder
          .add_and_run_barr(&mut w.w_device, &WBarr::new_general_barr());


        // Render

        {
          let cmd_buf = {
            s.rt_gbuffer.get_mut().begin_pass(&mut w.w_device)
          };

          s.thing
            .draw(w, Some(s.rt_gbuffer), &cmd_buf);

          // s.thing
          //   .draw(&mut w.w_device, &mut w.w_grouper, &mut w.w_tl, None, &rt.cmd_buf);
          s.thing_mesh
            .draw(w, Some(s.rt_gbuffer),&cmd_buf);

          {
            s.rt_gbuffer.get_mut().end_pass(&w.w_device);
            s.command_encoder.push_buff(cmd_buf);
          }
        }

        // Post 
        {
          let cmd_buf = {
            s.rt_gbuffer.get_mut().begin_pass(&mut w.w_device)
          };

          s.thing
            .draw(w, Some(s.rt_gbuffer), &cmd_buf);

          // s.thing
          //   .draw(&mut w.w_device, &mut w.w_grouper, &mut w.w_tl, None, &rt.cmd_buf);
          s.thing_mesh
            .draw(w, Some(s.rt_gbuffer),&cmd_buf);

          {
            s.rt_gbuffer.get_mut().end_pass(&w.w_device);
            s.command_encoder.push_buff(cmd_buf);
          }
        }

        s.command_encoder
          .add_and_run_barr(&mut w.w_device, &WBarr::new_general_barr());

        {
          s.comp_pass.dispatch(w, 1, 1, 1);
          s.command_encoder.push_buff(s.comp_pass.command_buffer);
        }

        s.command_encoder
          .add_and_run_barr(&mut w.w_device, &WBarr::new_general_barr());
        
        // s.command_encoder.
        // w.w_device.device.cmd_copy_image2(command_buffer, copy_image_info)
        

        // BLIT
        {
          let cmd_buff = s.command_encoder.get_and_begin_buff(&mut w.w_device);
          let test_rt = s.rt_gbuffer.get_mut();
          let src_img = test_rt.image_indices[test_rt.pong_idx as usize][0].get_mut();
          let dst_img = &rt.images[test_rt.pong_idx as usize];
          
          // let mut barr_src = WBarr::new_image_barr();
          // barr_src.old_layout(src_img.descriptor_image_info.image_layout);
          // barr_src.new_layout(src_img.descriptor_image_info.image_layout);

          let barr_dst_in = WBarr::new_image_barr()
            .old_layout(dst_img.descriptor_image_info.image_layout)
            // .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(dst_img.handle)
            .src_access(vk::AccessFlags2::MEMORY_READ)
            .dst_access(vk::AccessFlags2::TRANSFER_READ)
            .src_stage(vk::PipelineStageFlags2::TRANSFER)
            .dst_stage(vk::PipelineStageFlags2::TRANSFER)
            .run_on_cmd_buff(&w.w_device, cmd_buff);

          // let barr_src_in = WBarr::new_image_barr()
          //   .old_layout(src_img.descriptor_image_info.image_layout)
          //   .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
          //   .image(src_img.handle)
          //   .src_access(vk::AccessFlags2::MEMORY_READ)
          //   .dst_access(vk::AccessFlags2::TRANSFER_WRITE)
          //   .src_stage(vk::PipelineStageFlags2::TRANSFER)
          //   .dst_stage(vk::PipelineStageFlags2::TRANSFER)
          //   .run_on_cmd_buff(&w.w_device, cmd_buff);

          
          if true{
            let blank_sz = vk::Offset3D::builder().build();
            let blit_sz = vk::Offset3D::builder()
              .x(src_img.resx as i32)
              .y(src_img.resy as i32)
              .z(1)
              .build();
            
            let subresource_layers = vk::ImageSubresourceLayers::builder()
              .aspect_mask(vk::ImageAspectFlags::COLOR)
              .layer_count(1)
              .build();
            
            let region = vk::ImageBlit2::builder()
              .src_offsets([blank_sz,blit_sz])
              .dst_offsets([blank_sz,blit_sz])
              .src_subresource(subresource_layers)
              .dst_subresource(subresource_layers)
              .build();

            let blit_image_info = vk::BlitImageInfo2::builder()
              .src_image(src_img.handle)
              .dst_image(dst_img.handle)
              .src_image_layout(src_img.descriptor_image_info.image_layout)
              .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
              .regions(&[ region ])
              .filter(vk::Filter::NEAREST)
              .build()
            ;
            w.w_device.device.cmd_blit_image2(cmd_buff, &blit_image_info);

          }

          let barr_dst_out = WBarr::new_image_barr()
            .image(dst_img.handle)

            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_access(vk::AccessFlags2::TRANSFER_READ)
            .dst_access(vk::AccessFlags2::MEMORY_READ)

            .src_stage(vk::PipelineStageFlags2::TRANSFER)
            .dst_stage(vk::PipelineStageFlags2::TRANSFER)
            .run_on_cmd_buff(&w.w_device, cmd_buff);

          // let barr_src_out = WBarr::new_image_barr()
          //   .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
          //   .new_layout(src_img.descriptor_image_info.image_layout)
          //   .image(src_img.handle)
          //   .src_access(vk::AccessFlags2::MEMORY_READ)
          //   .dst_access(vk::AccessFlags2::TRANSFER_WRITE)
          //   .src_stage(vk::PipelineStageFlags2::TRANSFER)
          //   .dst_stage(vk::PipelineStageFlags2::TRANSFER)
          //   .run_on_cmd_buff(&w.w_device, cmd_buff);


          s.command_encoder.end_and_push_buff(&mut w.w_device, cmd_buff);
        }

        s.command_encoder
          .add_and_run_barr(&mut w.w_device, &WBarr::new_general_barr());

        // !! ---------- SUBMIT ---------- //


        s.command_encoder.submit(&mut w.w_device);

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
            // .command_buffer_infos(&cmd_buffs)
            .command_buffer_infos(&[])
            .signal_semaphore_infos(&signal_semaphore_submit_infos)
            .build();
          let in_flight_fence = w.w_swapchain.in_flight_fences[w.frame as usize];
          w.w_device.device.reset_fences(&[in_flight_fence]).unwrap();
          w.w_device
            .device
            .queue_submit2(w.w_device.queue, &[submit_info], in_flight_fence)
            .unwrap();
        }

        // !! ---------- END ---------- //
        w.w_input.refresh_keys();
        w.w_input.mouse_state.delta_pos = vec2(0.0, 0.0);
        w.w_input.mouse_state.delta_pos_normalized = vec2(0.0, 0.0);
      };
    }

    event_loop.run_return(move |event, _, control_flow| {
      match event {
        Event::NewEvents(StartCause::Init) => {
          *control_flow = ControlFlow::Poll;
        }
        #[allow(deprecated)]
        Event::WindowEvent { event, .. } => match event {
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
              (*GLOBALS.w_vulkan)
                .w_input
                .handle_mouse_press(button, state);
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
        },

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


        // Wait fence -> wait RT semaphore ->                     -> Reset Fence -> Render with fence
        Event::MainEventsCleared => unsafe {
          // ! ---------- Render Loop ---------- //
          // ! WAIT GPU TO BE DONE WITH OTHER FRAME
          let (rt, signal_semaphore, wait_semaphore, image_index) = unsafe {
            // -- update time -- //
            {
              // let shader_man = & (*GLOBALS.w_vulkan).w_shader_man;
              let time = &mut (*GLOBALS.w_vulkan).w_time;
              time.tick();
            }

            // -- RELOAD SHADERS -- //
            {
              let shader_man = &(*GLOBALS.w_vulkan).w_shader_man;
              if *shader_man.shader_was_modified.lock().unwrap() {
                shader_man.chan_sender_start_shader_comp.send(());
                shader_man
                  .chan_receiver_end_shader_comp
                  .recv()
                  .expect("Error: timed out.");
                println!("-- SHADER RELOAD END --")
              }
            }

            // -- UPDATE CAM -- //
            {
              let ubo = (*GLOBALS.w_vulkan).shared_ubo.get_mut();
              let cam = &mut (*GLOBALS.w_vulkan).w_cam;
              let time = &mut (*GLOBALS.w_vulkan).w_time;

              cam.update_movement(
                (*GLOBALS.w_vulkan).w_input.mouse_state,
                &(*GLOBALS.w_vulkan).w_input,
                time.dt_f32,
              );

              cam.update_matrices();

              // let mut mem_ptr = ubo.buff.mapped_mems[ubo.buff.pong_idx as usize] as *mut f32;
              let ubo_buff = &mut ubo.buff;
              ubo_buff.reset_ptr();

              // vec3
              ubo_buff.write_vec3((*GLOBALS.w_vulkan).w_cam.eye_pos);
              ubo_buff.write_float(0.0f32); // padding

              // vec2
              ubo_buff.write_float((*GLOBALS.w_vulkan).w_cam.width as f32);
              ubo_buff.write_float((*GLOBALS.w_vulkan).w_cam.height as f32);

              ubo_buff.write_vec2((*GLOBALS.w_vulkan).w_input.mouse_state.pos_normalized);
              ubo_buff.write_vec2((*GLOBALS.w_vulkan).w_input.mouse_state.delta_pos_normalized);
              ubo_buff.write_float(0.0f32); // padding
              ubo_buff.write_float(0.0f32); // padding

              // float
              ubo_buff.write_float((*GLOBALS.w_vulkan).w_time.t_f32);
              ubo_buff.write_float((*GLOBALS.w_vulkan).w_time.dt_f32);
              ubo_buff.write_uint((*GLOBALS.w_vulkan).w_time.frame as u32);

              ubo_buff.write_float(if (*GLOBALS.w_vulkan).w_input.mouse_state.rmb_down {
                1.0
              } else {
                0.0
              });
              ubo_buff.write_float(if (*GLOBALS.w_vulkan).w_input.mouse_state.lmb_down {
                1.0
              } else {
                0.0
              });

              ubo_buff.write_float((*GLOBALS.w_vulkan).w_cam.near as f32);
              ubo_buff.write_float((*GLOBALS.w_vulkan).w_cam.far as f32);

              ubo_buff.write_float(0.0f32); // padding

              // mat4
              ubo_buff.write_mat4x4(cam.view_mat);
              ubo_buff.write_mat4x4(cam.proj_mat);
              ubo_buff.write_mat4x4(cam.view_proj_mat);
              ubo_buff.write_mat4x4(cam.inv_view_mat);
            }

            let WV = &mut *GLOBALS.w_vulkan;

            WV.w_device
              .device
              .wait_for_fences(&[WV.w_swapchain.in_flight_fences[WV.frame]], true, u64::MAX)
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

            (rt, signal_semaphore, wait_semaphore, image_index)
          };

          rt.images[0].descriptor_image_info.image_layout = vk::ImageLayout::UNDEFINED;

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
            WV.w_swapchain
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


    // why 
    unsafe{
      shared_bind_group.1.image_array_binding =
        Some(GLOBALS.shared_binding_images_array);

      shared_bind_group.1.buffer_array_binding =
        Some(GLOBALS.shared_binding_buffers_array);
    }


    // shared_bind_group.1.set_binding(2,WBindingImageArray(shared_binding_image_array));

    let mut w_cam = WCamera::new(w_swapchain.width, w_swapchain.height);
    w_cam.update_matrices();

    let wv = Self {
      width: w_swapchain.width,
      height: w_swapchain.height,
      w_tl: w_tech_lead,
      w_swapchain,
      shared_ubo,
      shared_bind_group: shared_bind_group.0,
      frame: 0,
      w_device,
      w_grouper,
      w_shader_man: WShaderMan::new(),
      w_cam,
      w_input: WInput::new(),
      w_time: WTime::new(),
      // w_render_doc,
    };


    wv
  }

  fn draw_frame(&mut self) {}

  fn destroy(&self) {
    // unsafe{ self.instance.destroy_instance(None); }
  }
}
