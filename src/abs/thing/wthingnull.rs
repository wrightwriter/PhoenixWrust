use std::collections::HashMap;
use std::ops::Shr;

use ash::vk;
use nalgebra_glm::Mat4;
use nalgebra_glm::Vec3;

use crate::declare_thing;
use crate::impl_thing_trait;
use crate::res::buff::wpushconstant::WPushConstant;
use crate::res::buff::wuniformscontainer::WParamsContainer;
use crate::res::buff::wwritablebuffertrait::WWritableBufferTrait;
use crate::res::wmodel::WModel;
use crate::sys::pipeline::wpipelineconfig::WPipelineConfig;
use crate::sys::warenaitems::WAIdxBindGroup;
use crate::sys::warenaitems::WAIdxRenderPipeline;
use crate::sys::warenaitems::WAIdxRt;
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::warenaitems::WAIdxUbo;
use crate::sys::warenaitems::WArenaItem;
use crate::sys::pipeline::wbindgroup::WBindGroupsHaverTrait;
use crate::sys::wdevice::GLOBALS;
use crate::sys::wtl::WTechLead;
use crate::sys::pipeline::wrenderpipeline::WRenderPipeline;
use crate::sys::pipeline::wrenderpipeline::WRenderPipelineTrait;
use crate::wvulkan::WVulkan;

use crate::sys::pipeline::wrenderstate::WRenderState;

use crate::abs::thing::wthingtrait::WThingTrait;


use super::wthingtrait::init_thing_stuff;

declare_thing!(WThingNull{
});
impl_thing_trait!(WThingNull{});

impl WThingNull {
  pub fn new(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    prog_render: WAIdxShaderProgram,
    mut pip_cfg: Option<WPipelineConfig>
  ) -> Self {

    let pipeline_config: WPipelineConfig = {
      if let Some(pipeline_config) = pip_cfg{
        println!("{:?}",pipeline_config.blend_state);
        pipeline_config
      } else {
        WPipelineConfig::fullscreenQuad()
      }
    };
    let s = init_thing_stuff(w_v, w_tl, prog_render, pipeline_config);

    let mut s = Self {
      render_pipeline: s.2,
      bind_groups: s.4,
      bind_group: s.5,
      ubo: s.3,
      movable: false,
      world_pos: Vec3::zeros(),
      model_mat: Mat4::identity(),
      rt: None,
      push_constants: s.7,
      push_constants_internal: s.8,
      render_state: s.9,
    };

    s.render_state.depth_test = false;
    s.render_state.depth_write = false;
    s.render_state.cull_mode = vk::CullModeFlags::NONE;
    
    s
  }

  #[profiling::function]
  pub fn draw_cnt(
    &mut self,
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    rt: Option<WAIdxRt>,
    command_buffer: &vk::CommandBuffer,
    tri_count: u32,
    instance_cnt: u32,
  ) {
    // let w_grouper = &mut w_v.w_grouper;


    if let Some(rt) = rt {
      if self.rt.is_none() {
        self.rt = Some(rt);

        let rp = self.render_pipeline.get_mut();
        rp.set_pipeline_render_target(rt.get_mut());
        rp.refresh_pipeline(w_v, w_tl);
      }
    }
    let w_device = &mut w_v.w_device;

    self.update_push_constants(w_device, command_buffer);

    {
      let ubo_buff = self.get_ubo().get_mut();
      let ubo_buff = &mut ubo_buff.buff;
      ubo_buff.reset_ptr();
    }


    WParamsContainer::upload_uniforms(*self.get_ubo(), &self.get_uniforms_container());

    self.init_render_settings(w_device, w_tl, command_buffer);

    unsafe {
        w_device.device.cmd_draw(*command_buffer, tri_count, instance_cnt, 0, 0);
    }
  }
}





