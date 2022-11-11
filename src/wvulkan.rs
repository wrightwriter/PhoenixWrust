use ash::vk::BufferUsageFlags;
#[allow(non_snake_case)]
use ash::vk::{self};

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
    img::wrendertarget::{WRenderTarget, WRenderTargetInfo},
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
    wmanagers::{WGrouper, WTechLead},
    wrecorder::WRecorder,
    wshaderman::WShaderMan,
    wswapchain::WSwapchain,
    wtime::WTime,
  },
  wdef,
};

// use smallvec::SmallVec;
use winit::{dpi::LogicalSize, platform::run_return::EventLoopExtRunReturn};

use winit::{
  event::{DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  window::WindowBuilder,
};

use std::{borrow::BorrowMut, cell::Cell, ops::IndexMut, time::{Instant, Duration}};

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
  pub w_recorder: WRecorder,
  pub w_cam: WCamera,
  pub w_input: WInput,
  pub w_gui: WGUI,
  pub w_time: WTime,

  // w_render_doc: RenderDoc<V120>,
  pub shared_ubo: WAIdxUbo,
  pub shared_bind_group: WAIdxBindGroup,
  pub frame: usize,
  pub width: u32,
  pub height: u32,
}

pub struct Sketch {
  pub encoder: WCommandEncoder,

  pub test_img: WAIdxImage,
  pub test_file_img: WAIdxImage,

  pub flame_img: WAIdxImage,

  // pub test_video: WVideo,
  pub rt_gbuffer: WAIdxRt,
  pub rt_composite: WAIdxRt,

  pub composite_pass: WFxPass,

  pub fx_composer: WFxComposer,

  pub chromab_pass: WFxPass,
  pub kernel_pass: WKernelPass,
  pub gamma_pass: WFxPass,
  pub fxaa_pass: WFxPass,

  pub test_buff: WAIdxBuffer,

  pub flame_pass: WComputePass,

  pub thing: WThing,
  pub thing_mesh: WThing,

  pub thing_path: WThingPath,

  pub font: WFont,
  pub thing_text: WThingText,
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

      // !! ---------- Video ---------- //
      // let test_video = WVideo::new(WV);

      // !! ---------- SHADER ---------- //
      let prog_mesh = WV.w_shader_man.new_render_program(&mut WV.w_device, "mesh.vert", "mesh.frag");

      let prog_render = WV
        .w_shader_man
        .new_render_program(&mut WV.w_device, "triangle.vert", "triangle.frag");

      let prog_flame = WV.w_shader_man.new_compute_program(&mut WV.w_device, "flame.comp");

      // !! ---------- COMP ---------- //
      let mut flame_pass = WComputePass::new(WV, prog_flame);

      let prog_composite = WV
        .w_shader_man
        .new_render_program(&mut WV.w_device, "fullscreenQuad.vert", "composite.frag");

      let prog_path = WV.w_shader_man.new_render_program(&mut WV.w_device, "path.vert", "path.frag");

      let prog_text = WV.w_shader_man.new_render_program(&mut WV.w_device, "text.vert", "text.frag");

      // !! ---------- Lyon ---------- //

      let mut thing_path = WThingPath::new(WV, prog_path);
      thing_path.path();

      // !! ---------- Font ---------- //

      let font = WFont::new(WV, "ferritecore.otf");
      let thing_text = WThingText::new(WV, prog_text, font.clone());

      let fx_composer = WFxComposer::new(WV);

      // !! ---------- Models ---------- //

      // !! ---------- RT ---------- //
      let mut rt_create_info = WRenderTargetInfo {
        resx: WV.w_cam.width,
        resy: WV.w_cam.height,
        attachments: vec![
          WImageInfo { ..wdef!() },
          WImageInfo { ..wdef!() },
          WImageInfo { ..wdef!() },
          WImageInfo { ..wdef!() },
        ],
        ..wdef!()
      };
      let rt_gbuffer = WV.w_tl.new_render_target(&mut WV.w_device, rt_create_info.clone()).0;

      rt_create_info.has_depth = false;
      rt_create_info.attachments = WRenderTargetInfo::default().attachments;
      rt_create_info.pongable = true;

      let rt_composite = WV.w_tl.new_render_target(&mut WV.w_device, rt_create_info.clone()).0;

      let mut test_img = WV.w_tl.new_image(&mut WV.w_device, WImageInfo { ..wdef!() }).0;

