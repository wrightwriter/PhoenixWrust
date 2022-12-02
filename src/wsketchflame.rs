#[allow(non_snake_case)]
use ash::vk::{self};
use crate::{
  abs::{
    wcam::WCamera,
    wcomputepass::WComputePass,
    wfxcomposer::WFxComposer,
    wibl::WIbl, wbrdf::WBrdf, passes::{wpostpass::{ WPassTrait}, wfxpass::WFxPass, wkernelpass::WKernelPass, wbloompass::WBloomPass}, thing::{wthingshape::WThingPath, wthingtext::WThingText, wthing::WThing},
  },
  msdf::msdf::WFont,
  res::{
    img::wimage::WImageInfo,
    img::wrendertarget::{WRenderTarget, WRTInfo},
    wmodel::WModel,
  },
  sys::{
    warenaitems::{WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxRt, WAIdxUbo, WArenaItem},
    command::wbarr::WBarr,
    command::wcommandencoder::WCommandEncoder,
    wdevice::{WDevice, GLOBALS},
  },
  wdef,
};

use std::{borrow::BorrowMut};

// !! ---------- MAIN ---------- //


pub struct SketchFlame {
  pub encoder: WCommandEncoder,

  pub flame_img: WAIdxImage,

  // pub test_video: WVideo,
  pub rt_gbuffer: WAIdxRt,
  pub rt_composite: WAIdxRt,
  pub rt_bloom: WAIdxRt,
  pub rt_tonemap_taa: WAIdxRt,

  pub tonemap_taa_pass: WFxPass,
  pub composite_pass: WFxPass,

  pub fx_composer: WFxComposer,

  pub bloom_pass: WBloomPass,
  pub chromab_pass: WFxPass,
  pub kernel_pass: WKernelPass,
  pub gamma_pass: WFxPass,
  pub fxaa_pass: WFxPass,


  pub ibl: WIbl,
  pub brdf: WBrdf,
//   pub test_buff: WAIdxBuffer,

  pub flame_pass: WComputePass,
  pub particles_buff: WAIdxBuffer,

  pub thing: WThing,
  pub thing_mesh: WThing,

  pub thing_path: WThingPath,

//   pub font: WFont,
  pub thing_text: WThingText,
}

