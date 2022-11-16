use std::collections::HashMap;

use ash::vk;
use ash::vk::BufferUsageFlags;
use lyon::lyon_tessellation::FillBuilder;
use lyon::path::builder::NoAttributes;
use nalgebra_glm::Mat4;
use nalgebra_glm::Vec3;

use crate::declare_thing;
use crate::impl_thing_trait;
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
use crate::sys::wmanagers::WTechLead;
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


declare_thing!(WThingPath{
  vert_buff: WAIdxBuffer, 
  indices_buff: WAIdxBuffer,
  lyon_geom: VertexBuffers<Point, u16>,
  lyon_fill_options: FillOptions,
  lyon_tesselator: FillTessellator
});
impl_thing_trait!(WThingPath{});

impl WThingPath {
  pub fn path(&mut self){
    unsafe{
      self.lyon_geom.vertices.set_len(0);
      self.lyon_geom.indices.set_len(0);
    }

    let mut geometry_builder = simple_builder(&mut self.lyon_geom);
    
    let mut lyon_builder = self.lyon_tesselator.builder(
      &self.lyon_fill_options,
      &mut geometry_builder,
    );
      
    lyon_builder.add_rounded_rectangle(
      &Box2D { min: point(0.0, 0.0), max: point(100.0, 50.0) },
      &BorderRadii {
          top_left: 10.0,
          top_right: 5.0,
          bottom_left: 20.0,
          bottom_right: 25.0,
      },
      Winding::Positive,
    );

    lyon_builder.build();

    unsafe{
      memcpy(self.lyon_geom.vertices.as_ptr(), self.vert_buff.get().mapped_mems[0].cast(), self.lyon_geom.vertices.len());
      memcpy(self.lyon_geom.indices.as_ptr(), self.indices_buff.get().mapped_mems[0].cast(), self.lyon_geom.indices.len());
    }

  }
  pub fn new(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    prog_render: WAIdxShaderProgram,
  ) -> Self {
    let s = init_thing_stuff(w_v, w_tl, prog_render);

    // let w_tl = w_tl;
    let w_device = &mut w_v.w_device;

    let mut vert_buff = {
      let buff = w_tl.new_buffer(w_device, BufferUsageFlags::STORAGE_BUFFER, 10000, false);
      buff.1.map(&mut w_device.device);
      buff.0
    };

    let mut indices_buff = w_tl.new_buffer(w_device, BufferUsageFlags::STORAGE_BUFFER, 10000, false);
    indices_buff.1.map(&mut w_device.device);

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
      vert_buff: vert_buff,
      indices_buff: indices_buff.0,
      lyon_geom,
      lyon_fill_options,
      lyon_tesselator,
    }
  }

  #[profiling::function]
  pub fn draw(
    &mut self,
    w_v: &mut WVulkan,
    rt: Option<WAIdxRt>,
    command_buffer: &vk::CommandBuffer,
  ) {
    let w_device = &mut w_v.w_device;
    let w_grouper = &mut w_v.w_grouper;
    // let w_tl = &mut w_v.w_tl;

    if let Some(rt) = rt {
      if self.rt.is_none() {
        self.rt = Some(rt);

        let rp = self.render_pipeline.get_mut();
        rp.set_pipeline_render_target(rt.get_mut());
        rp.refresh_pipeline(&w_device.device, w_grouper);
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

    self.init_render_settings(w_device, w_grouper, command_buffer);


    let indices_arena_idx = self.indices_buff;
    let verts_arena_idx = self.vert_buff;

    unsafe {
      self.push_constants_internal.reset_ptr();

      let ubo_address = (*GLOBALS.shared_ubo_arena)[self.ubo.idx] // make this shorter? no?
        .buff
        .get_bda_address();
      self.push_constants_internal.write(ubo_address);

      self.push_constants_internal.write(indices_arena_idx.idx.index as u8 - 1);
      self.push_constants_internal.write(verts_arena_idx.idx.index as u8 - 1);

      self.update_push_constants(w_device, command_buffer);

      w_device.device.cmd_draw(*command_buffer, 3*self.lyon_geom.indices.len() as u32, 1, 0, 0);
    }
  }
}





