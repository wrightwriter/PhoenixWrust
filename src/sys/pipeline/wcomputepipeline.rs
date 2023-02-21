use std::{
  borrow::{Borrow},
  cell::Cell,
  collections::HashMap,
  ffi::CStr,
};

use ash::{
  vk,
};



use crate::{
  res::wshader::{WProgram},
  wmemzeroed, wvulkan::WVulkan,
};

use super::super::{warenaitems::{WAIdxBindGroup, WAIdxShaderProgram}, wtl::WTechLead};
use super::super::{wtl, wdevice::GLOBALS};

static entry_point: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

pub struct WComputePipeline {
  pub pipeline: Cell<vk::Pipeline>,
  pub pipeline_layout: vk::PipelineLayout,
  pub pipeline_layout_info: vk::PipelineLayoutCreateInfo,
  pub pipeline_info: vk::ComputePipelineCreateInfo,

  push_constant_range: vk::PushConstantRange,
  
  pub shader_program: WAIdxShaderProgram,

  pub set_layouts_vec: Vec<vk::DescriptorSetLayout>,

  pub bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
}
impl WComputePipeline {
  pub fn new(
    device: &ash::Device,
    // shader: &WProgram,
    shader_program: WAIdxShaderProgram,
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
    

    // let stage_create_info = ;
    let mut pipeline_info = vk::ComputePipelineCreateInfo::builder()
      .layout(pipeline_layout)
      .build();

    let mut pipeline_layout = wmemzeroed!();

    let pipeline = wmemzeroed!();


    let mut set_layouts_vec = vec![];
    set_layouts_vec.reserve(8);


    let mut w = Self {
      pipeline,
      pipeline_layout,
      set_layouts_vec,
      push_constant_range,
      pipeline_layout_info,
      pipeline_info,
      shader_program,
      bind_groups: wmemzeroed!(),
    };
    w
  }
  
  pub fn init(&mut self){
    self.pipeline_layout_info.p_push_constant_ranges = &self.push_constant_range;
    self.pipeline_layout_info.p_set_layouts = self.set_layouts_vec.as_ptr();
    unsafe{
      self.pipeline_info.stage = (*GLOBALS.shader_programs_arena)[self.shader_program.idx].borrow().stages[0] ;
    }
  }

  pub fn refresh_pipeline(
    &mut self,
    // device: &ash::Device,
    w_v: &mut WVulkan,
    w_tl: &WTechLead,
    // bind_groups: &HashMap<u32, WAIdxBindGroup>,
  ) {
    self.refresh_bind_group_layouts(w_tl, self.bind_groups);

    unsafe{
      self.pipeline_info.stage = (*GLOBALS.shader_programs_arena)[self.shader_program.idx].borrow().stages[0] ;
    }

    // (*self.shader_stages).set_len(0);
    // for i in 0..2 {
    //   // (*self.shader_stages)[i] = (*GLOBALS.shaders_arena)[self.shader_program.idx].stages[i];
    //   (*self.shader_stages).push(
    //     (*GLOBALS.shader_programs_arena)[self.shader_program.idx].stages[i]
    //   );
    // }

    self.pipeline.set(
      unsafe {

        self.pipeline_layout =
          unsafe { w_v.w_device.device.create_pipeline_layout(&self.pipeline_layout_info, None) }.unwrap();
        // let info = std::mem::transmute(self.pipeline_info);
        // maybe not needed?
        self.pipeline_info.layout = self.pipeline_layout;
        w_v.w_device.device.create_compute_pipelines(vk::PipelineCache::null(), &[self.pipeline_info], None) .unwrap()[0]
      }
    );
  }

  pub fn set_pipeline_bind_groups<'a>(
    &mut self,
    // bindings: &HashMap<u32, &dyn WTraitBinding>,
    w_tl: &mut WTechLead,
    bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
  ) {
    self.bind_groups = bind_groups;
    self.refresh_bind_group_layouts(w_tl, bind_groups);
  }

  fn refresh_bind_group_layouts(
    &mut self,
    // bindings: &HashMap<u32, &dyn WTraitBinding>,
    w_tl: &WTechLead,
    bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
  ) {

    unsafe{
      self.set_layouts_vec.set_len(0);
    }


    let bind_groups = unsafe{&mut *bind_groups};
    // self.set_layouts_vec = bind_groups.iter().map(|binding|{
    //   let bind_group_layout = w_grouper.bind_groups_arena.get((*binding.1).idx).unwrap().descriptor_set_layout;
    //   bind_group_layout
    // }).collect();
    // let mut sets = vec![];
    for i in 0..2 {
      match bind_groups.get(&i) {
          Some(__) => unsafe {
            // w_grouper.bind_groups_arena.borrow();

            let group = (&*GLOBALS.bind_groups_arena)[__.idx].borrow();
            // self.set_layouts_vec.push(bind_group_layout)
            let bind_group_layout = group.descriptor_set_layout;
            self.set_layouts_vec.push(bind_group_layout)
          },
          None => {},
      }
    }

    self.pipeline_layout_info.set_layout_count = self.set_layouts_vec.len() as u32;

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