pub fn init_sketch() -> SketchFlame{
unsafe {
      let w_v = &mut *GLOBALS.w_vulkan;
      let w_tl = &mut *GLOBALS.w_tl;
      let command_encoder = WCommandEncoder::new();

      // !! ---------- Video ---------- //
      // let test_video = WVideo::new(WV);
      // !! ---------- IMG ---------- //


      // !! ---------- SHADER ---------- //
      let prog_mesh = w_v.w_shader_man.new_render_program(&mut w_v.w_device, "mesh.vert", "mesh.frag");

      let prog_render = w_v
        .w_shader_man
        .new_render_program(&mut w_v.w_device, "triangle.vert", "triangle.frag");

      let prog_flame = w_v.w_shader_man.new_compute_program(&mut w_v.w_device, "flameMesh.comp");

      // !! ---------- COMP ---------- //
      let mut flame_pass = WComputePass::new(w_v, w_tl, prog_flame);

      let prog_path = w_v.w_shader_man.new_render_program(&mut w_v.w_device, "path.vert", "path.frag");

      let prog_text = w_v.w_shader_man.new_render_program(&mut w_v.w_device, "text.vert", "text.frag");

      
      // let ibl = WIbl::new(w_v, w_tl, "hdri\\unfinished_office_2k.exr");
      let ibl = WIbl::new(w_v, w_tl, "hdri\\kloofendal_38d_partly_cloudy_puresky_2k.exr");


      // !! ---------- Lyon ---------- //

      let mut thing_path = WThingPath::new(w_v, w_tl, prog_path);
      thing_path.path();

      // !! ---------- RTs ---------- //
      let mut rt_create_info = WRTInfo {
        resx: w_v.w_cam.width,
        resy: w_v.w_cam.height,
        attachment_infos: vec![
          WImageInfo { format: vk::Format::R32G32B32A32_SFLOAT,..wdef!() },
          WImageInfo { ..wdef!() },
          WImageInfo { ..wdef!() },
          WImageInfo { ..wdef!() },
        ],
        ..wdef!()
      };
      let rt_gbuffer = w_tl.new_render_target(w_v, rt_create_info.clone()).0;

      rt_create_info.has_depth = false;
      rt_create_info.attachment_infos = WRTInfo::default().attachment_infos;
      rt_create_info.pongable = true;
      // rt_create_info.format = vk::Format::R32G32B32A32_SFLOAT; // remove alpha?
      let rt_tonemap_taa = w_tl.new_render_target(w_v, rt_create_info.clone()).0;

      rt_create_info.pongable = false;
      rt_create_info.attachment_infos[0].format = vk::Format::R32G32B32A32_SFLOAT;
      // rt_create_info.attachment_infos
      // rt_create_info.format = vk::Format::R32G32B32A32_SFLOAT; // remove alpha?
      let rt_composite = w_tl.new_render_target(w_v, rt_create_info.clone()).0;


      let rt_bloom = w_tl.new_render_target(w_v, rt_create_info.clone()).0;

      let mut flame_img = w_tl
        .new_image(
          w_v,
          WImageInfo {
            resx: w_v.w_cam.width,
            resy: w_v.w_cam.height,
            format: vk::Format::R32_SFLOAT,
            ..wdef!()
          },
        )
        .0;

      let mut kernel_pass = WKernelPass::new(w_v, w_tl, false);
      kernel_pass.get_uniforms_container().exposed = true;

      // let test_model = WModel::new("gltf_test_models\\Sponza\\glTF\\Sponza.gltf", w_v, w_tl);
      // let test_model = WModel::new( "battle\\scene.gltf", WV,);
      let test_model = WModel::new( "gltf_test_models\\DamagedHelmet\\glTF\\DamagedHelmet.gltf", w_v, w_tl);
      // let test_model = WModel::new("sponza2\\NewSponza_Main_glTF_002.gltf", w_v, w_tl);
      // let test_model = WModel::new("AdamHead\\adamHead.gltf", w_v, w_tl);

      // let test_model = WModel::new("test.gltf", WV);
      let mut thing_mesh = WThing::new(w_v, w_tl, prog_mesh);
      thing_mesh.model = Some(test_model);


      let font = WFont::new(w_v, w_tl, "ferritecore.otf");


      // !! ---------- END INIT ---------- //
      let mut sketch = SketchFlame {
        // test_buff,
        // comp_pass,
        ibl,
        particles_buff: w_tl.new_buffer(w_v, vk::BufferUsageFlags::STORAGE_BUFFER, 14556 * 4 * 4 * 8*8, false).0,
        thing: WThing::new(w_v,  w_tl,prog_render),
        rt_gbuffer,
        encoder: command_encoder,
        thing_mesh,
        rt_composite,
        rt_tonemap_taa,
        tonemap_taa_pass: WFxPass::new_from_frag(w_v, w_tl, false,  "sketchb\\tonemap_and_taa.frag"),
        composite_pass: WFxPass::new_from_frag(w_v, w_tl, false,  "sketchb\\composite.frag"),
        fx_composer: WFxComposer::new(w_v, w_tl),
        chromab_pass: WFxPass::new_from_frag(w_v, w_tl, false, "FX/chromab.frag"),
        bloom_pass: WBloomPass::new(w_v, w_tl, false),
        fxaa_pass: WFxPass::new_from_frag(w_v, w_tl, false, "FX/fxaa.frag"),
        gamma_pass: WFxPass::new_from_frag(w_v, w_tl, false, "FX/gamma.frag"),
        kernel_pass,
        // font: font,
        thing_path,
        flame_pass: WComputePass::new(w_v, w_tl, prog_flame),
        thing_text: WThingText::new(w_v, w_tl, prog_text, font),

        // flame_pass,
        flame_img,
        brdf: WBrdf::new(w_v, w_tl),
        rt_bloom,
      
        // test_video,
        // test_video,
        // test_model,
      };

      // big brain 🧠🧠
      w_v.w_device.device.queue_wait_idle(w_v.w_device.queue);
      sketch
    }
}

