use std::collections::HashMap;

use ash::vk;
use macros::add_uniform;
use macros::init_uniform;
use smallvec::SmallVec;

use crate::abs::passes::wpostpass::init_fx_pass_stuff;
use crate::abs::passes::wpostpass::WPassTrait;
use crate::abs::thing::wthingnull::WThingNull;
use crate::declare_pass;
use crate::res::img::wimage::WImageInfo;
use crate::res::img::wrendertarget::WRPConfig;
use crate::res::img::wrendertarget::WRenderTarget;
use crate::sys::command::wbarr::WBarr;
use crate::sys::pipeline::wpipelineconfig::WPipelineConfig;
use crate::sys::warenaitems::WAIdxImage;
use crate::sys::wtl::WTechLead;
use crate::{
  res::{
    buff::{wpushconstant::WPushConstant, wuniformscontainer::WParamsContainer, wwritablebuffertrait::WWritableBufferTrait},
    img::wrendertarget::WRTInfo,
    wshader::WShaderEnumPipelineBind,
  },
  sys::{
    pipeline::wrenderpipeline::{WRenderPipeline, WRenderPipelineTrait},
    warenaitems::{WAIdxBindGroup, WAIdxRenderPipeline, WAIdxRt, WAIdxShaderProgram, WAIdxUbo, WArenaItem},
    wdevice::{WDevice, GLOBALS},
  },
  wvulkan::WVulkan,
};
use nalgebra_glm as glm;

impl WDOFPass {
  add_uniform!(0, f32, Threshold);
  add_uniform!(1, f32, Ramp);
  add_uniform!(2, f32, Amount);

  pub fn new(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    has_rt: bool,
  ) -> Self {
    let sp = w_v
      .w_shader_man
      .new_render_program(&mut w_v.w_device, "fullscreenQuad.vert", "FX/dof.frag");

    let sp_scatter = w_v
      .w_shader_man
      .new_render_program(&mut w_v.w_device, "FX/dofScatter.vert", "FX/dofScatter.frag");

    let thing = WThingNull::new(w_v, w_tl, sp, Some(WPipelineConfig::fullscreenQuad()));
    let thing_scatter = WThingNull::new(w_v, w_tl, sp_scatter, Some({
      let mut cfg = WPipelineConfig::fullscreenQuad();
      cfg.depth_test_enable = false;
      cfg.blend_state = vk::PipelineColorBlendAttachmentState {
        blend_enable: 1,
        src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_color_blend_factor: vk::BlendFactor::ONE,
        color_blend_op: vk::BlendOp::ADD,
        // src_alpha_blend_factor: vk::BlendFactor::ONE,
        src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_alpha_blend_factor: vk::BlendFactor::ONE,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::R
          | vk::ColorComponentFlags::G
          | vk::ColorComponentFlags::B
          | vk::ColorComponentFlags::A,
      };
      cfg
    }));



    let s = init_fx_pass_stuff(w_v, w_tl, has_rt, sp);

    let res = glm::vec2(w_v.width, w_v.height);

    let mut image_info = WImageInfo {
      resx: res.x,
      resy: res.y,
      format: vk::Format::R32G32B32A32_SFLOAT,
      ..wdef!()
    };

    let rt_info = WRTInfo::from_images(&[w_tl.new_image(w_v, image_info.clone()).0]);
    let rt_near = w_tl.new_render_target(w_v, rt_info.clone()).0;
    let rt_info = WRTInfo::from_images(&[w_tl.new_image(w_v, image_info.clone()).0]);
    let rt_far = w_tl.new_render_target(w_v, rt_info).0;
    let rt_info = WRTInfo::from_images(&[w_tl.new_image(w_v, image_info.clone()).0]);
    let rt_scatter = w_tl.new_render_target(w_v, rt_info).0;


    let mut s = Self {
      rt: s.0,
      shader_program: sp,
      render_pipeline: s.2,
      ubo: s.3,
      bind_groups: s.4,
      bind_group: s.5,
      // uniforms: s.6,
      push_constants: s.7,
      push_constants_internal: s.8,
      thing,
      thing_scatter,
      rt_near,
      rt_far,
      rt_scatter,
      sp_scatter,
    };

    unsafe {
      let uniforms = s.get_uniforms_container();
      uniforms.uniforms.set_len(3);
    }

    init_uniform!(s, Threshold, 0.3);
    init_uniform!(s, Ramp, 0.1);
    init_uniform!(s, Amount, 1.);

    s
  }
}

