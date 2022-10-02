use std::{
  borrow::BorrowMut,
  cell::Cell,
  collections::HashMap,
  ffi::CStr,
  mem::MaybeUninit,
};

use ash::{
  vk,
};


use crate::{
  res::wshader::{WProgram},
  wmemzeroed,
};

use super::wmanagers::{WAIdxBindGroup, WGrouper};

static entry_point: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

pub struct WComputePipeline {
  pub pipeline: Cell<vk::Pipeline>,
  pub pipeline_layout: vk::PipelineLayout,
  pub pipeline_layout_info: vk::PipelineLayoutCreateInfo,
  pub pipeline_info: vk::ComputePipelineCreateInfo,

  push_constant_range: vk::PushConstantRange,

  pub set_layouts_vec: Vec<vk::DescriptorSetLayout>,
}
impl WComputePipeline {

  pub fn new(
    device: &ash::Device,
    shader: &WProgram,
  ) -> Self {
    let push_constant_range = vk::PushConstantRange::builder()
      .offset(0)
      .size(256)
      .stage_flags(vk::ShaderStageFlags::ALL)
      .build();

    let mut pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder().build();

    pipeline_layout_info.push_constant_range_count = 1;
    pipeline_layout_info.p_push_constant_ranges = &push_constant_range;

    let pipeline_layout =
      unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();
    // let stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
    //   .stage(vk::ShaderStageFlags::COMPUTE)
    //   .name(entry_point);

    let stage_create_info = &shader.stages[0];
    let mut pipeline_info = vk::ComputePipelineCreateInfo::builder()
      .layout(pipeline_layout)
      .stage(
        *stage_create_info, // vk::ShaderStageFlags::COMPUTE
      ).build();

    let mut pipeline_layout = wmemzeroed!();

    let pipeline = wmemzeroed!();

    // let pipeline = Cell::new(
    //   unsafe {
    //     pipeline_layout =
    //       unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();
    //     // let info = std::mem::transmute(self.pipeline_info);
    //     // maybe not needed?
    //     pipeline_info.layout = pipeline_layout;
    //     device.create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
    //   }
    //   .unwrap()[0],
    // );

    
    

    let mut w = Self {
      pipeline,
      pipeline_layout,
      set_layouts_vec: vec![],
      push_constant_range,
      pipeline_layout_info,
      pipeline_info,
    };
    w
  }

  pub fn refresh_pipeline(
    &mut self,
    device: &ash::Device,
    w_grouper: &mut WGrouper,
    bind_groups: &HashMap<u32, WAIdxBindGroup>,
  ) {
    self.refresh_bind_group_layouts(w_grouper, bind_groups);

    self.pipeline.set(
      unsafe {
        self.pipeline_layout_info.p_push_constant_ranges = &self.push_constant_range;
        self.pipeline_layout =
          unsafe { device.create_pipeline_layout(&self.pipeline_layout_info, None) }.unwrap();
        // let info = std::mem::transmute(self.pipeline_info);
        // maybe not needed?
        self.pipeline_info.layout = self.pipeline_layout;
        device.create_compute_pipelines(vk::PipelineCache::null(), &[self.pipeline_info], None) .unwrap()[0]
      }
    );
  }

  pub fn set_pipeline_bind_groups<'a>(
    &mut self,
    // bindings: &HashMap<u32, &dyn WTraitBinding>,
    w_grouper: &mut WGrouper,
    bind_groups: &HashMap<u32, WAIdxBindGroup>,
  ) {
    self.refresh_bind_group_layouts(w_grouper, bind_groups);
  }

  fn refresh_bind_group_layouts(
    &mut self,
    // bindings: &HashMap<u32, &dyn WTraitBinding>,
    w_grouper: &mut WGrouper,
    bind_groups: &HashMap<u32, WAIdxBindGroup>,
  ) {
    self.set_layouts_vec.clear();

    // self.set_layouts_vec = bind_groups.iter().map(|binding|{
    //   let bind_group_layout = w_grouper.bind_groups_arena.get((*binding.1).idx).unwrap().descriptor_set_layout;
    //   bind_group_layout
    // }).collect();
    // let mut sets = vec![];
    for i in (0..2){
      match bind_groups.get(&i) {
          Some(__) => {
            let group = w_grouper.bind_groups_arena[__.idx].borrow_mut();
            // self.set_layouts_vec.push(bind_group_layout)
            let bind_group_layout = group.descriptor_set_layout;
            self.set_layouts_vec.push(bind_group_layout)
          },
          None => {},
      }
    }

    // bind_groups.iter().for_each(|binding| {
    //   let bind_group_layout = w_grouper
    //     .bind_groups_arena
    //     .get((*binding.1).idx)
    //     .unwrap()
    //     .descriptor_set_layout;
    //   self.set_layouts_vec.push(bind_group_layout)
    // });

    self.pipeline_layout_info.set_layout_count = self.set_layouts_vec.len() as u32;
    self.pipeline_layout_info.p_set_layouts = self.set_layouts_vec.as_ptr();
  }
  pub fn set_pipeline_shader(
    &mut self,
    shader: &WProgram,
  ) {
    unsafe {
      self.pipeline_info.stage = shader.stages[0];
    }
  }
}

// impl Default for WImage{
//     fn default() -> Self {
//         Self { handle: None, resx: 500, resy: 500, format: None, view: None }
//     }
// }