      let mut flame_img = WV
        .w_tl
        .new_image(
          &mut WV.w_device,
          WImageInfo {
            resx: WV.w_cam.width,
            resy: WV.w_cam.height,
            format: vk::Format::R32_SFLOAT,
            ..wdef!()
          },
        )
        .0;

      let mut test_file_img = WV
        .w_tl
        .new_image(
          &mut WV.w_device,
          WImageInfo {
            file_path: Some("test.png".to_string()),
            ..wdef!()
          },
        )
        .0;

      // WV.w_device.device.cmd_set_depth_compare_op(command_buffer, depth_compare_op)

      let mut test_buff = WV
        .w_tl
        .new_buffer(&mut WV.w_device, vk::BufferUsageFlags::STORAGE_BUFFER, 1000, false)
        .0;

      // !! ---------- Thing ---------- //

      let mut thing = WThing::new(WV, prog_render);

      // !! ---------- POSTFX ---------- //

      let composite_pass = WFxPass::new(WV, false, prog_composite);
      let chromab_pass = WFxPass::new_from_frag_shader(WV, false, "FX/chromab.frag");
      let gamma_pass = WFxPass::new_from_frag_shader(WV, false, "FX/gamma.frag");

      let mut kernel_pass = WKernelPass::new(WV, false);
      kernel_pass.get_uniforms_container().exposed = true;

      let fxaa_pass = WFxPass::new_from_frag_shader(WV, false, "FX/fxaa.frag");

      // let test_model = WModel::new( "battle\\scene.gltf", WV,);
      // let test_model = WModel::new( "gltf_test_models\\DamagedHelmet\\glTF\\DamagedHelmet.gltf", WV,);
      let test_model = WModel::new("gltf_test_models\\Sponza\\glTF\\Sponza.gltf", WV);

      // let test_model = WModel::new("test.gltf", WV);
      let mut thing_mesh = WThing::new(WV, prog_mesh);
      thing_mesh.model = Some(test_model);

      {
        WV.w_grouper.bind_groups_arena[WV.shared_bind_group.idx].borrow_mut().rebuild_all(
          &WV.w_device.device,
          &WV.w_device.descriptor_pool,
          &mut WV.w_tl,
        );
      }

      // !! ---------- END INIT ---------- //
      let mut sketch = Sketch {
        test_img,
        test_buff,
        // comp_pass,
        thing,
        rt_gbuffer,
        encoder: command_encoder,
        thing_mesh,
        test_file_img,
        rt_composite,
        composite_pass,
        fx_composer,
        chromab_pass,
        fxaa_pass,
        gamma_pass,
        kernel_pass,
        font,
        thing_path,
        thing_text,
        flame_pass,
        flame_img,
        // test_video,
        // test_video,
        // test_model,
      };