#[profiling::function]
pub fn render_sketch(
  s: &mut SketchFlame,
  rt: &mut WRenderTarget,
  imgui_cmd_buff: vk::CommandBuffer,
  wait_semaphore: vk::Semaphore,
  signal_semaphore: vk::Semaphore,
) {
  unsafe {
    let w = &mut *GLOBALS.w_vulkan;
    let w_tl = &mut *GLOBALS.w_tl;
    s.encoder.reset(&mut w.w_device);


    // unsafe{
    //   let cmd_buf = s.test_video.update_frame(w);
    //   s.encoder.push_buf(cmd_buf);
    // }

    s.encoder.push_barr(w, WBarr::render());

    // !! Render
    {
      let cmd_buf = { s.rt_gbuffer.get_mut().begin_pass(&mut w.w_device) };

      // s.thing.draw(w, Some(s.rt_gbuffer), &cmd_buf);
      // s.thing_path.draw(w, Some(s.rt_gbuffer), &cmd_buf);

      s.thing_mesh.push_constants.reset();
      s.thing_mesh.push_constants.add(0f32);
      s.thing_mesh.draw(w, w_tl, Some(s.rt_gbuffer), &cmd_buf);

      // s.thing_text.draw(w, Some(s.rt_gbuffer), &cmd_buf);

      {
        s.rt_gbuffer.get_mut().end_pass(&w.w_device);
        s.encoder.push_buf(cmd_buf);
      }
    }

    s.encoder
      .push_barr(w, WBarr::render());

    // !! COMPOSITE
    {
      s.composite_pass.push_constants.reset();
      
      s.composite_pass.push_constants.add_many(&[
        s.ibl.cubemap_prefilter,
        s.brdf.brdf,
        s.rt_gbuffer.get().image_at(0),
        s.rt_gbuffer.get().image_at(1),
        s.rt_gbuffer.get().image_at(2),
        s.rt_gbuffer.get().image_depth.unwrap(),
        s.rt_tonemap_taa.get().back_image_at(0),
        s.flame_img,
        // s.test_video.gpu_image,
      ]);

      s.encoder.push_bufs(&s.composite_pass.run_on_external_rt(s.rt_composite, w,  w_tl, None));

      s.encoder.push_barr(w, WBarr::render());
    }
    
    // !! BLOOM
    {
      s.encoder.push_bufs(
        &s.bloom_pass.run_on_external_rt(s.rt_bloom, w, w_tl, Some(s.rt_composite.get().image_at(0)))
      );
      // s.encoder.push_bufs(&s.tonemap_taa_pass.run_on_external_rt(s.rt_tonemap_taa, w,  w_tl, Some(s.rt_composite.get().image_at(0))));
      s.encoder.push_barr(w, WBarr::render());
    }

    // !! TAA, TONEMAP
    {
      s.tonemap_taa_pass.push_constants.reset();
      
      s.tonemap_taa_pass.push_constants.add_many(&[
        s.ibl.cubemap_prefilter,
        s.brdf.brdf,
        s.rt_gbuffer.get().image_at(0),
        s.rt_gbuffer.get().image_at(1),
        s.rt_gbuffer.get().image_at(2),
        s.rt_gbuffer.get().image_depth.unwrap(),
        s.rt_tonemap_taa.get().back_image_at(0),
        s.flame_img,
        // s.test_video.gpu_image,
      ]);

      s.encoder.push_bufs(&s.tonemap_taa_pass.run_on_external_rt(s.rt_tonemap_taa, w,  w_tl, Some(s.rt_bloom.get().image_at(0))));

      s.encoder.push_barr(w, WBarr::render());
    }

    // !! POST
    {

      s.fx_composer.begin(s.rt_tonemap_taa);
      // s.fx_composer.run(w, &mut s.fxaa_pass);
      // s.fx_composer.run(w, &mut s.kernel_pass);
      // s.fx_composer.run(w, &mut s.chromab_pass);
      // s.fx_composer.run(w, w_tl, &mut s.bloom_pass);
      s.fx_composer.run(w, w_tl, &mut s.gamma_pass);

      s.encoder.push_bufs(&s.fx_composer.cmd_bufs);
    }

    // s.encoder.push_buf();

    // {
    //   s.comp_pass.dispatch(w, 1, 1, 1);
    //   s.command_encoder.push_buff(s.comp_pass.command_buffer);
    // }

    // blit
    WDevice::blit_image_to_swapchain(w, &mut s.encoder, s.fx_composer.get_front_img(), &rt);
    // WDevice::blit_image_to_swapchain(w, &mut s.encoder, s.rt_composite.get().image_at(0), &rt);

    s.encoder.push_barr(w, WBarr::render());

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

    

    // !! FLAME


    // // clear flame
    // {
    //   let cmd_buf = s.encoder.get_and_begin_buff(&mut w.w_device);
    //   w.w_device.device.cmd_clear_color_image(
    //     cmd_buf,
    //     s.flame_img.get().handle,
    //     vk::ImageLayout::GENERAL,
    //     &vk::ClearColorValue::default(),
    //     &[vk::ImageSubresourceRange::builder()
    //       .aspect_mask(vk::ImageAspectFlags::COLOR)
    //       .base_array_layer(0)
    //       .layer_count(1)
    //       .level_count(1)
    //       .build()],
    //   );
    //   s.encoder.end_and_push_buff(&mut w.w_device, cmd_buf);
    // }

    // s.encoder.push_barr(
    //   w,
    //   WBarr::general()
    //     .src_stage(vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS)
    //     .dst_stage(vk::PipelineStageFlags2::COMPUTE_SHADER),
    // );


    // s.encoder.push_barr(w, WBarr::comp_to_frag());

    // let mut i = 0;
    // let model = s.thing_mesh.model.as_ref().unwrap();
    // for mesh in &model.meshes{
    //   s.flame_pass.push_constants.reset();

    //   s.flame_pass.push_constants.add(
    //     mesh.verts_len
    //   );
    //   s.flame_pass.push_constants.add_many(&[
    //     // s.rt_gbuffer.get().image_depth.unwrap(),
    //     s.flame_img,
    //     // s.rt_composite.get().image_at(0),
    //     // s.rt_composite.get().image_at(0),
    //     // s.test_video.gpu_image,
    //   ]);

    //   s.flame_pass.push_constants.add(
    //     // s.thing_mesh.model.as_ref().unwrap().meshes[0].gpu_verts_buff,
    //     mesh.gpu_verts_buff.get().arena_index,
    //       // let verts_arena_idx = mesh.gpu_verts_buff.get().arena_index;
    //   );
    //   s.flame_pass.push_constants.add(
    //     s.particles_buff
    //   );

    //   if w.w_time.frame % 1000 == 0{
    //     println!("{}", 
    //       // mesh.verts_len
    //       mesh.verts_len
    //     );
    //     i += 1;
    //   }

    //   s.encoder.push_buf(s.flame_pass.dispatch(w, mesh.verts_len, 1, 1));
    // }

    // {
      // s.flame_pass.push_constants.reset();
      // s.flame_pass.push_constants.add_many(&[
      //   s.rt_gbuffer.get().image_depth.unwrap(),
      //   s.flame_img,
      //   // s.rt_composite.get().image_at(0),
      //   // s.rt_composite.get().image_at(0),
      //   // s.test_video.gpu_image,
      // ]);
      // s.flame_pass.push_constants.add(
      //   s.thing_mesh.model.as_ref().unwrap().meshes[0].gpu_verts_buff,
      // );

      // s.encoder.push_buf(s.flame_pass.dispatch(w, 1000, 1, 1));
    // }

    // s.encoder
    //   .push_barr(w, &WBarr::general()
    //     .src_stage(vk::PipelineStageFlags2::FRAGMENT_SHADER)
    //     .dst_stage(vk::PipelineStageFlags2::COMPUTE_SHADER)
    //   );