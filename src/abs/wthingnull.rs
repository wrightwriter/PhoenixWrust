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
use crate::sys::warenaitems::WAIdxBindGroup;
use crate::sys::warenaitems::WAIdxRenderPipeline;
use crate::sys::warenaitems::WAIdxRt;
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::warenaitems::WAIdxUbo;
use crate::sys::warenaitems::WArenaItem;
use crate::sys::wbindgroup::WBindGroupsHaverTrait;
use crate::sys::wdevice::GLOBALS;
use crate::sys::wmanagers::WTechLead;
use crate::sys::wrenderpipeline::WRenderPipeline;
use crate::sys::wrenderpipeline::WRenderPipelineTrait;
use crate::wvulkan::WVulkan;

use crate::abs::wthingtrait::WThingTrait;


use super::wthingtrait::init_thing_stuff;


declare_thing!(WThingNull{
});
impl_thing_trait!(WThingNull{});

impl WThingNull {
  pub fn new(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    prog_render: WAIdxShaderProgram,
  ) -> Self {
    let s = init_thing_stuff(w_v, w_tl, prog_render);


    let s = Self {
      render_pipeline: s.2,
      // render_pipeline_box: render_pipeline_box,
      bind_groups: s.4,
      bind_group: s.5,
      ubo: s.3,
      movable: false,
      world_pos: Vec3::zeros(),
      model_mat: Mat4::identity(),
      rt: None,
      push_constants: s.7,
      // uniforms: WUniformsContainer::new(),
      push_constants_internal: s.8,
    };


    {
      let mut rp = s.render_pipeline.get_mut();
      rp.input_assembly.topology = vk::PrimitiveTopology::TRIANGLE_STRIP;
      rp.init();
      rp.refresh_pipeline(
        &w_v.w_device.device,
        w_tl,
        // bind_groups,
      );

    }
    
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
    let w_device = &mut w_v.w_device;
    // let w_grouper = &mut w_v.w_grouper;

    if let Some(rt) = rt {
      if self.rt.is_none() {
        self.rt = Some(rt);

        let rp = self.render_pipeline.get_mut();
        rp.set_pipeline_render_target(rt.get_mut());
        rp.refresh_pipeline(&w_device.device, w_tl);
      }
    }

    {
      let ubo_buff = self.get_ubo().get_mut();
      let ubo_buff = &mut ubo_buff.buff;
      ubo_buff.reset_ptr();
    }

    WParamsContainer::upload_uniforms(*self.get_ubo(), &self.get_uniforms_container());

    self.init_render_settings(w_device, w_tl, command_buffer);

    unsafe {
        self.update_push_constants(w_device, command_buffer);
        w_device.device.cmd_draw(*command_buffer, tri_count, instance_cnt, 0, 0);
    }
  }
}





