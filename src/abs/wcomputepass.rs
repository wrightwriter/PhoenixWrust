
use std::borrow::BorrowMut;
use std::collections::HashMap;

use ash::vk;

use ash::vk::CommandBuffer;
use ash::vk::DescriptorSet;


use crate::res::wshader::WShaderEnumPipelineBind;
use crate::sys::wbindgroup::WBindGroupsHaverTrait;
use crate::sys::wcomputepipeline::WComputePipeline;

use crate::sys::wdevice::GLOBALS;
use crate::sys::warenaitems::WAIdxBindGroup;
use crate::sys::warenaitems::WAIdxComputePipeline;
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::warenaitems::WArenaItem;

use crate::sys::wmanagers::WTechLead;
use crate::wvulkan::WVulkan;

pub struct WComputePass {
  pub compute_pipeline: WAIdxComputePipeline,
  pub shader_program: WAIdxShaderProgram,
  pub command_buffer: CommandBuffer,
  pub bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
}

impl WComputePass {
  pub fn new(
    w_v: &mut WVulkan,
    shader_program: WAIdxShaderProgram,
  ) -> Self {

    let w_device = &mut w_v.w_device;
    let w_grouper = &mut w_v.w_grouper;
    let w_tech_lead = &mut w_v.w_tl;
    let shared_bind_group = w_v.shared_bind_group;

    let mut compute_pipeline = WAIdxComputePipeline {
      idx: unsafe {
        (&mut *GLOBALS.shared_compute_pipelines)
          .insert(WComputePipeline::new(&w_device.device, shader_program))
      },
    };

    unsafe {
      compute_pipeline.get_mut().init();
    }

// useless
    unsafe {
      match &mut (*GLOBALS.shader_programs_arena)[shader_program.idx].comp_shader {
        Some(__) => {
          __.pipelines
            .push(WShaderEnumPipelineBind::ComputePipeline(compute_pipeline));
        }
        None => {}
      }
    }
    

    let ubo = w_tech_lead.new_uniform_buffer(w_device, 1000).0;

    let mut personal_bind_group_idx = {
      let bind_group = w_grouper.new_group(w_device);
      bind_group.1.set_binding_ubo(0, ubo.idx);

      // NEED TO REBUILD LATER TOO ? 
      bind_group
        .1
        .rebuild_all(&w_device.device, &w_device.descriptor_pool, w_tech_lead);
      bind_group.0
    };


    let mut bind_groups = unsafe{
      let bind_groups = ptralloc!( HashMap<u32, WAIdxBindGroup>);
      std::ptr::write(bind_groups, HashMap::new());


      (*bind_groups).insert(0, shared_bind_group);
      (*bind_groups).insert(1, personal_bind_group_idx);

      bind_groups
    };


    compute_pipeline
      .get_mut()
      .set_pipeline_bind_groups(w_grouper, bind_groups);

    compute_pipeline
      .get_mut()
      .refresh_pipeline(&w_device.device, w_grouper);

    
    Self {
      compute_pipeline,
      shader_program,
      command_buffer: wmemzeroed!(),
      bind_groups,
    }
  }

  pub fn dispatch(
    &mut self,
    wv: &mut WVulkan,
    wkg_sz_x: u32,
    wkg_sz_y: u32,
    wkg_sz_z: u32,
  ) {
    let w_device = &mut wv.w_device;
    let w_grouper = &mut wv.w_grouper;
    // w_grouper: &WGrouper,
    self.command_buffer = w_device.curr_pool().get_cmd_buff();

    let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe {
      // let barrier = vk::MemoryBarri
      // w_device.device.reset_command_buffer(
      //   self.command_buffer,
      //   vk::CommandBufferResetFlags::RELEASE_RESOURCES,
      // );
      w_device
        .device
        .begin_command_buffer(self.command_buffer, &cmd_buf_begin_info)
        .unwrap();

      // let mut sets = vec![];
      let mut sets: [DescriptorSet; 2] = wmemzeroed!();
      for i in 0..2 {
        match (&*self.bind_groups).get(&i) {
          Some(__) => sets[i as usize] = w_grouper.bind_groups_arena[__.idx].descriptor_set,
          None => {}
        }
      }

      w_device.device.cmd_bind_descriptor_sets(
        self.command_buffer,
        vk::PipelineBindPoint::COMPUTE,
        self.compute_pipeline.get_mut().pipeline_layout,
        0,
        &sets,
        &[],
      );
      w_device.device.cmd_bind_pipeline(
        self.command_buffer,
        vk::PipelineBindPoint::COMPUTE,
        self.compute_pipeline.get_mut().pipeline.get(),
      );

      w_device
        .device
        .cmd_dispatch(self.command_buffer, wkg_sz_x, wkg_sz_y, wkg_sz_z);

      w_device
        .device
        .end_command_buffer(self.command_buffer)
        .unwrap();
    }
  }
}

impl WBindGroupsHaverTrait for WComputePass {
  fn get_bind_groups(&self) -> &HashMap<u32, WAIdxBindGroup> {
    unsafe{
      &*self.bind_groups
    }
  }
}