declare_pass!(
  WDOFPass {
    rt_near: WAIdxRt,
    rt_far: WAIdxRt,
    rt_scatter: WAIdxRt,
    sp_scatter: WAIdxShaderProgram,
    thing: WThingNull,
    thing_scatter: WThingNull
  },
  true,
  |me: &mut WDOFPass,
   w_v: &mut WVulkan,
   w_tl: &mut WTechLead,
   img_in: Option<WAIdxImage>,
   rt_idx: WAIdxRt|
   -> SmallVec<[vk::CommandBuffer; 30]> {
    debug_assert!(me.push_constants.len() == 2);

    let mut cmd_bufs = SmallVec::new();

    me.thing.push_constants.set_len(5);
    me.thing.push_constants.set_at(0, me.push_constants[0]);
    me.thing.push_constants.set_at(1, me.push_constants[1]);
    me.thing.push_constants.set_at(2, me.rt_far.get().image_at(0).idx.index as u16);
    me.thing.push_constants.set_at(3, me.rt_near.get().image_at(0).idx.index as u16);

    me.push_constants.set_len(5);
    me.push_constants.set_at(2, me.rt_far.get().image_at(0).idx.index as u16);
    me.push_constants.set_at(3, me.rt_near.get().image_at(0).idx.index as u16);
    me.push_constants.set_at(4, 2u8); // seg

    if false{
      // {
      //   let cmd_buf = me.rt_far.get_mut().begin_pass(w_v);

      //   {
      //     me.thing.push_constants.set_at(4, 0u8); // seg
      //     me.thing.draw_cnt(w_v, w_tl, Some(me.rt_far), &cmd_buf, 4, 1);
      //   }

      //   cmd_bufs.push(me.rt_far.get_mut().end_pass(&mut w_v.w_device));
      // }

      // {
      //   let cmd_buf = me.rt_near.get_mut().begin_pass(w_v);

      //   {
      //     me.thing.push_constants.set_at(4, 1u8); // seg
      //     me.thing.draw_cnt(w_v, w_tl, Some(me.rt_near), &cmd_buf, 4, 1);
      //   }

      //   cmd_bufs.push(me.rt_near.get_mut().end_pass(&mut w_v.w_device));
      // }
      // cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));
    }

    if false{
      {
        // let cmd_buf = me.rt_scatter.get_mut().begin_pass(w_v);
        // // let cmd_buf = me.rt_scatter.get_mut().begin_pass_ext(w_v,
        // //    WRPConfig { load_op: (), store_op: () }
        // // );

        // me.thing_scatter.push_constants.set_len(5);
        // me.thing_scatter.push_constants.set_at(0, me.push_constants[0]);
        // me.thing_scatter.push_constants.set_at(1, me.push_constants[1]);
        // me.thing_scatter.push_constants.set_at(2, me.rt_far.get().image_at(0).idx.index as u16);
        // me.thing_scatter.push_constants.set_at(3, me.rt_near.get().image_at(0).idx.index as u16);
        // {
        //   me.thing_scatter.push_constants.set_at(4, 1u8); // seg
        //   me.thing_scatter.draw_cnt(w_v, w_tl, Some(me.rt_near), &cmd_buf, 4, w_v.w_cam.width * w_v.w_cam.height);
        // }

        // cmd_bufs.push(me.rt_scatter.get_mut().end_pass(&mut w_v.w_device));
      }
      cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));
      me.push_constants.set_at(2, me.rt_scatter.get().image_at(0).idx.index as u16);
    }
    
    if true {
      {
        let cmd_buf = me.rt_far.get_mut().begin_pass(w_v);

        {
          me.thing.push_constants.set_at(4, 2u8); // seg
          me.thing.draw_cnt(w_v, w_tl, Some(me.rt_far), &cmd_buf, 4, 1);
        }

        cmd_bufs.push(me.rt_far.get_mut().end_pass(&mut w_v.w_device));
      }

      cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));


      me.push_constants.set_at(4, 3u8); // seg
    }



    let cmd_buf = me.run_ext(w_v, w_tl, None, rt_idx);
    cmd_bufs.push(cmd_buf[0]);

    // cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));

    // smallvec::smallvec![]
    cmd_bufs
  }
);
