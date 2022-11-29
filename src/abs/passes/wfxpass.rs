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


declare_pass!(WFxPass {});

impl WFxPass {
  pub fn new_from_frag_shader<S: Into<String>>(
    w_v: &mut WVulkan,
    w_t_l: &mut WTechLead,
    has_rt: bool,
    shader_path: S,
  ) -> Self {
    let shader_program = w_v
      .w_shader_man
      .new_render_program(&mut w_v.w_device, "fullscreenQuad.vert", &shader_path.into());
    Self::new(w_v, w_t_l, has_rt, shader_program)
  }
  pub fn new(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    has_rt: bool,
    shader_program: WAIdxShaderProgram,
  ) -> Self {
    let s = init_fx_pass_stuff(w_v, w_tl, has_rt, shader_program);

    Self {
      rt: s.0,
      shader_program,
      render_pipeline: s.2,
      ubo: s.3,
      bind_groups: s.4,
      bind_group: s.5,
      // uniforms: s.6,
      push_constants: s.7,
      push_constants_internal: s.8,
    }
  }
}


impl WRenderPipelineTrait for WFxPass {
  fn get_pipeline(&self) -> &WRenderPipeline {
    &*self.render_pipeline.get_mut()
  }
}