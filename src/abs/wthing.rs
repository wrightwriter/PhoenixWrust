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
use crate::sys::wtl::WTechLead;
use crate::sys::wrenderpipeline::WRenderPipeline;
use crate::sys::wrenderpipeline::WRenderPipelineTrait;
use crate::wvulkan::WVulkan;

use crate::abs::wthingtrait::WThingTrait;


use super::wthingtrait::init_thing_stuff;


declare_thing!(WThing{
  model: Option<WModel>
});
impl_thing_trait!(WThing{});

// pub struct WThing{
// }

impl WThing {
  pub fn new(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    prog_render: WAIdxShaderProgram,
  ) -> Self {
    let s = init_thing_stuff(w_v, w_tl, prog_render);

    Self {
      render_pipeline: s.2,
      // render_pipeline_box: render_pipeline_box,
      bind_groups: s.4,
      bind_group: s.5,
      ubo: s.3,
      movable: false,
      world_pos: Vec3::zeros(),
      model_mat: Mat4::identity(),
      model: None,
      rt: None,
      push_constants: s.7,
      // uniforms: WUniformsContainer::new(),
      push_constants_internal: s.8,
    }
  }

  #[profiling::function]
  pub fn draw(
    &mut self,
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    rt: Option<WAIdxRt>,
    command_buffer: &vk::CommandBuffer,
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
      let model_mat = self.model_mat;
      let ubo_buff = self.get_ubo().get_mut();
      let ubo_buff = &mut ubo_buff.buff;
      ubo_buff.reset_ptr();
      ubo_buff.write_mat4x4(model_mat);
    }

    WParamsContainer::upload_uniforms(*self.get_ubo(), &self.get_uniforms_container());

    self.init_render_settings(w_device, w_tl, command_buffer);

    unsafe {
      // -- PUSH CONSTANTS -- //
      let ubo_address = (*GLOBALS.shared_ubo_arena)[self.ubo.idx] // make this shorter? no?
        .buff
        .get_bda_address();

      // -- DRAW -- //
      if let Some(model) = &self.model {
        for mesh in &model.meshes {
          self.push_constants_internal.reset_ptr();

          let indices_arena_idx = mesh.gpu_indices_buff.get().arena_index;
          let verts_arena_idx = mesh.gpu_verts_buff.get().arena_index;

          self.push_constants_internal.write(ubo_address);

          let indices_idx = indices_arena_idx.idx.index as u16 - 1;
          let verts_idx = verts_arena_idx.idx.index as u16 - 1;

          self.push_constants_internal.write(indices_idx.shr(8) as u8);
          self.push_constants_internal.write(indices_idx as u8);

          self.push_constants_internal.write(verts_idx.shr(8) as u8);
          self.push_constants_internal.write(verts_idx as u8);

          // self.push_constants_internal.write(verts_arena_idx.idx.index as u8 - 1);

          let mut i = 0;
          // if(model.textures.len() > 0){
            self
              .push_constants_internal
              .write(model.textures[0].idx.index as u8 + mesh.material.diffuse_tex_idx as u8);
            self
              .push_constants_internal
              .write(model.textures[1].idx.index as u8 + mesh.material.normal_tex_idx as u8);
            self
              .push_constants_internal
              .write(model.textures[2].idx.index as u8 + mesh.material.metallic_roughness_tex_idx as u8);
            self
              .push_constants_internal
              .write(model.textures[3].idx.index as u8 + mesh.material.occlusion_tex_idx as u8);
          // }

          self.push_constants_internal.write_params_container(&self.push_constants);

          w_device.device.cmd_push_constants(
            *command_buffer,
            self.render_pipeline.get_mut().pipeline_layout,
            vk::ShaderStageFlags::ALL,
            0,
            &self.push_constants_internal.array,
          );

          w_device.device.cmd_draw(*command_buffer, mesh.indices_len, 1, 0, 0);
        }
      } else {
        self.update_push_constants(w_device, command_buffer);
        w_device.device.cmd_draw(*command_buffer, 3, 1, 0, 0);
      }
    }
  }
}





