use std::collections::HashMap;

use ash::vk;
use macros::add_uniform;
use macros::init_uniform;

use crate::abs::passes::wpostpass::init_fx_pass_stuff;
use crate::abs::passes::wpostpass::WPassTrait;
use crate::declare_pass;
use crate::sys::wtl::WTechLead;
use crate::{
  res::{
    buff::{wpushconstant::WPushConstant, wuniformscontainer::WParamsContainer, wwritablebuffertrait::WWritableBufferTrait},
    img::wrendertarget::WRTInfo,
    wshader::WShaderEnumPipelineBind,
  },
  sys::{
    warenaitems::{WAIdxBindGroup, WAIdxRenderPipeline, WAIdxRt, WAIdxShaderProgram, WAIdxUbo, WArenaItem},
    wdevice::{WDevice, GLOBALS},
    wrenderpipeline::{WRenderPipeline, WRenderPipelineTrait},
  },
  wvulkan::WVulkan,
};


declare_pass!(WKernelPass {});

impl WKernelPass {
  add_uniform!(0, f32, Type);
  add_uniform!(1, f32, EdgeType);
  add_uniform!(2, f32, KernSz);
  add_uniform!(3, f32, SharpenAmt);
  add_uniform!(4, f32, SharpenBias);
  add_uniform!(5, f32, BlurCenterBias);
  add_uniform!(6, f32, BlurEdgeBias);

  pub fn new(
    w_v: &mut WVulkan,
    w_t_l: &mut WTechLead,
    has_rt: bool,
  ) -> Self {
    let sp = w_v
      .w_shader_man
      .new_render_program(&mut w_v.w_device, "fullscreenQuad.vert", "FX/kernel.frag");
    let s = init_fx_pass_stuff(w_v, w_t_l, has_rt, sp);

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
    };

    unsafe {
      let uniforms = s.get_uniforms_container();
      uniforms.uniforms.set_len(7);
    }

    init_uniform!(s, Type, 0.);
    init_uniform!(s, EdgeType, 5.);
    init_uniform!(s, KernSz, 1.);
    init_uniform!(s, SharpenAmt, 1.);
    init_uniform!(s, SharpenBias, 0.6);
    init_uniform!(s, BlurCenterBias, 1. / 9.);
    init_uniform!(s, BlurEdgeBias, 0.5);

    s
  }
}