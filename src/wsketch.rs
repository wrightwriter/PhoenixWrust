#[allow(non_snake_case)]
use ash::vk::{self};
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
    img::wimage::WImageInfo,
    img::wrendertarget::{WRenderTarget, WRenderTargetInfo},
    wmodel::WModel,
  },
  sys::{
    warenaitems::{WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxRt, WAIdxUbo, WArenaItem},
    wbarr::WBarr,
    wcommandencoder::WCommandEncoder,
    wdevice::{WDevice, GLOBALS},
  },
  wdef,
};

use std::{borrow::BorrowMut};

// !! ---------- MAIN ---------- //


pub struct Sketch {
  pub encoder: WCommandEncoder,

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

//   pub test_buff: WAIdxBuffer,

  pub flame_pass: WComputePass,

  pub thing: WThing,
  pub thing_mesh: WThing,

  pub thing_path: WThingPath,

//   pub font: WFont,
  pub thing_text: WThingText,
}

pub fn init_sketch() -> Sketch{
unsafe {
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

      // !! ---------- RTs ---------- //
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

      let mut kernel_pass = WKernelPass::new(WV, false);
      kernel_pass.get_uniforms_container().exposed = true;

      // let test_model = WModel::new( "battle\\scene.gltf", WV,);
      // let test_model = WModel::new( "gltf_test_models\\DamagedHelmet\\glTF\\DamagedHelmet.gltf", WV,);
      let test_model = WModel::new("gltf_test_models\\Sponza\\glTF\\Sponza.gltf", WV);

      // let test_model = WModel::new("test.gltf", WV);
      let mut thing_mesh = WThing::new(WV, prog_mesh);
      thing_mesh.model = Some(test_model);


      let font = WFont::new(WV, "ferritecore.otf");
      // !! ---------- END INIT ---------- //
      let mut sketch = Sketch {
        // test_buff,
        // comp_pass,
        thing: WThing::new(WV, prog_render),
        rt_gbuffer,
        encoder: command_encoder,
        thing_mesh,
        rt_composite,
        composite_pass: WFxPass::new(WV, false, prog_composite),
        fx_composer: WFxComposer::new(WV),
        chromab_pass: WFxPass::new_from_frag_shader(WV, false, "FX/chromab.frag"),
        fxaa_pass: WFxPass::new_from_frag_shader(WV, false, "FX/fxaa.frag"),
        gamma_pass: WFxPass::new_from_frag_shader(WV, false, "FX/gamma.frag"),
        kernel_pass,
        // font: font,
        thing_path,
        flame_pass: WComputePass::new(WV, prog_flame),
        thing_text: WThingText::new(WV, prog_text, font),

        // flame_pass,
        flame_img,
        // test_video,
        // test_video,
        // test_model,
      };

      // big brain 🧠🧠
      WV.w_device.device.queue_wait_idle(WV.w_device.queue);
      sketch
    }
}

#[profiling::function]
pub fn render_sketch(
  s: &mut Sketch,
  rt: &mut WRenderTarget,
  imgui_cmd_buff: vk::CommandBuffer,
  wait_semaphore: vk::Semaphore,
  signal_semaphore: vk::Semaphore,
) {
  unsafe {
    let w = &mut *GLOBALS.w_vulkan;
    s.encoder.reset(&mut w.w_device);


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

  };
}

    