      // big brain 🧠🧠
      WV.w_device.device.queue_wait_idle(WV.w_device.queue);
      sketch
    };

    #[profiling::function]
    fn render(
      s: &mut Sketch,
      rt: &mut WRenderTarget,
      imgui_cmd_buff: vk::CommandBuffer,
      wait_semaphore: vk::Semaphore,
      signal_semaphore: vk::Semaphore,
    ) {
      unsafe {
        let w = &mut *GLOBALS.w_vulkan;
        s.encoder.reset(&mut w.w_device);

        w.w_tl.pong_all();

        // unsafe{
        //   let cmd_buf = s.test_video.update_frame(w);
        //   s.encoder.push_buf(cmd_buf);
        // }

        s.encoder.push_barr(w, &WBarr::render());

        // !! Render
        if false {
          let cmd_buf = { s.rt_gbuffer.get_mut().begin_pass(&mut w.w_device) };

          // s.thing.draw(w, Some(s.rt_gbuffer), &cmd_buf);
          // s.thing_path.draw(w, Some(s.rt_gbuffer), &cmd_buf);

          s.thing_mesh.push_constants.reset();
          s.thing_mesh.push_constants.add(0f32);
          s.thing_mesh.draw(w, Some(s.rt_gbuffer), &cmd_buf);

          // s.thing_text.draw(w, Some(s.rt_gbuffer), &cmd_buf);

          {
            s.rt_gbuffer.get_mut().end_pass(&w.w_device);
            s.encoder.push_buf(cmd_buf);
          }
        }

        // clear flame
        {
          let cmd_buf = s.encoder.get_and_begin_buff(&mut w.w_device);
          w.w_device.device.cmd_clear_color_image(
            cmd_buf,
            s.flame_img.get().handle,
            vk::ImageLayout::GENERAL,
            &vk::ClearColorValue::default(),
            &[vk::ImageSubresourceRange::builder()
              .aspect_mask(vk::ImageAspectFlags::COLOR)
              .base_array_layer(0)
              .layer_count(1)
              .level_count(1)
              .build()],
          );
          s.encoder.end_and_push_buff(&mut w.w_device, cmd_buf);
        }

        s.encoder.push_barr(
          w,
          &WBarr::general()
            .src_stage(vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS)
            .dst_stage(vk::PipelineStageFlags2::COMPUTE_SHADER),
        );

        // !! FLAME
        {
          s.flame_pass.push_constants.reset();
          s.flame_pass.push_constants.add_many(&[
            s.rt_gbuffer.get().image_depth.unwrap(),
            s.flame_img,
            // s.rt_composite.get().image_at(0),
            // s.rt_composite.get().image_at(0),
            // s.test_video.gpu_image,
          ]);

          s.encoder.push_buf(s.flame_pass.dispatch(w, 10, 100, 1));
        }

        // s.encoder
        //   .push_barr(w, &WBarr::general()
        //     .src_stage(vk::PipelineStageFlags2::FRAGMENT_SHADER)
        //     .dst_stage(vk::PipelineStageFlags2::COMPUTE_SHADER)
        //   );

        s.encoder.push_barr(w, &WBarr::comp_to_frag());

        // s.encoder
        //   .push_barr(w, &WBarr::render());

        // !! COMPOSITE
        s.composite_pass.push_constants.reset();
        s.composite_pass.push_constants.add_many(&[
          s.rt_gbuffer.get().image_at(0),
          s.rt_gbuffer.get().image_at(1),
          s.rt_gbuffer.get().image_at(2),
          s.rt_gbuffer.get().image_depth.unwrap(),
          s.rt_composite.get().back_image_at(0),
          s.flame_img,
          // s.test_video.gpu_image,
        ]);

        s.encoder.push_buf(s.composite_pass.run_on_external_rt(s.rt_composite, w));

        // s.encoder
        //   .push_barr(w, &WBarr::render());

        // !! POST

        s.fx_composer.begin(s.rt_composite);
        // s.fx_composer.run(w, &mut s.fxaa_pass);
        // s.fx_composer.run(w, &mut s.kernel_pass);
        // s.fx_composer.run(w, &mut s.chromab_pass);
        s.fx_composer.run(w, &mut s.gamma_pass);

        s.encoder.push_bufs(&s.fx_composer.cmd_bufs);

        // s.encoder.push_buf();

        // {
        //   s.comp_pass.dispatch(w, 1, 1, 1);
        //   s.command_encoder.push_buff(s.comp_pass.command_buffer);
        // }
        s.encoder.push_barr(w, &WBarr::general());

        // blit
        WDevice::blit_image_to_swapchain(w, &mut s.encoder, s.fx_composer.get_front_img(), &rt);

        s.encoder.push_barr(w, &WBarr::general());

        s.encoder.push_buf(imgui_cmd_buff);

        // !! ---------- SUBMIT ---------- //

        s.encoder.submit_to_queue(&mut w.w_device);

        // *s.test_video.speed.lock().unwrap() = 0.04;
        // if w.w_time.frame % 500 == 0{
        //   // s.test_video.seek(0.0);
        //   *s.test_video.speed.lock().unwrap() = 10.;
        //   println!("bleep");
        // } else if w.w_time.frame % 500 == 250{
        //   // s.test_video.seek(0.0);
        //   *s.test_video.speed.lock().unwrap() = 0.1;
        //   println!("bloop");
        // }

        // !! ---------- END ---------- //
        w.w_input.refresh_keys();
        w.w_input.mouse_state.delta_pos = vec2(0.0, 0.0);
        w.w_input.mouse_state.delta_pos_normalized = vec2(0.0, 0.0);
      };
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
        // New frame
        Event::NewEvents(_) => {
          let now = Instant::now();
          unsafe {
            let mut dt = (*GLOBALS.w_vulkan).w_time.dt;
            if (*GLOBALS.w_vulkan).w_recorder.recording{
              dt = Duration::from_secs_f64(0.1);
            }

            let mut imgui = unsafe { (*GLOBALS.imgui).borrow_mut() };
            imgui.io_mut().update_delta_time(dt);
          }
        }

        // Wait fence -> wait RT semaphore ->                     -> Reset Fence -> Render with fence
        Event::MainEventsCleared => unsafe {
          let (rt, signal_semaphore, wait_semaphore, image_index, imgui_cmd_buf) = unsafe {
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
            }

            // -- update time -- //
            {
              let rec = &mut (*GLOBALS.w_vulkan).w_recorder;
              let time = &mut (*GLOBALS.w_vulkan).w_time;
              if rec.recording {
                let dt = (1.0/(rec.frame_rate as f64)) as f64;
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

            // -- UPDATE CAM -- //
            {
              let ubo = (*GLOBALS.w_vulkan).shared_ubo.get_mut();
              let cam = &mut (*GLOBALS.w_vulkan).w_cam;
              let time = &mut (*GLOBALS.w_vulkan).w_time;

              cam.update_movement((*GLOBALS.w_vulkan).w_input.mouse_state, &(*GLOBALS.w_vulkan).w_input, time.dt_f32);

              cam.update_matrices();

              // let mut mem_ptr = ubo.buff.mapped_mems[ubo.buff.pong_idx as usize] as *mut f32;
              let ubo_buff = &mut ubo.buff;
              ubo_buff.reset_ptr();

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

            let mut im_gui = (*GLOBALS.imgui).borrow_mut();

            // UI

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

            if !WV.w_recorder.recording{
              WV.w_gui
                .draw_internal(&mut WV.w_device, &mut WV.w_time, &mut im_ui, &mut WV.w_shader_man, &mut WV.w_cam, &mut WV.w_recorder );
            }

            WV.w_device.platform.prepare_render(&im_ui, &window);

            let draw_data = im_ui.render();

            let imgui_cmd_buf = rt.begin_pass(&mut WV.w_device);
            WV.w_device.imgui_renderer.cmd_draw(imgui_cmd_buf, draw_data).unwrap();
            rt.end_pass(&mut WV.w_device);

            (rt, signal_semaphore, wait_semaphore, image_index, imgui_cmd_buf)
          };

          rt.images[0].descriptor_image_info.image_layout = vk::ImageLayout::UNDEFINED;

          render(&mut sketch, rt, imgui_cmd_buf, wait_semaphore, signal_semaphore);

          // ! Reset curr fence and submit
          unsafe {
            let mut cmd_buffs = [vk::CommandBufferSubmitInfo::builder().command_buffer(rt.cmd_buf).build()];
            let WV = &mut *GLOBALS.w_vulkan;
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
            let WV = &mut *GLOBALS.w_vulkan;
            WV.w_recorder.try_recording(&rt.images[0], &mut WV.w_device);
          }

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
      .with_title("👩‍💻")
      .build(event_loop)
      .unwrap();

    return window;
  }
  pub fn new(window: &'a Window) -> WVulkan {
    let (mut w_device, w_swapchain) = WDevice::init_device_and_swapchain(window);

    let mut w_tl = WTechLead::new(&mut w_device);

    let mut w_grouper = WGrouper {
      bind_groups_arena: Arena::new(),
    };

    let shared_ubo = w_tl.new_uniform_buffer(&mut w_device, 32 * 20).0;

    let mut shared_bind_group = w_grouper.new_group(&mut w_device).0;

    unsafe {
      {
        let sbg = &mut w_grouper.bind_groups_arena[shared_bind_group.idx];

        sbg.set_binding_ubo(0, shared_ubo.idx);
        sbg.image_array_binding = Some(GLOBALS.shared_binding_images_array);

        sbg.buffer_array_binding = Some(GLOBALS.shared_binding_buffers_array);
      }

      w_grouper.bind_groups_arena[shared_bind_group.idx]
        .borrow_mut()
        .rebuild_all(&w_device.device, &w_device.descriptor_pool, &mut w_tl);
    }

    let mut w_cam = WCamera::new(w_swapchain.width, w_swapchain.height);
    w_cam.update_matrices();

    let wv = Self {
      width: w_swapchain.width,
      height: w_swapchain.height,
      w_tl,
      w_swapchain,
      shared_ubo,
      shared_bind_group: shared_bind_group,
      frame: 0,
      w_device,
      w_grouper,
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

  fn draw_frame(&mut self) {}

  fn destroy(&self) {
    // unsafe{ self.instance.destroy_instance(None); }
  }
}
