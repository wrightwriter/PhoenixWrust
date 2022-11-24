use std::collections::HashMap;

use ash::vk;
use ash::vk::BufferUsageFlags;
use lyon::lyon_tessellation::FillBuilder;
use lyon::path::builder::NoAttributes;
use nalgebra_glm::Mat4;
use nalgebra_glm::Vec3;

use crate::declare_thing;
use crate::impl_thing_trait;
use crate::msdf::msdf::WFont;
use crate::res::buff::wpushconstant::WPushConstant;
use crate::res::buff::wuniformscontainer::WParamsContainer;
use crate::res::buff::wwritablebuffertrait::WWritableBufferTrait;
use crate::res::wmodel::WModel;
use crate::sys::warenaitems::WAIdxBindGroup;
use crate::sys::warenaitems::WAIdxBuffer;
use crate::sys::warenaitems::WAIdxRenderPipeline;
use crate::sys::warenaitems::WAIdxRt;
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::warenaitems::WAIdxUbo;
use crate::sys::warenaitems::WArenaItem;
use crate::sys::wbindgroup::WBindGroupsHaverTrait;
use crate::sys::wdevice::GLOBALS;
use crate::sys::wrenderstate::WRenderState;
use crate::sys::wtl::WTechLead;
use crate::sys::wrenderpipeline::WRenderPipeline;
use crate::sys::wrenderpipeline::WRenderPipelineTrait;
use crate::wvulkan::WVulkan;

use lyon::math::{Box2D, Point, point};
use lyon::path::{Winding, builder::BorderRadii};
use lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers};
use lyon::tessellation::geometry_builder::simple_builder;

use crate::abs::wthingtrait::WThingTrait;

use std::ptr::copy_nonoverlapping as memcpy;

use super::wthingtrait::init_thing_stuff;


declare_thing!(WThingText{
  font: WFont
});

impl_thing_trait!(WThingText{});

impl WThingText {
  pub fn new(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    prog_render: WAIdxShaderProgram,
    font: WFont,
  ) -> Self {
    let s = init_thing_stuff(w_v, w_tl, prog_render);
    let rp = s.2.get_mut();
    unsafe {
      rp.input_assembly.topology = vk::PrimitiveTopology::TRIANGLE_STRIP;
      rp.refresh_pipeline(&w_v.w_device.device, &w_tl);
    }

    // let w_tl = w_tl;
    // let w_device = &mut ;

    let mut vert_buff = {
      let buff = w_tl.new_buffer(w_v, BufferUsageFlags::STORAGE_BUFFER, 10000, false);
      buff.1.map(&mut w_v.w_device.device);
      buff.0
    };

    let mut indices_buff = w_tl.new_buffer(w_v, BufferUsageFlags::STORAGE_BUFFER, 10000, false);
    indices_buff.1.map(&mut w_v.w_device.device);

    let mut lyon_geom: VertexBuffers<Point, u16> = VertexBuffers::new();

    let lyon_fill_options = FillOptions::tolerance(0.1);
    
    let mut lyon_tesselator = FillTessellator::new();
    

    Self {
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
      font,
      render_state: s.9,
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
    // let w_tl = &mut w_v.w_tl;

    if let Some(rt) = rt {
      if self.rt.is_none() {
        self.rt = Some(rt);

        let rp = self.render_pipeline.get_mut();
        rp.set_pipeline_render_target(rt.get_mut());
        rp.refresh_pipeline(&w_device.device, &w_tl);
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


    let metadata_buff_arena_idx = self.font.gpu_metadata_buff;
    let atlas_texture_arena_idx = self.font.gpu_atlas;

    unsafe {
      self.push_constants_internal.reset_ptr();

      let ubo_address = (*GLOBALS.shared_ubo_arena)[self.ubo.idx] // make this shorter? no?
        .buff
        .get_bda_address();
      self.push_constants_internal.write(ubo_address);

      self.push_constants_internal.write(metadata_buff_arena_idx);
      self.push_constants_internal.write(atlas_texture_arena_idx);

      self.update_push_constants(w_device, command_buffer);

      w_device.device.cmd_draw(*command_buffer, 4, 1, 0, 0);

    }
  }
}





