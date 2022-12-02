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

impl WBloomPass {
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
      .new_render_program(&mut w_v.w_device, "fullscreenQuad.vert", "FX/boxblur.frag");

    let thing = WThingNull::new(
      w_v,
      w_tl,
      sp,
      Some({
        let mut cfg = WPipelineConfig::fullscreenQuad();
        cfg.topology = vk::PrimitiveTopology::TRIANGLE_STRIP;
        cfg
      }),
    );
    let thing_upscale = WThingNull::new(
      w_v,
      w_tl,
      sp,
      Some({
        let mut cfg = WPipelineConfig::fullscreenQuad();
        cfg.topology = vk::PrimitiveTopology::TRIANGLE_STRIP;
        cfg.blend_state = vk::PipelineColorBlendAttachmentState {
          blend_enable: 1,
          src_color_blend_factor: vk::BlendFactor::ONE,
          dst_color_blend_factor: vk::BlendFactor::ONE,
          color_blend_op: vk::BlendOp::ADD,
          src_alpha_blend_factor: vk::BlendFactor::ONE,
          dst_alpha_blend_factor: vk::BlendFactor::ONE,
          alpha_blend_op: vk::BlendOp::ADD,
          color_write_mask: vk::ColorComponentFlags::R
            | vk::ColorComponentFlags::G
            | vk::ColorComponentFlags::B
            | vk::ColorComponentFlags::A,
        };
        cfg
      }),
    );

    let s = init_fx_pass_stuff(w_v, w_tl, has_rt, sp);

    let iters = 6;
    let mut rts = SmallVec::new();

    let mut res = glm::vec2(w_v.width, w_v.height);
    for i in 0..iters {
      res /= 2;

      let mut image_info = WImageInfo {
        resx: res.x,
        resy: res.y,
        format: vk::Format::R32G32B32A32_SFLOAT,
        ..wdef!()
      };
      let rt_info = WRTInfo::from_images(&[w_tl.new_image(w_v, image_info).0]);
      rts.push(w_tl.new_render_target(w_v, rt_info).0);
    }

    let mut s = Self {
      rt: s.0,
      rts,
      shader_program: sp,
      render_pipeline: s.2,
      ubo: s.3,
      bind_groups: s.4,
      bind_group: s.5,
      // uniforms: s.6,
      push_constants: s.7,
      push_constants_internal: s.8,
      iters,
      thing,
      thing_upscale,
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
  WBloomPass {
    iters: u32,
    rts: SmallVec<[WAIdxRt; 10]>,
    thing: WThingNull,
    thing_upscale: WThingNull
  },
  true,

  |
    me: &mut WBloomPass,
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    img_in: Option<WAIdxImage>,
    rt_idx: WAIdxRt
  | -> SmallVec<[vk::CommandBuffer;30]> {

    // me.thing.rt = Some(me.rts[0]);

    let mut cmd_bufs = SmallVec::new();
    cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));

    let mut i: u8 = 0;
    let iter_cnt = me.rts.len() as u8;
    for rt in me.rts.clone(){
      let cmd_buf = rt.get_mut().begin_pass(&mut w_v.w_device);

      {
        me.thing.push_constants.reset();
        if i == 0{
          me.thing.push_constants.add(img_in.unwrap());
        } else {
          me.thing.push_constants.add(me.rts[i as usize-1].get().image_at(0));
        }
        me.thing.push_constants.add(img_in.unwrap());
        me.thing.push_constants.add(i); // iter
        me.thing.push_constants.add(iter_cnt);
        me.thing.push_constants.add(0 as u8); // seg
        me.thing.draw_cnt(w_v, w_tl, Some(rt), &cmd_buf, 4, 1);
      }

      cmd_bufs.push( rt.get_mut().end_pass(&mut w_v.w_device));
      cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));

      // w_v.w_device.device.cmd_set_blend_constants();
      i += 1;
    }

    for i in 0..(iter_cnt - 1){
      let rt = me.rts[(iter_cnt - 2 - i) as usize];
      // let cmd_buf = rt.get_mut().begin_pass_ext(&mut w_v.w_device, {
      //   let mut cfg = WRPConfig::default();
      //   cfg.load_op = Some(smallvec::smallvec![vk::AttachmentLoadOp::LOAD]);
      //   cfg
      // });

      let cmd_buf = rt.get_mut().begin_pass_ext(&mut w_v.w_device, {
        let mut cfg = WRPConfig::default();
        cfg.load_op = Some(smallvec::smallvec![vk::AttachmentLoadOp::LOAD]);
        cfg
      });


      me.thing_upscale.push_constants.reset();
      {
        me.thing_upscale.push_constants.add(me.rts[iter_cnt as usize - 1 - i as usize].get().image_at(0));
        me.thing_upscale.push_constants.add(img_in.unwrap());
        me.thing_upscale.push_constants.add(iter_cnt + i); // iter
        me.thing_upscale.push_constants.add(iter_cnt);
        me.thing_upscale.push_constants.add(1 as u8); // seg
        me.thing_upscale.draw_cnt(w_v, w_tl, Some(rt), &cmd_buf, 4, 1);
      }

      cmd_bufs.push( rt.get_mut().end_pass(&mut w_v.w_device));
      cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));

    }


    me.push_constants.reset();
    // me.push_constants.add((me.rts[me.rts.len()-1]).get().image_at(0));

    me.push_constants.add(img_in.unwrap());
    me.push_constants.add(0u8);  // iter
    me.push_constants.add(iter_cnt);
    me.push_constants.add(2u8);  // seg

    // cmd_bufs.push(WPassTrait::run_ext(me, w_v, w_tl, rt_idx)[0]);
    let cmd_buf = me.run_ext(w_v, w_tl, Some((me.rts[0]).get().image_at(0)), rt_idx);
    cmd_bufs.push(cmd_buf[0]);

    cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));

    // smallvec::smallvec![]
    cmd_bufs
  }
);
